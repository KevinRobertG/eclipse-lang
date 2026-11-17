// ============================================================
//  ECLIPSE 1.1 - El probador del LSP (todo en Rust)
//  Archivo: src/bin/probar_lsp.rs
//
//  Qué hace:
//  Habla con el servidor de lenguaje de Eclipse (LSP) por
//  JSON-RPC, como lo haría VS Code, y revisa que responda
//  bien: initialize (con definition y references), abrir un
//  archivo bueno (sin diagnósticos), autocompletado, hover,
//  esquema, definición, referencias, abrir un archivo roto
//  (un diagnóstico en la línea exacta) y apagarse.
//
//  Se corre con:
//      cargo run --bin probar_lsp
//
//  Si todo sale bien, imprime TODO OK y sale con código 0.
//  Si algo falla, dice qué y sale con código 1.
//
//  Solo usa la librería estándar de Rust.
// ============================================================

#[path = "../lsp/json.rs"]
mod json;

use json::{arreglo, booleano, entero, objeto, texto, Json};
use std::env;
use std::io::{Read, Write};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

const TIEMPO_MAX: Duration = Duration::from_secs(5);

fn main() {
    let eclipse = env::var("ECLIPSE_BIN").unwrap_or_else(|_| "target/debug/eclipse".to_string());
    let mut hijo = match Command::new(&eclipse)
        .arg("--lsp")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
    {
        Ok(h) => h,
        Err(e) => {
            eprintln!("No pude arrancar el servidor LSP: {}", e);
            std::process::exit(1);
        }
    };
    let mut cliente = Cliente::nuevo(&mut hijo);
    let mut fallas = 0usize;

    // 1. initialize: tiene que ofrecer definition y references.
    let respuesta = cliente.pedir(1, "initialize", objeto(vec![
        ("processId", Json::Nulo),
        ("rootUri", Json::Nulo),
        (
            "capabilities",
            objeto(vec![]),
        ),
    ]));
    match respuesta {
        Ok(r) => {
            let caps = r.get("result").and_then(|x| x.get("capabilities"));
            let def = caps
                .and_then(|c| c.get("definitionProvider"))
                .and_then(|v| v.como_bool())
                .unwrap_or(false);
            let refs = caps
                .and_then(|c| c.get("referencesProvider"))
                .and_then(|v| v.como_bool())
                .unwrap_or(false);
            if def && refs {
                println!("OK   initialize (definition + references)");
            } else {
                println!("FALLA initialize: definition={} references={}", def, refs);
                fallas += 1;
            }
        }
        Err(e) => {
            println!("FALLA initialize: {}", e);
            fallas += 1;
        }
    }
    cliente.avisar("initialized", objeto(vec![]));

    // El archivo bueno: una variable y una función.
    let bueno = "sea nombre = \"Kevin\";\nsea mut puntos = 10;\nfuncion saludar() {\n    imprimir(nombre);\n}\nimprimir(puntos);\n";
    let uri = "file:///tmp/prueba.ec";
    cliente.avisar(
        "textDocument/didOpen",
        objeto(vec![(
            "textDocument",
            objeto(vec![("uri", texto(uri)), ("text", texto(bueno))]),
        )]),
    );
    // 2. Diagnósticos del archivo bueno: ninguno.
    match cliente.esperar_notificacion("textDocument/publishDiagnostics") {
        Ok(n) => {
            let diags = n
                .get("params")
                .and_then(|p| p.get("diagnostics"))
                .cloned()
                .unwrap_or(Json::Nulo);
            if diags == arreglo(vec![]) {
                println!("OK   diagnósticos del archivo bueno: ninguno");
            } else {
                println!("FALLA diagnósticos del archivo bueno: {:?}", diags);
                fallas += 1;
            }
        }
        Err(e) => {
            println!("FALLA diagnósticos del archivo bueno: {}", e);
            fallas += 1;
        }
    }

    // 3. Autocompletado en la línea 5: tiene que sugerir nombre y puntos.
    let comp = cliente.pedir(
        2,
        "textDocument/completion",
        objeto(vec![
            (
                "textDocument",
                objeto(vec![("uri", texto(uri))]),
            ),
            (
                "position",
                objeto(vec![("line", entero(4)), ("character", entero(9))]),
            ),
        ]),
    );
    match comp {
        Ok(r) => {
            let items = r
                .get("result")
                .and_then(|x| x.get("items"))
                .cloned()
                .unwrap_or(Json::Nulo);
            let etiquetas = etiquetas_de(&items);
            if etiquetas.contains(&"nombre".to_string()) && etiquetas.contains(&"puntos".to_string()) {
                println!("OK   completion: sugiere nombre y puntos");
            } else {
                println!("FALLA completion: {:?}", etiquetas);
                fallas += 1;
            }
        }
        Err(e) => {
            println!("FALLA completion: {}", e);
            fallas += 1;
        }
    }

    // 4. Hover sobre `nombre` en la línea 4: tiene que describirlo.
    let hov = cliente.pedir(
        3,
        "textDocument/hover",
        objeto(vec![
            (
                "textDocument",
                objeto(vec![("uri", texto(uri))]),
            ),
            (
                "position",
                objeto(vec![("line", entero(3)), ("character", entero(13))]),
            ),
        ]),
    );
    match hov {
        Ok(r) => {
            let valor = r
                .get("result")
                .and_then(|x| x.get("contents"))
                .and_then(|c| c.get("value"))
                .and_then(|v| v.como_texto())
                .unwrap_or("")
                .to_string();
            if valor.contains("nombre") {
                println!("OK   hover: describe nombre");
            } else {
                println!("FALLA hover: {:?}", valor);
                fallas += 1;
            }
        }
        Err(e) => {
            println!("FALLA hover: {}", e);
            fallas += 1;
        }
    }

    // 5. Esquema del documento: nombre, puntos y saludar.
    let sim = cliente.pedir(
        4,
        "textDocument/documentSymbol",
        objeto(vec![(
            "textDocument",
            objeto(vec![("uri", texto(uri))]),
        )]),
    );
    match sim {
        Ok(r) => {
            let items = r.get("result").cloned().unwrap_or(Json::Nulo);
            let nombres = nombres_de(&items);
            if nombres.contains(&"nombre".to_string())
                && nombres.contains(&"puntos".to_string())
                && nombres.contains(&"saludar".to_string())
            {
                println!("OK   documentSymbol: nombre, puntos y saludar");
            } else {
                println!("FALLA documentSymbol: {:?}", nombres);
                fallas += 1;
            }
        }
        Err(e) => {
            println!("FALLA documentSymbol: {}", e);
            fallas += 1;
        }
    }

    // 6. Definition: saltar a donde se definió `puntos` (línea 2).
    // `puntos` está en `imprimir(puntos);`, línea 6 (base 1),
    // o sea línea 5 y carácter 10 en base 0.
    let def = cliente.pedir(
        5,
        "textDocument/definition",
        objeto(vec![
            (
                "textDocument",
                objeto(vec![("uri", texto(uri))]),
            ),
            (
                "position",
                objeto(vec![("line", entero(5)), ("character", entero(10))]),
            ),
        ]),
    );
    match def {
        Ok(r) => {
            let linea = r
                .get("result")
                .and_then(|x| x.get("range"))
                .and_then(|x| x.get("start"))
                .and_then(|x| x.get("line"))
                .and_then(|x| x.como_entero())
                .unwrap_or(-1);
            if linea == 1 {
                println!("OK   definition: puntos se definió en la línea 2");
            } else {
                println!("FALLA definition: línea {:?}", linea);
                fallas += 1;
            }
        }
        Err(e) => {
            println!("FALLA definition: {}", e);
            fallas += 1;
        }
    }

    // 7. References: `nombre` se usa en la línea 1 (definición)
    // y en la línea 4 (dentro de saludar): 2 lugares.
    let refs = cliente.pedir(
        6,
        "textDocument/references",
        objeto(vec![
            (
                "textDocument",
                objeto(vec![("uri", texto(uri))]),
            ),
            (
                "position",
                objeto(vec![("line", entero(3)), ("character", entero(13))]),
            ),
            ("context", objeto(vec![("includeDeclaration", booleano(true))])),
        ]),
    );
    match refs {
        Ok(r) => {
            let items = r.get("result").cloned().unwrap_or(Json::Nulo);
            let n = match &items {
                Json::Arreglo(lista) => lista.len(),
                _ => 0,
            };
            if n == 2 {
                println!("OK   references: nombre aparece en 2 lugares");
            } else {
                println!("FALLA references: {} lugares", n);
                fallas += 1;
            }
        }
        Err(e) => {
            println!("FALLA references: {}", e);
            fallas += 1;
        }
    }

    // 8. Archivo roto: `sea puntos =;` en la línea 2.
    let roto = "sea nombre = \"Kevin\";\nsea puntos =;\n";
    cliente.avisar(
        "textDocument/didOpen",
        objeto(vec![(
            "textDocument",
            objeto(vec![("uri", texto("file:///tmp/roto.ec")), ("text", texto(roto))]),
        )]),
    );
    match cliente.esperar_notificacion("textDocument/publishDiagnostics") {
        Ok(n) => {
            let diags = n
                .get("params")
                .and_then(|p| p.get("diagnostics"))
                .cloned()
                .unwrap_or(Json::Nulo);
            let (linea, mensaje) = match &diags {
                Json::Arreglo(lista) if !lista.is_empty() => {
                    let d = &lista[0];
                    let l = d
                        .get("range")
                        .and_then(|x| x.get("start"))
                        .and_then(|x| x.get("line"))
                        .and_then(|x| x.como_entero())
                        .unwrap_or(-1);
                    let m = d
                        .get("message")
                        .and_then(|x| x.como_texto())
                        .unwrap_or("")
                        .to_string();
                    (l, m)
                }
                _ => (-1, String::new()),
            };
            if linea == 1 && mensaje.contains("esperaba un valor") {
                println!("OK   diagnósticos del archivo roto: línea 2, '{}'", mensaje);
            } else {
                println!(
                    "FALLA diagnósticos del archivo roto: línea {} '{}'",
                    linea, mensaje
                );
                fallas += 1;
            }
        }
        Err(e) => {
            println!("FALLA diagnósticos del archivo roto: {}", e);
            fallas += 1;
        }
    }

    // 9. Apagarse.
    let _ = cliente.pedir(7, "shutdown", Json::Nulo);
    cliente.avisar("exit", Json::Nulo);
    let _ = hijo.wait();

    println!("----------------------------------------");
    if fallas == 0 {
        println!("TODO OK");
    } else {
        println!("Fallas: {}", fallas);
        std::process::exit(1);
    }
}

// Las etiquetas de los items de autocompletado.
fn etiquetas_de(items: &Json) -> Vec<String> {
    match items {
        Json::Arreglo(lista) => lista
            .iter()
            .filter_map(|i| i.get("label").and_then(|l| l.como_texto()).map(|s| s.to_string()))
            .collect(),
        _ => Vec::new(),
    }
}

// Los nombres de los símbolos del esquema.
fn nombres_de(items: &Json) -> Vec<String> {
    match items {
        Json::Arreglo(lista) => lista
            .iter()
            .filter_map(|i| i.get("name").and_then(|l| l.como_texto()).map(|s| s.to_string()))
            .collect(),
        _ => Vec::new(),
    }
}

// Un cliente LSP chiquito: escribe mensajes con
// Content-Length y lee las respuestas en otro hilo.
struct Cliente {
    entrada: std::process::ChildStdin,
    rx: mpsc::Receiver<Json>,
}

impl Cliente {
    fn nuevo(hijo: &mut Child) -> Self {
        let entrada = hijo.stdin.take().expect("el hijo no tiene stdin");
        let mut salida = hijo.stdout.take().expect("el hijo no tiene stdout");
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || loop {
            match leer_mensaje(&mut salida) {
                Some(m) => {
                    if tx.send(m).is_err() {
                        return;
                    }
                }
                None => return,
            }
        });
        Cliente { entrada, rx }
    }

    // Escribe un mensaje (petición o notificación).
    fn escribir(&mut self, mensaje: &Json) {
        let cuerpo = mensaje.texto();
        let _ = write!(
            self.entrada,
            "Content-Length: {}\r\n\r\n{}",
            cuerpo.len(),
            cuerpo
        );
        let _ = self.entrada.flush();
    }

    // Una petición con id: espera su respuesta.
    fn pedir(&mut self, id: i64, metodo: &str, params: Json) -> Result<Json, String> {
        self.escribir(&objeto(vec![
            ("jsonrpc", texto("2.0")),
            ("id", entero(id)),
            ("method", texto(metodo)),
            ("params", params),
        ]));
        let inicio = std::time::Instant::now();
        loop {
            let resto = TIEMPO_MAX
                .checked_sub(inicio.elapsed())
                .ok_or_else(|| "se acabó el tiempo esperando la respuesta".to_string())?;
            match self.rx.recv_timeout(resto) {
                Ok(m) => {
                    // Si es la respuesta a esta petición, listo.
                    // Si es una notificación, se ignora y se sigue.
                    if m.get("id").and_then(|i| i.como_entero()) == Some(id) {
                        if m.get("error").is_some() {
                            return Err(format!("el servidor devolvió un error: {:?}", m));
                        }
                        return Ok(m);
                    }
                }
                Err(_) => return Err("se acabó el tiempo esperando la respuesta".to_string()),
            }
        }
    }

    // Una notificación (sin id): no espera respuesta.
    fn avisar(&mut self, metodo: &str, params: Json) {
        self.escribir(&objeto(vec![
            ("jsonrpc", texto("2.0")),
            ("method", texto(metodo)),
            ("params", params),
        ]));
    }

    // Espera una notificación con ese método (por ejemplo,
    // publishDiagnostics después de abrir un archivo).
    fn esperar_notificacion(&mut self, metodo: &str) -> Result<Json, String> {
        let inicio = std::time::Instant::now();
        loop {
            let resto = TIEMPO_MAX
                .checked_sub(inicio.elapsed())
                .ok_or_else(|| "se acabó el tiempo esperando el aviso".to_string())?;
            match self.rx.recv_timeout(resto) {
                Ok(m) => {
                    if m.get("method").and_then(|x| x.como_texto()) == Some(metodo) {
                        return Ok(m);
                    }
                }
                Err(_) => return Err("se acabó el tiempo esperando el aviso".to_string()),
            }
        }
    }
}

// Lee un mensaje con Content-Length de la salida del servidor.
fn leer_mensaje(salida: &mut impl Read) -> Option<Json> {
    let mut longitud: Option<usize> = None;
    let mut byte = [0u8; 1];
    let mut linea = Vec::new();
    loop {
        linea.clear();
        loop {
            match salida.read(&mut byte) {
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
        if linea.last() == Some(&b'\r') {
            linea.pop();
        }
        if linea.is_empty() {
            break;
        }
        let texto = String::from_utf8_lossy(&linea).to_string();
        if let Some(valor) = texto.to_lowercase().strip_prefix("content-length:") {
            longitud = valor.trim().parse::<usize>().ok();
        }
    }
    let n = longitud?;
    let mut cuerpo = vec![0u8; n];
    if salida.read_exact(&mut cuerpo).is_err() {
        return None;
    }
    Json::parsear(&String::from_utf8_lossy(&cuerpo)).ok()
}
