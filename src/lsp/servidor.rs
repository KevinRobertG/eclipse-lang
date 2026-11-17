// ============================================================
//  ECLIPSE 1.2 - El servidor LSP (en Rust puro)
//  Archivo: src/lsp/servidor.rs
//
//  Qué es esto:
//  Un servidor del Protocolo de Servidor de Lenguaje (LSP):
//  es el programa que le dice a tu editor qué puedes escribir
//  y qué significa cada palabra, mientras escribes. Habla con
//  el editor por la entrada y la salida estándar, con mensajes
//  JSON que dicen `Content-Length: N` y luego el JSON.
//
//  Qué sabe hacer:
//  - `initialize`: dice qué sabe hacer (autocompletado, hover,
//    diagnósticos y esquema del documento).
//  - `textDocument/didOpen` y `didChange`: guarda el texto
//    del archivo que estás editando y te avisa de sus errores
//    (diagnósticos) en cuanto lo abres o lo cambias.
//  - `textDocument/completion`: sugiere qué escribir (usa el
//    autocompletado de Eclipse, que conoce tus variables,
//    tus funciones, las palabras clave y las funciones).
//  - `textDocument/hover`: explica qué es la palabra que
//    tienes debajo del ratón.
//  - `textDocument/documentSymbol`: el esquema del archivo
//    (tus variables, constantes y funciones, con su línea).
//  - `textDocument/definition`: salta a donde se definió la
//    palabra que tienes debajo del ratón.
//  - `textDocument/references`: todos los lugares donde se
//    usa esa palabra.
//  - `shutdown` y `exit`: se despide y se apaga.
//
//  No hay JavaScript aquí: todo es Rust, y la inteligencia
//  vive en `autocompletar.rs`. Para usarlo en VS Code hay
//  un lanzador chiquito (extension.js) que solo arranca este
//  servidor; en otros editores (Neovim, Helix, Zed) se usa
//  directo, sin JavaScript.
//
//  Para probarlo a mano:
//      eclipse --lsp
//  (y se le escriben mensajes JSON-RPC por la entrada).
// ============================================================

use crate::herramientas::autocompletar;
use crate::lenguaje::lexer::Lexer;
use crate::lenguaje::parser::Parser;
use crate::lsp::json::{arreglo, booleano, entero, objeto, texto, Json};
use std::collections::HashMap;
use std::io::{Read, Write};

// Arranca el servidor LSP y no vuelve hasta que el editor
// le dice que se apague (o se cierra la entrada).
pub fn servidor() {
    let mut documentos: HashMap<String, String> = HashMap::new();
    let mut entrada = std::io::stdin();
    let mut salida = std::io::stdout();
    let mut apagado = false;
    loop {
        let mensaje = match leer_mensaje(&mut entrada) {
            Some(m) => m,
            // Se cerró la entrada: adiós.
            None => return,
        };
        let respuestas = manejar(&mensaje, &mut documentos, &mut apagado);
        for respuesta in respuestas {
            escribir_mensaje(&mut salida, &respuesta);
        }
        if apagado {
            return;
        }
    }
}

// Lee un mensaje JSON-RPC de la entrada: primero las
// cabeceras (`Content-Length: N`) y luego N bytes de JSON.
fn leer_mensaje(entrada: &mut std::io::Stdin) -> Option<Json> {
    let mut longitud: Option<usize> = None;
    let mut byte = [0u8; 1];
    let mut linea = Vec::new();
    loop {
        // Leer una línea de cabecera, byte a byte.
        linea.clear();
        loop {
            match entrada.read(&mut byte) {
                Ok(0) => return None,
                Ok(_) => {
                    if byte[0] == b'\n' {
                        break;
                    }
                    linea.push(byte[0]);
                }
                Err(_) => return None,
            }
        }
        // Quitar el \r del final.
        if linea.last() == Some(&b'\r') {
            linea.pop();
        }
        if linea.is_empty() {
            break;
        }
        let texto = String::from_utf8_lossy(&linea).to_string();
        let minusculas = texto.to_lowercase();
        if let Some(valor) = minusculas.strip_prefix("content-length:") {
            longitud = valor.trim().parse::<usize>().ok();
        }
    }
    let n = longitud?;
    let mut cuerpo = vec![0u8; n];
    if entrada.read_exact(&mut cuerpo).is_err() {
        return None;
    }
    let texto = String::from_utf8_lossy(&cuerpo).to_string();
    Json::parsear(&texto).ok()
}

// Escribe un mensaje JSON-RPC en la salida.
fn escribir_mensaje(salida: &mut std::io::Stdout, mensaje: &Json) {
    let cuerpo = mensaje.texto();
    let _ = write!(
        salida,
        "Content-Length: {}\r\n\r\n{}",
        cuerpo.len(),
        cuerpo
    );
    let _ = salida.flush();
}

// Responde a un mensaje. Devuelve los mensajes que hay que
// escribir en la salida: la respuesta (si es una petición)
// y, al abrir o cambiar un archivo, el aviso de
// diagnósticos (que es una notificación aparte).
fn manejar(
    mensaje: &Json,
    documentos: &mut HashMap<String, String>,
    apagado: &mut bool,
) -> Vec<Json> {
    let metodo = mensaje.get("method").and_then(|m| m.como_texto());
    let id = mensaje.get("id").cloned();
    let params = mensaje.get("params").cloned().unwrap_or(Json::Nulo);
    match metodo {
        Some("initialize") => vec![responder(
            id,
            objeto(vec![
                (
                    "capabilities",
                    objeto(vec![
                        ("textDocumentSync", entero(1)),
                        (
                            "completionProvider",
                            objeto(vec![
                                (
                                    "triggerCharacters",
                                    arreglo(vec![texto("."), texto("("), texto(" ")]),
                                ),
                                ("resolveProvider", booleano(false)),
                            ]),
                        ),
                        ("hoverProvider", booleano(true)),
                        ("documentSymbolProvider", booleano(true)),
                        ("definitionProvider", booleano(true)),
                        ("referencesProvider", booleano(true)),
                    ]),
                ),
                (
                    "serverInfo",
                    objeto(vec![
                        ("name", texto("Eclipse")),
                        ("version", texto("1.2")),
                    ]),
                ),
            ]),
        )],
        Some("initialized") => vec![],
        Some("textDocument/didOpen") => {
            let mut avisos = Vec::new();
            if let Some(doc) = params.get("textDocument") {
                let uri = doc
                    .get("uri")
                    .and_then(|u| u.como_texto())
                    .unwrap_or("")
                    .to_string();
                let texto_doc = doc
                    .get("text")
                    .and_then(|t| t.como_texto())
                    .unwrap_or("")
                    .to_string();
                documentos.insert(uri.clone(), texto_doc.clone());
                // En cuanto se abre el archivo, se avisa de
                // sus errores (o de que no tiene ninguno).
                avisos.push(diagnosticar(&uri, &texto_doc));
            }
            avisos
        }
        Some("textDocument/didChange") => {
            let mut avisos = Vec::new();
            if let Some(doc) = params.get("textDocument") {
                let uri = doc
                    .get("uri")
                    .and_then(|u| u.como_texto())
                    .unwrap_or("")
                    .to_string();
                // Sincronización completa: el cambio trae el
                // texto entero del archivo.
                let nuevo = params
                    .get("contentChanges")
                    .and_then(|c| match c {
                        Json::Arreglo(lista) => lista.first(),
                        _ => None,
                    })
                    .and_then(|p| p.get("text"))
                    .and_then(|t| t.como_texto());
                if let Some(nuevo) = nuevo {
                    documentos.insert(uri.clone(), nuevo.to_string());
                    // Cada cambio vuelve a revisar los errores.
                    avisos.push(diagnosticar(&uri, nuevo));
                }
            }
            avisos
        }
        Some("textDocument/didClose") => {
            if let Some(doc) = params.get("textDocument") {
                if let Some(uri) = doc.get("uri").and_then(|u| u.como_texto()) {
                    documentos.remove(uri);
                }
            }
            vec![]
        }
        Some("textDocument/completion") => {
            let resultado = completar(params, documentos);
            vec![responder(id, resultado)]
        }
        Some("textDocument/hover") => {
            let resultado = hover(params, documentos);
            vec![responder(id, resultado)]
        }
        Some("textDocument/documentSymbol") => {
            let resultado = simbolos_documento(params, documentos);
            vec![responder(id, resultado)]
        }
        Some("textDocument/definition") => {
            let resultado = definicion(params, documentos);
            vec![responder(id, resultado)]
        }
        Some("textDocument/references") => {
            let resultado = referencias(params, documentos);
            vec![responder(id, resultado)]
        }
        Some("shutdown") => {
            *apagado = true;
            vec![responder(id, Json::Nulo)]
        }
        Some("exit") => {
            *apagado = true;
            vec![]
        }
        Some("$/cancelRequest") => vec![],
        Some(_) => {
            // Un método que no conocemos: error, si es una
            // petición (las notificaciones no llevan respuesta).
            match id {
                Some(id) => vec![responder_error(id, -32601, "ese método no lo conozco")],
                None => vec![],
            }
        }
        None => vec![],
    }
}

// Una respuesta buena: { "jsonrpc": "2.0", "id": ..., "result": ... }
fn responder(id: Option<Json>, resultado: Json) -> Json {
    objeto(vec![
        ("jsonrpc", texto("2.0")),
        ("id", id.unwrap_or(Json::Nulo)),
        ("result", resultado),
    ])
}

// Una respuesta con error: { "jsonrpc": "2.0", "id": ..., "error": ... }
fn responder_error(id: Json, codigo: i64, mensaje: &str) -> Json {
    objeto(vec![
        ("jsonrpc", texto("2.0")),
        ("id", id),
        (
            "error",
            objeto(vec![
                ("code", entero(codigo)),
                ("message", texto(mensaje)),
            ]),
        ),
    ])
}

// La posición (línea, columna) de una petición, en base 1.
fn posicion(params: &Json) -> (usize, usize) {
    let linea = params
        .get("position")
        .and_then(|p| p.get("line"))
        .and_then(|l| l.como_entero())
        .unwrap_or(0) as usize
        + 1;
    let caracter = params
        .get("position")
        .and_then(|p| p.get("character"))
        .and_then(|c| c.como_entero())
        .unwrap_or(0) as usize
        + 1;
    (linea, caracter)
}

// El texto del documento de una petición, por su uri.
fn texto_documento(params: &Json, documentos: &HashMap<String, String>) -> String {
    let uri = params
        .get("textDocument")
        .and_then(|d| d.get("uri"))
        .and_then(|u| u.como_texto())
        .unwrap_or("");
    documentos.get(uri).cloned().unwrap_or_default()
}

// Autocompletado: qué puedes escribir en esa posición.
fn completar(params: Json, documentos: &HashMap<String, String>) -> Json {
    let (linea, columna) = posicion(&params);
    let fuente = texto_documento(&params, documentos);
    let sugerencias = autocompletar::completar(&fuente, linea, columna);
    let mut items = Vec::new();
    for s in sugerencias {
        // kind: 6 = Variable, 3 = Function, 14 = Keyword.
        let kind = if s.tipo == "palabra clave" {
            14
        } else if s.tipo.starts_with("función") {
            3
        } else {
            6
        };
        items.push(objeto(vec![
            ("label", texto(&s.nombre)),
            ("kind", entero(kind)),
            ("detail", texto(&s.tipo)),
            (
                "documentation",
                objeto(vec![("kind", texto("plaintext")), ("value", texto(&s.descripcion))]),
            ),
        ]));
    }
    objeto(vec![
        ("isIncomplete", booleano(false)),
        ("items", arreglo(items)),
    ])
}

// Hover: qué es la palabra que tienes debajo del ratón.
fn hover(params: Json, documentos: &HashMap<String, String>) -> Json {
    let (linea, columna) = posicion(&params);
    let fuente = texto_documento(&params, documentos);
    let palabra = autocompletar::palabra_en(&fuente, linea, columna);
    if palabra.is_empty() {
        return Json::Nulo;
    }
    match autocompletar::describir(&fuente, linea, columna, &palabra) {
        Some(info) => objeto(vec![(
            "contents",
            objeto(vec![
                ("kind", texto("markdown")),
                (
                    "value",
                    texto(&format!("**{}** ({})\n\n{}", info.nombre, info.tipo, info.descripcion)),
                ),
            ]),
        )]),
        None => Json::Nulo,
    }
}

// El esquema del documento: tus variables, constantes y
// funciones, con la línea donde se definieron. Es lo que
// ves en el esquema de tu editor.
fn simbolos_documento(params: Json, documentos: &HashMap<String, String>) -> Json {
    let fuente = texto_documento(&params, documentos);
    let simbolos = autocompletar::simbolos(&fuente);
    let mut items = Vec::new();
    for s in simbolos {
        // La línea es base 1 en Eclipse y base 0 en LSP.
        // El rango cubre el nombre entero.
        let linea = (s.linea.saturating_sub(1)) as i64;
        let fin = s.nombre.chars().count() as i64;
        let rango = objeto(vec![
            (
                "start",
                objeto(vec![("line", entero(linea)), ("character", entero(0))]),
            ),
            (
                "end",
                objeto(vec![("line", entero(linea)), ("character", entero(fin))]),
            ),
        ]);
        items.push(objeto(vec![
            ("name", texto(&s.nombre)),
            ("kind", entero(s.kind as i64)),
            ("range", rango.clone()),
            ("selectionRange", rango),
        ]));
    }
    arreglo(items)
}

// Un lugar (Location) del LSP: la uri y el rango que
// cubre la palabra en su línea (líneas y columnas base 0).
fn ubicacion(uri: &str, linea: usize, columna: usize, largo: usize) -> Json {
    let linea_lsp = (linea.saturating_sub(1)) as i64;
    let col_lsp = (columna.saturating_sub(1)) as i64;
    objeto(vec![
        ("uri", texto(uri)),
        (
            "range",
            objeto(vec![
                (
                    "start",
                    objeto(vec![("line", entero(linea_lsp)), ("character", entero(col_lsp))]),
                ),
                (
                    "end",
                    objeto(vec![
                        ("line", entero(linea_lsp)),
                        ("character", entero(col_lsp + largo as i64)),
                    ]),
                ),
            ]),
        ),
    ])
}

// Definition: salta a donde se definió la palabra que tienes
// debajo del ratón. Si no es una variable o función que
// definiste, no devuelve nada.
fn definicion(params: Json, documentos: &HashMap<String, String>) -> Json {
    let (linea, columna) = posicion(&params);
    let fuente = texto_documento(&params, documentos);
    let palabra = autocompletar::palabra_en(&fuente, linea, columna);
    if palabra.is_empty() {
        return Json::Nulo;
    }
    let uri = params
        .get("textDocument")
        .and_then(|d| d.get("uri"))
        .and_then(|u| u.como_texto())
        .unwrap_or("");
    // La definición es el primer lugar donde aparece la
    // palabra en su línea de definición.
    for simbolo in autocompletar::simbolos(&fuente) {
        if simbolo.nombre == palabra {
            let lugares = autocompletar::ocurrencias(&fuente, &palabra);
            if let Some((l, c)) = lugares.iter().find(|(l, _)| *l == simbolo.linea) {
                return ubicacion(uri, *l, *c, palabra.chars().count());
            }
            return ubicacion(uri, simbolo.linea, 1, palabra.chars().count());
        }
    }
    Json::Nulo
}

// References: todos los lugares donde se usa la palabra
// que tienes debajo del ratón (su definición y cada uso).
fn referencias(params: Json, documentos: &HashMap<String, String>) -> Json {
    let (linea, columna) = posicion(&params);
    let fuente = texto_documento(&params, documentos);
    let palabra = autocompletar::palabra_en(&fuente, linea, columna);
    if palabra.is_empty() {
        return arreglo(Vec::new());
    }
    let uri = params
        .get("textDocument")
        .and_then(|d| d.get("uri"))
        .and_then(|u| u.como_texto())
        .unwrap_or("");
    let largo = palabra.chars().count();
    let lugares = autocompletar::ocurrencias(&fuente, &palabra);
    arreglo(
        lugares
            .iter()
            .map(|(l, c)| ubicacion(uri, *l, *c, largo))
            .collect(),
    )
}

// Extrae (línea, mensaje) de un error del lexer o del
// parser: "Error en la línea N: mensaje". Si el error no
// trae línea, se pone en la línea 1.
fn linea_y_mensaje(error: &str) -> (usize, String) {
    if let Some(resto) = error.strip_prefix("Error en la línea ") {
        if let Some((num, mensaje)) = resto.split_once(": ") {
            if let Ok(linea) = num.parse::<usize>() {
                return (linea, mensaje.to_string());
            }
        }
    }
    (1, error.to_string())
}

// Revisa un archivo y devuelve el aviso de diagnósticos
// (textDocument/publishDiagnostics): sus errores, con la
// línea y el mensaje en español. Si no tiene errores, el
// aviso lleva la lista vacía (para borrar los de antes).
// Solo se muestra el primer error: el lexer y el parser
// se paran en el primero que encuentran.
fn diagnosticar(uri: &str, texto_doc: &str) -> Json {
    let mut diagnosticos: Vec<Json> = Vec::new();
    // Primero el lexer y el parser (errores de sintaxis).
    // Si no hay ninguno, el verificador de préstamos revisa
    // el código y te regaña antes de correr nada.
    let error: Option<(usize, String)> = match Lexer::nuevo(texto_doc).tokenizar() {
        Ok(tokens) => match Parser::nuevo(tokens).parsear() {
            Ok(declaraciones) => {
                // La carpeta del archivo (para los `importar`):
                // el uri es file:///ruta/al/archivo.ec
                let base = uri
                    .strip_prefix("file://")
                    .map(|ruta| {
                        std::path::Path::new(ruta)
                            .parent()
                            .map(|p| p.to_path_buf())
                            .unwrap_or_else(|| std::path::PathBuf::from("."))
                    })
                    .unwrap_or_else(|| std::path::PathBuf::from("."));
                match crate::motor::verificador::verificar(&declaraciones, &base) {
                    Ok(()) => None,
                    Err(e) => Some((e.linea, e.mensaje)),
                }
            }
            Err(e) => Some(linea_y_mensaje(&e)),
        },
        Err(e) => Some(linea_y_mensaje(&e)),
    };
    if let Some((linea, mensaje)) = error {
        // La línea es base 1 en Eclipse y base 0 en LSP.
        // El aviso cubre la línea entera.
        let linea_lsp = (linea.saturating_sub(1)) as i64;
        let largo = texto_doc
            .lines()
            .nth(linea.saturating_sub(1))
            .map(|l| l.chars().count() as i64)
            .unwrap_or(0);
        diagnosticos.push(objeto(vec![
            (
                "range",
                objeto(vec![
                    (
                        "start",
                        objeto(vec![("line", entero(linea_lsp)), ("character", entero(0))]),
                    ),
                    (
                        "end",
                        objeto(vec![("line", entero(linea_lsp)), ("character", entero(largo))]),
                    ),
                ]),
            ),
            ("severity", entero(1)),
            ("message", texto(&mensaje)),
            ("source", texto("Eclipse")),
        ]));
    }
    objeto(vec![
        ("jsonrpc", texto("2.0")),
        ("method", texto("textDocument/publishDiagnostics")),
        (
            "params",
            objeto(vec![
                ("uri", texto(uri)),
                ("diagnostics", arreglo(diagnosticos)),
            ]),
        ),
    ])
}
