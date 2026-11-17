// ============================================================
//  ECLIPSE 1.2 - El probador (todo en Rust)
//  Archivo: src/bin/probar.rs
//
//  Qué hace:
//  Compara la máquina virtual, el paquete standalone y el
//  .exe nativo en todos los ejemplos y en todos los casos
//  difíciles: los tres tienen que dar exactamente lo mismo
//  (misma salida, mismo error y mismo código de salida).
//  La VM es EL motor de Eclipse (el intérprete ya no existe).
//  También prueba los proyectos (construir y correr-todos)
//  y la extensión de VS Code.
//
//  Se corre con:
//      cargo run --bin probar
//  o, con el binario ya compilado:
//      ./target/debug/probar [carpeta...]
//
//  Si no le das carpetas, prueba `ejemplos` y `pruebas05`.
//  Cada archivo que falla se marca con FALLA y al final
//  dice cuántos pasaron. Si algo falla, sale con código 1.
//
//  Solo usa la librería estándar de Rust.
// ============================================================

use std::env;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

// Cuánto se espera a que un programa termine (10 segundos).
const TIEMPO_MAX: Duration = Duration::from_secs(10);

// Lo que devuelve correr un programa: su salida, su error
// y si terminó bien (código 0).
#[derive(Debug, Clone, PartialEq)]
struct Resultado {
    salida: String,
    error: String,
    bien: bool,
}

// La entrada que se le da a un archivo por teclado, si la
// necesita. Sin entrada, se le cierra el teclado.
fn entrada_para(ruta: &Path) -> &'static str {
    let nombre = ruta.file_name().and_then(|n| n.to_str()).unwrap_or("");
    match nombre {
        "calculadora.ec" => "10\n5\n",
        "leer_input.ec" => "Kevin\n13\n",
        "leer_lista.ec" => "1, 2, 3\nKevin\n",
        _ => "",
    }
}

// Corre un comando con entrada por teclado y un tiempo
// máximo. Si se pasa del tiempo, lo mata y lo marca mal.
fn correr(cmd: &mut Command, entrada: &str) -> Resultado {
    let mut hijo = match cmd
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(h) => h,
        Err(e) => {
            return Resultado {
                salida: String::new(),
                error: format!("no pude correr el comando: {}", e),
                bien: false,
            }
        }
    };
    if let Some(mut teclado) = hijo.stdin.take() {
        let _ = teclado.write_all(entrada.as_bytes());
    }
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let _ = tx.send(hijo.wait_with_output());
    });
    match rx.recv_timeout(TIEMPO_MAX) {
        Ok(Ok(salida)) => Resultado {
            salida: String::from_utf8_lossy(&salida.stdout).to_string(),
            error: String::from_utf8_lossy(&salida.stderr).to_string(),
            bien: salida.status.success(),
        },
        Ok(Err(e)) => Resultado {
            salida: String::new(),
            error: format!("no pude leer la salida: {}", e),
            bien: false,
        },
        Err(_) => Resultado {
            salida: String::new(),
            error: "se pasó de 10 segundos, ¿tiene un bucle sin fin?".to_string(),
            bien: false,
        },
    }
}

// Busca el programa eclipse: primero la variable ECLIPSE_BIN,
// luego el target de este proyecto, y si no, el PATH.
fn eclipse_bin() -> PathBuf {
    if let Ok(ruta) = env::var("ECLIPSE_BIN") {
        return PathBuf::from(ruta);
    }
    for candidato in ["target/debug/eclipse", "target/release/eclipse"] {
        let ruta = PathBuf::from(candidato);
        if ruta.exists() {
            // Ruta absoluta: los proyectos se prueban en /tmp
            // y una ruta relativa no se encontraría desde ahí.
            if let Ok(abs) = ruta.canonicalize() {
                return abs;
            }
            return ruta;
        }
    }
    PathBuf::from("eclipse")
}

// Junta todos los .ec de una carpeta (y sus subcarpetas).
fn listar_ec(carpeta: &Path) -> Vec<PathBuf> {
    let mut archivos = Vec::new();
    listar_ec_rec(carpeta, &mut archivos);
    archivos.sort();
    archivos
}

fn listar_ec_rec(carpeta: &Path, archivos: &mut Vec<PathBuf>) {
    let entradas = match fs::read_dir(carpeta) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entrada in entradas.flatten() {
        let ruta = entrada.path();
        if ruta.is_dir() {
            listar_ec_rec(&ruta, archivos);
        } else if ruta.extension().map(|e| e == "ec").unwrap_or(false) {
            archivos.push(ruta);
        }
    }
}

// Quita la línea del banner ("Eclipse 1.2 - ...") del
// principio, sin tocar nada más (ni el salto final).
fn sin_banner(salida: &str) -> String {
    if salida.starts_with("Eclipse ") {
        match salida.find('\n') {
            Some(i) => salida[i + 1..].to_string(),
            None => String::new(),
        }
    } else {
        salida.to_string()
    }
}

// Prueba un archivo .ec en la máquina virtual, el paquete
// standalone y el .exe nativo. Devuelve (pasó, mensaje).
// La VM es EL motor de Eclipse (el intérprete ya no existe):
// el paquete corre en la VM y el .exe es código Rust generado,
// así que los tres tienen que dar exactamente lo mismo.
fn probar_archivo(eclipse: &Path, ruta: &Path, con_exe: bool) -> (bool, String) {
    let entrada = entrada_para(ruta);
    let nombre = ruta.display().to_string();

    // 1. La máquina virtual (el modo normal).
    let vm = correr(Command::new(eclipse).arg(ruta), entrada);
    // Quitar el banner (la primera línea es el banner).
    let salida_vm = sin_banner(&vm.salida);

    // 2. El paquete standalone (sin rustc).
    let paquete_ruta = ruta.with_extension("");
    let paquete_ruta = if cfg!(windows) {
        paquete_ruta.with_extension("exe")
    } else {
        paquete_ruta
    };
    let _ = correr(
        Command::new(eclipse)
            .arg("empaquetar")
            .arg(ruta)
            .arg(&paquete_ruta),
        "",
    );
    if paquete_ruta.exists() {
        let paquete = correr(&mut Command::new(&paquete_ruta), entrada);
        let salida_paq = sin_banner(&paquete.salida);
        // El paquete no imprime el banner (es el programa solo).
        if paquete.salida != salida_vm || paquete.bien != vm.bien || paquete.error != vm.error {
            let _ = fs::remove_file(&paquete_ruta);
            return (
                false,
                format!(
                    "{}: el paquete no dio lo mismo que la VM\n  VM: {:?}\n  paquete: {:?}",
                    nombre, salida_vm, salida_paq
                ),
            );
        }
        let _ = fs::remove_file(&paquete_ruta);
    }

    // 3. El .exe nativo (solo si hay rustc).
    if con_exe {
        let exe = correr(Command::new(eclipse).arg("--exe").arg(ruta), "");
        if !exe.bien {
            // Si la VM falló con el mismo error, los motores
            // están de acuerdo: no es una falla.
            if !vm.bien && exe.error == vm.error {
                return (true, String::new());
            }
            return (
                false,
                format!("{}: --exe falló: {}", nombre, exe.error),
            );
        }
        let exe_ruta = ruta.with_extension("");
        let exe_ruta = if cfg!(windows) {
            exe_ruta.with_extension("exe")
        } else {
            exe_ruta
        };
        if exe_ruta.exists() {
            let nativo = correr(&mut Command::new(&exe_ruta), entrada);
            if nativo.salida != salida_vm || nativo.bien != vm.bien || nativo.error != vm.error {
                let _ = fs::remove_file(&exe_ruta);
                return (
                    false,
                    format!(
                        "{}: el .exe no dio lo mismo que la VM\n  VM: {:?}\n  .exe: {:?}",
                        nombre, salida_vm, nativo.salida
                    ),
                );
            }
            let _ = fs::remove_file(&exe_ruta);
        }
        // Limpiar el código Rust generado.
        let tallo = ruta
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("programa");
        let dir_gen = ruta
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join(format!("{}_eclipse", tallo));
        let _ = fs::remove_dir_all(&dir_gen);
    }

    (true, nombre)
}

// Prueba los proyectos: crear, construir, correr y
// correr-todos en una carpeta temporal.
fn probar_proyectos(eclipse: &Path) -> (bool, String) {
    let tmp = env::temp_dir().join("eclipse_probar_proyecto");
    let _ = fs::remove_dir_all(&tmp);
    let _ = fs::create_dir_all(&tmp);
    let crear = correr(
        Command::new(eclipse)
            .arg("nuevo")
            .arg("mi_juego")
            .current_dir(&tmp),
        "",
    );
    if !crear.bien {
        return (false, format!("nuevo falló: {}", crear.error));
    }
    let carpeta = tmp.join("mi_juego");
    let construir = correr(
        Command::new(eclipse)
            .arg("construir")
            .current_dir(&carpeta),
        "",
    );
    if !construir.bien {
        return (false, format!("construir falló: {}", construir.error));
    }
    let correr_p = correr(
        Command::new(eclipse).arg("correr").current_dir(&carpeta),
        "",
    );
    if !correr_p.bien || !sin_banner(&correr_p.salida).contains("¡Hola, mi_juego!") {
        return (
            false,
            format!("correr falló: {:?} {:?}", correr_p.salida, correr_p.error),
        );
    }
    let todos = correr(
        Command::new(eclipse)
            .arg("correr-todos")
            .arg("src")
            .current_dir(&carpeta),
        "",
    );
    if !todos.bien {
        return (false, format!("correr-todos falló: {}", todos.error));
    }
    let _ = fs::remove_dir_all(&tmp);
    (true, "proyectos (nuevo, construir, correr, correr-todos)".to_string())
}

// Revisa la extensión de VS Code: que el package.json y
// el extension.js tengan los 3 comandos Run y el LSP.
fn probar_extension() -> (bool, String) {
    let mut faltan = Vec::new();
    let package = fs::read_to_string("vscode-eclipse/package.json").unwrap_or_default();
    for texto in [
        "eclipse.ejecutar",
        "eclipse.ejecutarVm",
        "eclipse.ejecutarDepurador",
        "ctrl+alt+r",
        "ctrl+alt+v",
        "ctrl+alt+d",
    ] {
        if !package.contains(texto) {
            faltan.push(format!("package.json: falta '{}'", texto));
        }
    }
    let extension = fs::read_to_string("vscode-eclipse/extension.js").unwrap_or_default();
    for texto in [
        "eclipse.ejecutar",
        "eclipse.ejecutarVm",
        "eclipse.ejecutarDepurador",
        "--vm",
        "--depurar",
    ] {
        if !extension.contains(texto) {
            faltan.push(format!("extension.js: falta '{}'", texto));
        }
    }
    if faltan.is_empty() {
        (true, "extensión de VS Code (3 comandos Run)".to_string())
    } else {
        (false, faltan.join("\n"))
    }
}

// Prueba el sistema de librerías con una casa temporal
// (para no tocar el Registro de verdad ni el límite de 5 horas):
// publicar con las 6 preguntas, revisión de 1-2 días, límite
// de 5 horas, aprobar y usar.
fn probar_bibliotecas(eclipse: &Path) -> (bool, String) {
    let casa = env::temp_dir().join("eclipse_prueba_bib");
    let _ = fs::remove_dir_all(&casa);
    if fs::create_dir_all(&casa).is_err() {
        return (false, "bibliotecas: no pude crear la casa temporal".to_string());
    }
    let lib = casa.join("doble.ec");
    if fs::write(&lib, "funcion doble(n) {\n    devolver n * 2;\n}\n").is_err() {
        return (false, "bibliotecas: no pude escribir la librería de prueba".to_string());
    }
    let prog = casa.join("usa.ec");
    if fs::write(&prog, "usar prueba_doble;\nimprimir(doble(21));\n").is_err() {
        return (false, "bibliotecas: no pude escribir el programa de prueba".to_string());
    }
    let cmd = |args: &[&str]| {
        let mut c = Command::new(eclipse);
        c.env("HOME", &casa);
        for a in args {
            c.arg(a);
        }
        c
    };
    let respuestas = "prueba_doble\nKevin\nSirve para doblar\n1.0.0\nmatematicas\n\n";
    // 1. Publicar: tiene que quedar en revisión.
    let pub1 = correr(
        &mut cmd(&["biblioteca", "publicar", lib.to_str().unwrap()]),
        respuestas,
    );
    if !pub1.bien || !pub1.salida.contains("enviada a revisión") {
        return (false, format!("bibliotecas: publicar falló: {:?}", pub1));
    }
    // 2. Listar: tiene que decir "en revisión".
    let lista = correr(&mut cmd(&["biblioteca", "listar"]), "");
    if !lista.salida.contains("en revisión") {
        return (
            false,
            format!("bibliotecas: listar no muestra la revisión: {:?}", lista.salida),
        );
    }
    // 3. Usar antes de tiempo: tiene que dar error de revisión.
    let antes = correr(&mut cmd(&[prog.to_str().unwrap()]), "");
    if antes.bien || !antes.error.contains("todavía está en revisión") {
        return (
            false,
            format!("bibliotecas: usar antes de tiempo no dio el error: {:?}", antes),
        );
    }
    // 4. Segunda subida: el límite de 5 horas.
    let pub2 = correr(
        &mut cmd(&["biblioteca", "publicar", lib.to_str().unwrap()]),
        respuestas,
    );
    if pub2.bien || !pub2.error.contains("cada 5 horas") {
        return (
            false,
            format!("bibliotecas: el límite de 5 horas no funcionó: {:?}", pub2),
        );
    }
    // 5. Pasar el tiempo: editar listo_en al pasado y revisar.
    let envio = casa.join(".eclipse/registro/cola/prueba_doble/envio.toml");
    let texto = match fs::read_to_string(&envio) {
        Ok(t) => t,
        Err(_) => {
            return (false, "bibliotecas: no encontré el envio.toml".to_string());
        }
    };
    let mut nuevo = String::new();
    for linea in texto.lines() {
        if linea.starts_with("listo_en") {
            nuevo.push_str("listo_en = 1000\n");
        } else {
            nuevo.push_str(linea);
            nuevo.push('\n');
        }
    }
    if fs::write(&envio, nuevo).is_err() {
        return (false, "bibliotecas: no pude pasar el tiempo".to_string());
    }
    let rev = correr(&mut cmd(&["biblioteca", "revisar"]), "");
    if !rev.salida.contains("ya está disponible") {
        return (false, format!("bibliotecas: revisar no aprobó: {:?}", rev.salida));
    }
    // 6. Usar ya aprobada: tiene que imprimir 42.
    let despues = correr(&mut cmd(&[prog.to_str().unwrap()]), "");
    if !despues.bien || sin_banner(&despues.salida).trim() != "42" {
        return (
            false,
            format!("bibliotecas: usar aprobada no dio 42: {:?}", despues),
        );
    }
    // 7. Info: tiene que traer los datos de las 6 preguntas.
    let info = correr(&mut cmd(&["biblioteca", "info", "prueba_doble"]), "");
    if !info.salida.contains("Kevin")
        || !info.salida.contains("Sirve para doblar")
        || !info.salida.contains("matematicas")
    {
        return (false, format!("bibliotecas: info no trae los datos: {:?}", info.salida));
    }
    let _ = fs::remove_dir_all(&casa);
    (
        true,
        "bibliotecas (publicar, revisión 1-2 días, límite 5h, aprobar, usar)".to_string(),
    )
}

fn main() {
    let eclipse = eclipse_bin();
    if !eclipse.exists() && env::var("ECLIPSE_BIN").is_err() {
        eprintln!("No encontré el programa eclipse. Compílalo primero con: cargo build");
        std::process::exit(1);
    }
    // ¿Hay rustc para probar el .exe nativo?
    let con_exe = Command::new("rustc")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false);
    if !con_exe {
        println!("Aviso: no hay rustc, no se prueba el .exe nativo (el paquete sí se prueba).");
    }

    let args: Vec<String> = env::args().skip(1).collect();
    let carpetas: Vec<String> = if args.is_empty() {
        vec![
            "ejemplos".to_string(),
            "pruebas05".to_string(),
            "pruebas08".to_string(),
            "pruebas09".to_string(),
            "pruebas10".to_string(),
            "pruebas11".to_string(),
            "pruebas12".to_string(),
        ]
    } else {
        args
    };
    let mut total = 0usize;
    let mut fallas = 0usize;
    for carpeta in &carpetas {
        let archivos = listar_ec(Path::new(carpeta));
        if archivos.is_empty() {
            println!("Aviso: no hay .ec en '{}'", carpeta);
            continue;
        }
        for ruta in &archivos {
            total += 1;
            let (paso, mensaje) = probar_archivo(&eclipse, ruta, con_exe);
            if paso {
                println!("OK   [{}]", mensaje);
            } else {
                fallas += 1;
                println!("FALLA {}", mensaje);
            }
        }
    }

    // Proyectos y extensión.
    total += 1;
    let (paso, mensaje) = probar_proyectos(&eclipse);
    if paso {
        println!("OK   [{}]", mensaje);
    } else {
        fallas += 1;
        println!("FALLA {}", mensaje);
    }
    total += 1;
    let (paso, mensaje) = probar_extension();
    if paso {
        println!("OK   [{}]", mensaje);
    } else {
        fallas += 1;
        println!("FALLA {}", mensaje);
    }
    total += 1;
    let (paso, mensaje) = probar_bibliotecas(&eclipse);
    if paso {
        println!("OK   [{}]", mensaje);
    } else {
        fallas += 1;
        println!("FALLA {}", mensaje);
    }

    println!("----------------------------------------");
    println!("Total: {}  Fallas: {}", total, fallas);
    // Limpiar el archivo que deja el ejemplo de archivos.
    let _ = fs::remove_file("saludo.txt");
    let _ = fs::remove_file("ejemplos/04-archivos/saludo.txt");
    if fallas > 0 {
        std::process::exit(1);
    }
}
