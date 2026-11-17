// Eclipse 1.2 - Lanzador del servidor LSP para VS Code
// -------------------------------------------------------
// ESTE ARCHIVO ES SOLO UN PUENTE. No sabe nada de Eclipse:
// no tiene las palabras clave, no completa nada, no explica
// nada. Todo eso vive en el programa `eclipse`, que es Rust
// puro. Este archivo solo hace tres cosas:
//
//   1. Enciende el servidor:  eclipse --lsp
//   2. Le manda lo que escribes (abrir, cambiar, cerrar)
//   3. Le pide autocompletado y ayuda (hover), y te muestra
//      lo que el servidor responde.
//   4. Te pone un botón ▶ Ejecutar para correr tu archivo
//      .ec en la terminal de VS Code (sin salir del editor).
//
// ¿Por qué hay JavaScript aquí si Eclipse es "puro Rust"?
// Porque VS Code solo acepta extensiones en JavaScript o
// TypeScript: es la única forma de encender un servidor
// desde VS Code. Pero la inteligencia es 100% Rust: si
// usas otro editor (Neovim, Helix, Zed), no necesitas
// este archivo para nada, solo `eclipse --lsp`.
//
// Para que funcione, VS Code tiene que encontrar el programa
// `eclipse`. Por defecto busca `eclipse` en el PATH. Si lo
// tienes en otro lugar, pon su ruta en la configuración:
//   "eclipse.ruta": "C:\\ruta\\a\\eclipse.exe"
//
// Para compilar el programa eclipse:
//   cargo build --release
// El programa queda en target/release/eclipse.exe (Windows)
// o target/release/eclipse (Linux/Mac).

const vscode = require('vscode');
const { spawn } = require('child_process');

// --- El cliente LSP (mínimo) --------------------------------
// Habla JSON-RPC con el servidor por stdin/stdout, con el
// formato que pide el protocolo: una cabecera
// "Content-Length: N" y después el mensaje JSON.

let servidor = null;      // el proceso eclipse --lsp
let siguienteId = 1;      // el número del próximo pedido
let pendientes = new Map(); // id -> función que recibe la respuesta
let bufer = Buffer.alloc(0); // lo que llegó del servidor y aún no se leyó
let iniciado = false;

function rutaEclipse() {
    const config = vscode.workspace.getConfiguration('eclipse');
    return config.get('ruta', 'eclipse');
}

// Manda un mensaje al servidor.
function mandar(mensaje) {
    if (!servidor || !servidor.stdin.writable) {
        return;
    }
    const texto = JSON.stringify(mensaje);
    servidor.stdin.write(
        'Content-Length: ' + Buffer.byteLength(texto, 'utf8') + '\r\n\r\n' + texto,
        'utf8'
    );
}

// Manda un pedido y espera la respuesta (una promesa).
function pedir(metodo, params) {
    return new Promise((resolver) => {
        const id = siguienteId++;
        pendientes.set(id, resolver);
        mandar({ jsonrpc: '2.0', id: id, method: metodo, params: params });
        // Si el servidor no responde en 5 segundos, no te dejo esperando.
        setTimeout(() => {
            if (pendientes.has(id)) {
                pendientes.delete(id);
                resolver(null);
            }
        }, 5000);
    });
}

// Lee las respuestas del servidor, que pueden llegar partidas.
function alRecibir(datos) {
    bufer = Buffer.concat([bufer, datos]);
    for (;;) {
        const finCabecera = bufer.indexOf('\r\n\r\n');
        if (finCabecera === -1) {
            return; // todavía no llegó la cabecera completa
        }
        const cabecera = bufer.slice(0, finCabecera).toString('utf8');
        const m = cabecera.match(/Content-Length:\s*(\d+)/i);
        if (!m) {
            return;
        }
        const largo = parseInt(m[1], 10);
        const inicio = finCabecera + 4;
        if (bufer.length < inicio + largo) {
            return; // todavía no llegó el cuerpo completo
        }
        const cuerpo = bufer.slice(inicio, inicio + largo).toString('utf8');
        bufer = bufer.slice(inicio + largo);
        let mensaje;
        try {
            mensaje = JSON.parse(cuerpo);
        } catch (e) {
            continue;
        }
        if (mensaje.id !== undefined && pendientes.has(mensaje.id)) {
            const resolver = pendientes.get(mensaje.id);
            pendientes.delete(mensaje.id);
            resolver(mensaje.result !== undefined ? mensaje.result : null);
        }
        // Los avisos del servidor (sin id) se ignoran: este
        // servidor solo responde pedidos, no manda avisos.
    }
}

function uriDe(documento) {
    return documento.uri.toString();
}

function encenderServidor() {
    if (servidor) {
        return;
    }
    servidor = spawn(rutaEclipse(), ['--lsp'], { stdio: ['pipe', 'pipe', 'ignore'] });
    servidor.stdout.on('data', alRecibir);
    servidor.on('exit', () => {
        servidor = null;
        iniciado = false;
        pendientes.clear();
    });
    servidor.on('error', () => {
        servidor = null;
        iniciado = false;
        vscode.window.showErrorMessage(
            'No pude encender el servidor de Eclipse. Revisa la configuración "eclipse.ruta".'
        );
    });
    // El saludo del protocolo: initialize + initialized.
    pedir('initialize', {
        processId: process.pid,
        rootUri: vscode.workspace.workspaceFolders
            ? vscode.workspace.workspaceFolders[0].uri.toString()
            : null,
        capabilities: {},
    }).then(() => {
        iniciado = true;
        mandar({ jsonrpc: '2.0', method: 'initialized', params: {} });
        // Avisar de los archivos .ec que ya están abiertos.
        for (const doc of vscode.workspace.textDocuments) {
            if (doc.languageId === 'eclipse') {
                avisarAbrir(doc);
            }
        }
    });
}

function avisarAbrir(documento) {
    mandar({
        jsonrpc: '2.0',
        method: 'textDocument/didOpen',
        params: {
            textDocument: {
                uri: uriDe(documento),
                languageId: 'eclipse',
                version: documento.version,
                text: documento.getText(),
            },
        },
    });
}

function avisarCambiar(documento) {
    mandar({
        jsonrpc: '2.0',
        method: 'textDocument/didChange',
        params: {
            textDocument: { uri: uriDe(documento), version: documento.version },
            contentChanges: [{ text: documento.getText() }],
        },
    });
}

function avisarCerrar(documento) {
    mandar({
        jsonrpc: '2.0',
        method: 'textDocument/didClose',
        params: { textDocument: { uri: uriDe(documento) } },
    });
}

function tipoDeItem(kind) {
    if (kind === 14) {
        return vscode.CompletionItemKind.Keyword;
    }
    if (kind === 3) {
        return vscode.CompletionItemKind.Function;
    }
    return vscode.CompletionItemKind.Variable;
}

// --- El botón ▶ Ejecutar ----------------------------------
// Corre el archivo .ec que tienes abierto, en la terminal
// de VS Code. Guarda el archivo primero, para que corra
// lo último que escribiste. Si el archivo no está guardado
// en el disco (es nuevo y sin nombre), te avisa.
// `extra` son los argumentos de Eclipse: '' para el
// intérprete, '--vm' para la máquina virtual y
// '--depurar' para el depurador paso a paso.
async function ejecutarArchivo(extra) {
    const editor = vscode.window.activeTextEditor;
    if (!editor || editor.document.languageId !== 'eclipse') {
        vscode.window.showInformationMessage('Abre un archivo .ec para ejecutarlo.');
        return;
    }
    const doc = editor.document;
    if (doc.isUntitled) {
        vscode.window.showWarningMessage('Guarda el archivo .ec primero, para poder ejecutarlo.');
        return;
    }
    await doc.save();
    const ruta = rutaEclipse();
    const archivo = doc.fileName;
    // Las comillas son para que funcionen las rutas con espacios.
    const comando = '"' + ruta + '" ' + (extra ? extra + ' ' : '') + '"' + archivo + '"';
    let terminal = vscode.window.terminals.find((t) => t.name === 'Eclipse');
    if (!terminal) {
        terminal = vscode.window.createTerminal('Eclipse');
    }
    terminal.show();
    terminal.sendText(comando);
}

function activate(context) {
    encenderServidor();

    // El botón ▶ Ejecutar (título, paleta y atajo Ctrl+Alt+R)
    // y el enlace "▶ Ejecutar" arriba del archivo.
    // También: Ejecutar con VM (Ctrl+Alt+V) y con el
    // depurador (Ctrl+Alt+D), en la paleta de comandos.
    context.subscriptions.push(
        vscode.commands.registerCommand('eclipse.ejecutar', () => ejecutarArchivo('')),
        vscode.commands.registerCommand('eclipse.ejecutarVm', () => ejecutarArchivo('--vm')),
        vscode.commands.registerCommand('eclipse.ejecutarDepurador', () => ejecutarArchivo('--depurar')),
        vscode.languages.registerCodeLensProvider('eclipse', {
            provideCodeLenses(document) {
                const rango = new vscode.Range(0, 0, 0, 0);
                return [
                    new vscode.CodeLens(rango, {
                        title: '▶ Ejecutar',
                        command: 'eclipse.ejecutar',
                    }),
                ];
            },
        })
    );

    // Cuando abres, cambias o cierras un .ec, avisar al servidor.
    context.subscriptions.push(
        vscode.workspace.onDidOpenTextDocument((doc) => {
            if (doc.languageId === 'eclipse') {
                avisarAbrir(doc);
            }
        }),
        vscode.workspace.onDidChangeTextDocument((e) => {
            if (e.document.languageId === 'eclipse') {
                avisarCambiar(e.document);
            }
        }),
        vscode.workspace.onDidCloseTextDocument((doc) => {
            if (doc.languageId === 'eclipse') {
                avisarCerrar(doc);
            }
        })
    );

    // Autocompletado: le pregunto al servidor en tu posición.
    context.subscriptions.push(
        vscode.languages.registerCompletionItemProvider('eclipse', {
            async provideCompletionItems(document, position) {
                if (!iniciado) {
                    return [];
                }
                const respuesta = await pedir('textDocument/completion', {
                    textDocument: { uri: uriDe(document) },
                    position: { line: position.line, character: position.character },
                });
                if (!respuesta || !respuesta.items) {
                    return [];
                }
                return respuesta.items.map((it) => {
                    const item = new vscode.CompletionItem(it.label, tipoDeItem(it.kind));
                    if (it.detail) {
                        item.detail = it.detail;
                    }
                    if (it.documentation) {
                        const texto =
                            typeof it.documentation === 'string'
                                ? it.documentation
                                : it.documentation.value;
                        item.documentation = new vscode.MarkdownString(texto);
                    }
                    return item;
                });
            },
        })
    );

    // Ayuda al pasar el mouse: le pregunto al servidor qué es eso.
    context.subscriptions.push(
        vscode.languages.registerHoverProvider('eclipse', {
            async provideHover(document, position) {
                if (!iniciado) {
                    return null;
                }
                const respuesta = await pedir('textDocument/hover', {
                    textDocument: { uri: uriDe(document) },
                    position: { line: position.line, character: position.character },
                });
                if (!respuesta || !respuesta.contents) {
                    return null;
                }
                const texto =
                    typeof respuesta.contents === 'string'
                        ? respuesta.contents
                        : respuesta.contents.value;
                return new vscode.Hover(new vscode.MarkdownString(texto));
            },
        })
    );
}

function deactivate() {
    if (servidor) {
        mandar({ jsonrpc: '2.0', id: siguienteId++, method: 'shutdown', params: null });
        mandar({ jsonrpc: '2.0', method: 'exit' });
        servidor.kill();
        servidor = null;
    }
}

module.exports = { activate, deactivate };
