// ============================================================
//  ECLIPSE 1.2 - El programa principal
//  Archivo: src/main.rs
//
//  Qué hace:
//  Es la puerta de entrada de Eclipse. Cuando escribes
//  `cargo run -- programa.ec`, este archivo:
//      1. Lee el archivo .ec
//      2. Pasa el texto por el lexer (lo corta en tokens)
//      3. Pasa los tokens por el parser (arma el arbolito)
//      4. Pasa el arbolito por el compilador (lo traduce
//      a bytecode) y la máquina virtual lo corre.
//      La máquina virtual es EL motor de Eclipse: todo
//      corre con ella, directo y rápido.
//      O lo traduce a Rust y lo compila a .exe
//      (con --exe), o lo depura paso a paso
//      (con --depurar o --trazar)
//
//  Si no le das ningún archivo, abre el modo interactivo:
//  escribes código línea por línea y Eclipse te responde
//  en el momento. Escribe 'salir' para terminar.
//
//  Con --completar te dice qué puedes escribir en una
//  posición (eso es lo que usa el autocompletado de VS Code).
//  Con --lsp arranca el servidor de lenguaje (LSP) en Rust.
//
//  Si algo sale mal en cualquier paso, te dice el error
//  en español, con la línea donde pasó.
// ============================================================

mod herramientas;
mod lenguaje;
mod lsp;
mod motor;

use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process;

// Corre un programa Eclipse con la máquina virtual:
// lexer -> parser -> verificador -> compilador -> bytecode -> VM.
// La VM es EL motor de Eclipse: todo corre con ella.
// Si `mostrar` es verdadero, primero enseña el bytecode.
fn correr(fuente: &str, base: PathBuf, nombre: &str, mostrar: bool) -> Result<(), String> {
    let programa = motor::compilador::compilar(fuente, base, nombre)?;
    if mostrar {
        programa.mostrar();
        println!();
    }
    let mut maquina = motor::vm::MaquinaVirtual::nueva(programa);
    maquina.correr()
}

// Lee un archivo .ec, o explica en español por qué no pudo.
fn leer_archivo(ruta: &str) -> String {
    match fs::read_to_string(ruta) {
        Ok(f) => f,
        Err(e) => {
            use std::io::ErrorKind;
            match e.kind() {
                ErrorKind::NotFound => eprintln!("No encontré el archivo '{}'", ruta),
                ErrorKind::InvalidData => {
                    eprintln!("El archivo '{}' no es texto válido", ruta)
                }
                _ => eprintln!("No pude leer el archivo '{}'", ruta),
            }
            process::exit(1);
        }
    }
}

// Genera el código Rust de un programa y, si se pide,
// lo compila a un .exe con rustc. Devuelve la ruta del .exe
// (o del código, si solo se generó el Rust).
fn generar_exe(ruta: &str, solo_rust: bool) -> Result<PathBuf, String> {
    let fuente = leer_archivo(ruta);
    let base = Path::new(ruta)
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| Path::new(".").to_path_buf());
    let codigo = motor::transpilador::transpile(&fuente, base, ruta)?;
    // El código Rust va en <programa>_eclipse/ junto al .ec,
    // para que puedas leerlo y aprender Rust con él.
    let ruta_path = Path::new(ruta);
    let tallo = ruta_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("programa")
        .to_string();
    let dir_ec = ruta_path
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| Path::new(".").to_path_buf());
    let dir_ec_abs = if dir_ec.is_absolute() {
        dir_ec
    } else {
        env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(dir_ec)
    };
    let dir_salida = dir_ec_abs.join(format!("{}_eclipse", tallo));
    fs::create_dir_all(&dir_salida)
        .map_err(|_| format!("no pude crear la carpeta '{}'", dir_salida.display()))?;
    fs::write(dir_salida.join("main.rs"), &codigo)
        .map_err(|_| "no pude escribir el main.rs".to_string())?;
    fs::write(dir_salida.join("eclipse_rt.rs"), motor::transpilador::RUNTIME_FUENTE)
        .map_err(|_| "no pude escribir el eclipse_rt.rs".to_string())?;
    println!("Generé el código Rust en: {}", dir_salida.display());
    if solo_rust {
        return Ok(dir_salida);
    }
    // Compilar con rustc a código nativo (.exe en Windows).
    let exe_nombre = if cfg!(windows) {
        format!("{}.exe", tallo)
    } else {
        tallo.clone()
    };
    let exe_ruta = dir_ec_abs.join(&exe_nombre);
    let salida = process::Command::new("rustc")
        .args(["-O", "--edition", "2021", "main.rs", "-o"])
        .arg(&exe_ruta)
        .current_dir(&dir_salida)
        .output();
    match salida {
        Ok(o) if o.status.success() => Ok(exe_ruta),
        Ok(o) => Err(format!(
            "rustc no pudo compilar el programa:\n{}",
            String::from_utf8_lossy(&o.stderr)
        )),
        Err(_) => Err(
            "No encontré rustc. Instala Rust en https://rustup.rs y vuelve a intentarlo."
                .to_string(),
        ),
    }
}

fn main() {
    // Si este ejecutable trae un paquete pegado al final
    // (lo creó `eclipse empaquetar`), corre ESE programa
    // en la máquina virtual y no hace nada más: es un
    // programa standalone, no necesita Rust ni Cargo.
    if let Some(programa) = motor::paquete::paquete_incrustado() {
        let mut maquina = motor::vm::MaquinaVirtual::nueva(programa);
        if let Err(e) = maquina.correr() {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
        return;
    }
    let args: Vec<String> = env::args().collect();
    // El servidor LSP habla por la entrada y salida estándar:
    // no se le puede poner el banner, porque lo rompería.
    if args.get(1).map(|a| a.as_str()) == Some("--lsp") {
        lsp::servidor::servidor();
        return;
    }
    println!("Eclipse 1.2 - tu lenguaje en español");

    // Sin archivo: modo interactivo.
    if args.len() == 1 {
        repl();
        return;
    }

    match args[1].as_str() {
        "--ayuda" | "-h" | "ayuda" => {
            println!("Uso:");
            println!("  eclipse programa.ec            Corre un archivo en la máquina virtual");
            println!("  eclipse --vm programa.ec       Lo mismo que sin --vm (la VM ya es el modo normal)");
            println!("  eclipse --bytecode programa.ec Muestra el bytecode y lo corre en la máquina virtual");
            println!("  eclipse --exe programa.ec      Lo compila a un .exe nativo (necesita rustc)");
            println!("  eclipse --rust programa.ec     Solo genera el código Rust (main.rs)");
            println!("  eclipse empaquetar programa.ec Crea un ejecutable solo, sin necesitar Rust ni Cargo");
            println!("  eclipse --depurar programa.ec  Corre el programa paso a paso (depurador)");
            println!("  eclipse --trazar programa.ec   Corre el programa mostrando cada paso");
            println!("  eclipse --lsp                  Arranca el servidor de lenguaje (LSP)");
            println!("  eclipse --completar archivo.ec linea columna");
            println!("                                 Dice qué puedes escribir en esa posición");
            println!("  eclipse nuevo mi_proyecto      Crea un proyecto nuevo");
            println!("  eclipse construir [--release]  Compila todos los .ec del proyecto (con --release empaqueta)");
            println!("  eclipse correr [archivo]       Corre un archivo o el principal del proyecto");
            println!("  eclipse correr-todos [carpeta] Corre TODOS los .ec de la carpeta, uno por uno");
            println!("  eclipse instalar               Instala Eclipse y su extensión de VS Code");
            println!("  eclipse biblioteca publicar a.ec Publica una librería (revisa que esté en español y sea segura)");
            println!("  eclipse biblioteca listar      Muestra las librerías instaladas");
            println!("  eclipse biblioteca info nombre Muestra los datos de una librería");
            println!("  eclipse biblioteca quitar nombre Quita una librería instalada");
            println!("  eclipse biblioteca estado nombre Muestra si una librería está disponible o en revisión");
            println!("  eclipse biblioteca revisar     Aprueba las librerías que ya cumplieron su revisión");
            println!("  eclipse desinstalar            Quita Eclipse y su extensión");
            println!("  eclipse                        Abre el modo interactivo");
            println!("  eclipse --version              Muestra la versión de Eclipse");
            println!("  eclipse --ayuda                Muestra esta ayuda");
        }
        "--version" | "-v" | "version" => {
            println!("Eclipse 1.2");
        }
        "instalar" => {
            if let Err(e) = herramientas::instalar::instalar() {
                eprintln!("Error: {}", e);
                process::exit(1);
            }
        }
        "desinstalar" => {
            if let Err(e) = herramientas::instalar::desinstalar() {
                eprintln!("Error: {}", e);
                process::exit(1);
            }
        }
        "biblioteca" => {
            use herramientas::bibliotecas as bib;
            match args.get(2).map(|a| a.as_str()) {
                Some("publicar") => match args.get(3) {
                    Some(archivo) => match bib::publicar(archivo) {
                        Ok(mensaje) => println!("{}", mensaje),
                        Err(e) => {
                            eprintln!("Error: {}", e);
                            process::exit(1);
                        }
                    },
                    None => {
                        eprintln!("Error: falta el archivo. Uso: eclipse biblioteca publicar mi_libreria.ec");
                        process::exit(1);
                    }
                },
                Some("listar") => match bib::listar() {
                    Ok(lista) => {
                        if lista.is_empty() {
                            println!("No tienes librerías instaladas. Publica una con: eclipse biblioteca publicar mi_libreria.ec");
                        } else {
                            for (nombre, version, estado) in lista {
                                println!("{} (v{}) - {}", nombre, version, estado);
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Error: {}", e);
                        process::exit(1);
                    }
                },
                Some("info") => match args.get(3) {
                    Some(nombre) => match bib::info(nombre) {
                        Ok(texto) => println!("{}", texto),
                        Err(e) => {
                            eprintln!("Error: {}", e);
                            process::exit(1);
                        }
                    },
                    None => {
                        eprintln!("Error: falta el nombre. Uso: eclipse biblioteca info matematicas");
                        process::exit(1);
                    }
                },
                Some("quitar") => match args.get(3) {
                    Some(nombre) => match bib::quitar(nombre) {
                        Ok(mensaje) => println!("{}", mensaje),
                        Err(e) => {
                            eprintln!("Error: {}", e);
                            process::exit(1);
                        }
                    },
                    None => {
                        eprintln!("Error: falta el nombre. Uso: eclipse biblioteca quitar matematicas");
                        process::exit(1);
                    }
                },
                Some("estado") => match args.get(3) {
                    Some(nombre) => match bib::estado(nombre) {
                        Ok(mensaje) => println!("{}", mensaje),
                        Err(e) => {
                            eprintln!("Error: {}", e);
                            process::exit(1);
                        }
                    },
                    None => {
                        eprintln!("Error: falta el nombre. Uso: eclipse biblioteca estado matematicas");
                        process::exit(1);
                    }
                },
                Some("revisar") => {
                    let aprobadas = bib::revisar_cola();
                    if aprobadas.is_empty() {
                        println!("No hay librerías listas para revisar todavía.");
                    } else {
                        for nombre in aprobadas {
                            println!("La librería '{}' ya está disponible. Se usa con: usar {};", nombre, nombre);
                        }
                    }
                }
                _ => {
                    eprintln!("Uso: eclipse biblioteca <publicar|listar|info|quitar|estado|revisar>");
                    process::exit(1);
                }
            }
        }
        "nuevo" => {
            let nombre = match args.get(2) {
                Some(n) => n.clone(),
                None => {
                    eprintln!("Te falta el nombre: escribe eclipse nuevo mi_proyecto");
                    process::exit(1);
                }
            };
            match herramientas::proyecto::nuevo(&nombre) {
                Ok(carpeta) => {
                    println!("Proyecto '{}' creado en: {}", nombre, carpeta.display());
                    println!("Entra con: cd {}", nombre);
                    println!("Y corre con: eclipse correr");
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    process::exit(1);
                }
            }
        }
        "construir" => {
            let release = args.iter().any(|a| a == "--release");
            let carpeta = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
            let proyecto = match herramientas::proyecto::buscar_proyecto(&carpeta) {
                Ok(p) => p,
                Err(e) => {
                    eprintln!("Error: {}", e);
                    process::exit(1);
                }
            };
            if let Err(e) = herramientas::proyecto::construir(&proyecto, release) {
                eprintln!("Error: {}", e);
                process::exit(1);
            }
        }
        "correr" => {
            let ruta = match args.get(2).filter(|a| !a.starts_with("--")) {
                Some(r) => r.clone(),
                None => {
                    // Sin archivo: correr el principal del proyecto.
                    let carpeta = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
                    match herramientas::proyecto::buscar_proyecto(&carpeta) {
                        Ok(p) => p.carpeta.join(&p.principal).display().to_string(),
                        Err(e) => {
                            eprintln!("Error: {}", e);
                            process::exit(1);
                        }
                    }
                }
            };
            let fuente = leer_archivo(&ruta);
            let base = Path::new(&ruta)
                .parent()
                .map(|p| p.to_path_buf())
                .unwrap_or_else(|| Path::new(".").to_path_buf());
            if let Err(e) = correr(&fuente, base, &ruta, false) {
                eprintln!("Error: {}", e);
                process::exit(1);
            }
        }
        "correr-todos" => {
            let carpeta = match args.get(2) {
                Some(c) => PathBuf::from(c),
                None => env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            };
            if let Err(e) = herramientas::proyecto::correr_todos(&carpeta) {
                eprintln!("Error: {}", e);
                process::exit(1);
            }
        }
        "empaquetar" => {
            let ruta = match args.get(2) {
                Some(r) => r.clone(),
                None => {
                    eprintln!("Te falta el archivo: escribe eclipse empaquetar programa.ec");
                    process::exit(1);
                }
            };
            let salida = args.get(3).map(|s| s.as_str());
            match motor::paquete::empaquetar(&ruta, salida) {
                Ok(salida) => {
                    println!("¡Listo! Tu programa standalone está en: {}", salida.display());
                    println!("Para correrlo, escribe: {}", salida.display());
                    println!("No necesita Rust ni Cargo: solo cópialo y corre.");
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    process::exit(1);
                }
            }
        }
        "--exe" | "--rust" => {
            let solo_rust = args[1] == "--rust";
            let ruta = match args.get(2) {
                Some(r) => r.clone(),
                None => {
                    eprintln!("Te falta el archivo: escribe eclipse --exe programa.ec");
                    process::exit(1);
                }
            };
            match generar_exe(&ruta, solo_rust) {
                Ok(salida) => {
                    if solo_rust {
                        println!("El código Rust está en: {}", salida.display());
                    } else {
                        println!("¡Listo! Tu programa está en: {}", salida.display());
                        println!("Para correrlo, escribe: {}", salida.display());
                    }
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    process::exit(1);
                }
            }
        }
        "--depurar" | "--trazar" => {
            let ruta = match args.get(2) {
                Some(r) => r.clone(),
                None => {
                    eprintln!("Te falta el archivo: escribe eclipse --depurar programa.ec");
                    process::exit(1);
                }
            };
            let fuente = leer_archivo(&ruta);
            let base = Path::new(&ruta)
                .parent()
                .map(|p| p.to_path_buf())
                .unwrap_or_else(|| Path::new(".").to_path_buf());
            let programa = match motor::compilador::compilar(&fuente, base, &ruta) {
                Ok(p) => p,
                Err(e) => {
                    eprintln!("Error: {}", e);
                    process::exit(1);
                }
            };
            let mut maquina = motor::vm::MaquinaVirtual::nueva(programa);
            if args[1] == "--depurar" {
                maquina.depurar = true;
            } else {
                maquina.trazar = true;
            }
            if let Err(e) = maquina.correr() {
                eprintln!("Error: {}", e);
                process::exit(1);
            }
            maquina.resumen_depurador();
        }
        "--completar" => {
            let ruta = match args.get(2) {
                Some(r) => r.clone(),
                None => {
                    eprintln!("Te falta el archivo: eclipse --completar programa.ec linea columna");
                    process::exit(1);
                }
            };
            let linea: usize = match args.get(3).and_then(|s| s.parse().ok()) {
                Some(n) => n,
                None => {
                    eprintln!("Te falta la línea: eclipse --completar programa.ec linea columna");
                    process::exit(1);
                }
            };
            let columna: usize = match args.get(4).and_then(|s| s.parse().ok()) {
                Some(n) => n,
                None => {
                    eprintln!("Te falta la columna: eclipse --completar programa.ec linea columna");
                    process::exit(1);
                }
            };
            let fuente = leer_archivo(&ruta);
            let completados = herramientas::autocompletar::completar(&fuente, linea, columna);
            for c in completados {
                println!("{}\t{}\t{}", c.nombre, c.tipo, c.descripcion);
            }
        }
        "--vm" | "--bytecode" => {
            let mostrar = args[1] == "--bytecode";
            let ruta = match args.get(2) {
                Some(r) => r.clone(),
                None => {
                    eprintln!("Te falta el archivo: escribe eclipse --vm programa.ec");
                    process::exit(1);
                }
            };
            let fuente = leer_archivo(&ruta);
            let base = Path::new(&ruta)
                .parent()
                .map(|p| p.to_path_buf())
                .unwrap_or_else(|| Path::new(".").to_path_buf());
            if let Err(e) = correr(&fuente, base, &ruta, mostrar) {
                eprintln!("Error: {}", e);
                process::exit(1);
            }
        }
        ruta => {
            let fuente = leer_archivo(ruta);
            // Los `importar` se buscan junto al archivo principal.
            let base = Path::new(ruta)
                .parent()
                .map(|p| p.to_path_buf())
                .unwrap_or_else(|| Path::new(".").to_path_buf());
            if let Err(e) = correr(&fuente, base, ruta, false) {
                eprintln!("Error: {}", e);
                process::exit(1);
            }
        }
    }
}

// El modo interactivo: escribes código y Eclipse responde.
// Si abres una llave { y no la cierras, sigue leyendo líneas
// hasta que las llaves estén parejas. Cada línea se compila
// y se corre en la misma máquina virtual, así que las
// variables y funciones de antes siguen vivas.
fn repl() {
    println!("Escribe código Eclipse. Escribe 'salir' para terminar.");
    let base = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let mut compilador = motor::compilador::Compilador::nuevo(base);
    let mut maquina = motor::vm::MaquinaVirtual::nueva_vacia();
    loop {
        print!("eclipse> ");
        let _ = io::stdout().flush();
        let mut codigo = String::new();
        loop {
            let mut linea = String::new();
            match io::stdin().read_line(&mut linea) {
                Ok(0) => return,
                Ok(_) => {}
                Err(_) => return,
            }
            codigo.push_str(&linea);
            let abiertas = codigo.matches('{').count();
            let cerradas = codigo.matches('}').count();
            if abiertas <= cerradas {
                break;
            }
            print!("... ");
            let _ = io::stdout().flush();
        }
        let linea = codigo.trim();
        if linea == "salir" {
            println!("¡Hasta luego!");
            break;
        }
        if linea.is_empty() {
            continue;
        }
        // Lexer y parser de la línea (sin verificador: cada
        // línea usa variables de las líneas anteriores).
        let tokens = match lenguaje::lexer::Lexer::nuevo(linea).tokenizar() {
            Ok(t) => t,
            Err(e) => {
                eprintln!("Error: {}", e);
                continue;
            }
        };
        let declaraciones = match lenguaje::parser::Parser::nuevo(tokens).parsear() {
            Ok(d) => d,
            Err(e) => {
                eprintln!("Error: {}", e);
                continue;
            }
        };
        match compilador.compilar_fragmento(&declaraciones) {
            Ok(pedazo) => {
                maquina.agregar_funciones(compilador.funciones());
                if let Err(e) = maquina.ejecutar_pedazo(pedazo) {
                    eprintln!("Error: {}", e);
                }
            }
            Err(e) => eprintln!("Error: {}", e),
        }
    }
}
