// ============================================================
//  ECLIPSE 1.1 - Los proyectos (como Cargo, pero en español)
//  Archivo: src/herramientas/proyecto.rs
//
//  Qué hace:
//  Un proyecto es una carpeta con un archivo `Eclipse.toml`
//  (el manifiesto: nombre, archivo principal, versión) y
//  una carpeta `src/` con tus archivos .ec. Así Eclipse
//  sabe qué archivos van juntos y en qué orden, y puede:
//      eclipse nuevo mi_juego      Crea el proyecto
//      eclipse construir           Compila todos los .ec
//      eclipse construir --release Además empaqueta el .exe
//      eclipse correr              Corre el principal
//      eclipse correr-todos src    Corre TODOS los .ec, uno
//                                  por uno, y te dice cuáles
//                                  pasaron y cuáles fallaron
//
//  Los `importar` ya enlazan los archivos entre sí: el
//  principal importa a los demás, y el compilador los
//  mete todos en un solo programa. `construir` revisa que
//  cada archivo compile y que sus `importar` existan.
//
//  Solo usa la librería estándar de Rust.
// ============================================================

use crate::herramientas::progreso::BarraProgreso;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;

// Un proyecto Eclipse: su nombre, su archivo principal
// y su versión. Se lee de Eclipse.toml.
#[derive(Debug, Clone)]
pub struct Proyecto {
    pub nombre: String,
    pub principal: String,
    pub version: String,
    // La carpeta donde está el Eclipse.toml.
    pub carpeta: PathBuf,
}

// Lee un Eclipse.toml chiquito: líneas `clave = "valor"`.
// Solo entiende las claves nombre, principal y version.
fn leer_manifiesto(ruta: &Path) -> Result<Proyecto, String> {
    let texto = fs::read_to_string(ruta)
        .map_err(|_| format!("no encontré el archivo '{}'", ruta.display()))?;
    let mut nombre = String::new();
    let mut principal = String::from("src/principal.ec");
    let mut version = String::from("0.1.0");
    for linea in texto.lines() {
        let linea = linea.trim();
        if linea.is_empty() || linea.starts_with('#') || linea.starts_with('[') {
            continue;
        }
        if let Some((clave, valor)) = linea.split_once('=') {
            let clave = clave.trim();
            let valor = valor.trim().trim_matches('"').trim_matches('\'');
            match clave {
                "nombre" => nombre = valor.to_string(),
                "principal" => principal = valor.to_string(),
                "version" => version = valor.to_string(),
                _ => {}
            }
        }
    }
    if nombre.is_empty() {
        return Err("el Eclipse.toml no dice el nombre del proyecto (nombre = \"...\")".to_string());
    }
    Ok(Proyecto {
        nombre,
        principal,
        version,
        carpeta: ruta
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| PathBuf::from(".")),
    })
}

// Busca el Eclipse.toml en la carpeta actual.
pub fn buscar_proyecto(carpeta: &Path) -> Result<Proyecto, String> {
    let ruta = carpeta.join("Eclipse.toml");
    if !ruta.exists() {
        return Err(
            "aquí no hay ningún proyecto: no encontré Eclipse.toml. Crea uno con `eclipse nuevo mi_proyecto`".to_string(),
        );
    }
    leer_manifiesto(&ruta)
}

// Crea un proyecto nuevo: la carpeta, el Eclipse.toml y
// el primer archivo src/principal.ec con un hola.
pub fn nuevo(nombre: &str) -> Result<PathBuf, String> {
    if nombre.is_empty() || nombre.contains('/') || nombre.contains('\\') {
        return Err("el nombre del proyecto no puede estar vacío ni tener / o \\".to_string());
    }
    let carpeta = PathBuf::from(nombre);
    if carpeta.exists() {
        return Err(format!("ya existe una carpeta llamada '{}'", nombre));
    }
    fs::create_dir_all(carpeta.join("src"))
        .map_err(|_| format!("no pude crear la carpeta '{}'", nombre))?;
    let manifiesto = format!(
        "[proyecto]\nnombre = \"{}\"\nprincipal = \"src/principal.ec\"\nversion = \"0.1.0\"\n",
        nombre
    );
    fs::write(carpeta.join("Eclipse.toml"), manifiesto)
        .map_err(|_| "no pude escribir el Eclipse.toml".to_string())?;
    let principal = format!(
        "// {} - tu primer programa\n// Corre con: eclipse correr\n// Compila con: eclipse construir\n\nimprimir(\"¡Hola, {}!\");\n",
        nombre, nombre
    );
    fs::write(carpeta.join("src").join("principal.ec"), principal)
        .map_err(|_| "no pude escribir el src/principal.ec".to_string())?;
    Ok(carpeta)
}

// Junta todos los archivos .ec de una carpeta (y sus
// subcarpetas), en orden alfabético.
pub fn listar_ec(carpeta: &Path) -> Vec<PathBuf> {
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

// Revisa que un archivo .ec compile (lexer + parser +
// compilador a bytecode). Devuelve Ok(()) si compila.
fn revisar_archivo(ruta: &Path) -> Result<(), String> {
    let fuente = fs::read_to_string(ruta)
        .map_err(|_| format!("no pude leer '{}'", ruta.display()))?;
    let base = ruta
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."));
    let _ = crate::motor::compilador::compilar(&fuente, base, &ruta.display().to_string())?;
    Ok(())
}

// Compila todos los .ec del proyecto, con barra de progreso.
// Con `release` también empaqueta el principal en un
// ejecutable standalone (sin rustc).
pub fn construir(proyecto: &Proyecto, release: bool) -> Result<(), String> {
    let carpeta_src = proyecto.carpeta.join("src");
    let archivos = listar_ec(&carpeta_src);
    if archivos.is_empty() {
        return Err("el proyecto no tiene ningún archivo .ec en src/".to_string());
    }
    println!(
        "Construyendo {} {} ({} archivos)...",
        proyecto.nombre,
        proyecto.version,
        archivos.len()
    );
    let mut barra = BarraProgreso::nueva(archivos.len());
    let mut fallas = 0;
    for ruta in &archivos {
        let nombre = ruta
            .strip_prefix(&proyecto.carpeta)
            .unwrap_or(ruta)
            .display()
            .to_string();
        match revisar_archivo(ruta) {
            Ok(()) => barra.avanzar(&nombre),
            Err(e) => {
                fallas += 1;
                barra.avanzar(&format!("{} <- {}", nombre, e));
            }
        }
    }
    if fallas > 0 {
        barra.terminar(&format!(
            "Terminó con {} {} de {}: arregla esos errores y vuelve a construir.",
            fallas,
            if fallas == 1 { "error" } else { "errores" },
            archivos.len()
        ));
        return Err(format!("{} archivo(s) no compilaron", fallas));
    }
    barra.terminar(&format!(
        "¡Listo! {} archivo(s) compilaron bien.",
        archivos.len()
    ));
    if release {
        let ruta_principal = proyecto.carpeta.join(&proyecto.principal);
        let ruta_str = ruta_principal.display().to_string();
        let salida = crate::motor::paquete::empaquetar(&ruta_str, None)?;
        println!("Ejecutable standalone en: {}", salida.display());
        println!("Lo puedes copiar a otra computadora y correrlo sin instalar nada.");
    }
    Ok(())
}

// Corre todos los .ec de una carpeta, uno por uno, cada uno
// en su propio proceso, con barra de progreso y un resumen
// al final: cuántos pasaron y cuáles fallaron.
pub fn correr_todos(carpeta: &Path) -> Result<(), String> {
    let archivos = listar_ec(carpeta);
    if archivos.is_empty() {
        return Err(format!(
            "no encontré ningún archivo .ec en '{}'",
            carpeta.display()
        ));
    }
    println!("Corriendo {} archivo(s) .ec...", archivos.len());
    let mut barra = BarraProgreso::nueva(archivos.len());
    let mut pasaron = 0usize;
    let mut fallaron: Vec<(String, String)> = Vec::new();
    let yo = env::current_exe().unwrap_or_else(|_| PathBuf::from("eclipse"));
    for ruta in &archivos {
        let nombre = ruta.display().to_string();
        let salida = process::Command::new(&yo)
            .arg(ruta)
            .stdin(process::Stdio::null())
            .output();
        match salida {
            Ok(o) if o.status.success() => {
                pasaron += 1;
                barra.avanzar(&nombre);
            }
            Ok(o) => {
                let error = String::from_utf8_lossy(&o.stderr)
                    .lines()
                    .next()
                    .unwrap_or("falló sin decir por qué")
                    .to_string();
                fallaron.push((nombre.clone(), error));
                barra.avanzar(&format!("{} <- falló", nombre));
            }
            Err(e) => {
                fallaron.push((nombre.clone(), format!("no pude correrlo: {}", e)));
                barra.avanzar(&format!("{} <- falló", nombre));
            }
        }
    }
    let resumen = format!(
        "Pasaron {} de {} archivo(s).{}",
        pasaron,
        archivos.len(),
        if fallaron.is_empty() {
            String::new()
        } else {
            let mut s = String::from("\nFallaron:\n");
            for (n, e) in &fallaron {
                s.push_str(&format!("  - {}: {}\n", n, e));
            }
            s
        }
    );
    barra.terminar(&resumen);
    if fallaron.is_empty() {
        Ok(())
    } else {
        Err(format!("{} archivo(s) fallaron", fallaron.len()))
    }
}
