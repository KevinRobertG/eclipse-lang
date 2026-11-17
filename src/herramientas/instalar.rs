// ============================================================
//  ECLIPSE 1.2 - El instalador (como rustup, pero en español)
//  Archivo: src/herramientas/instalar.rs
//
//  Qué hace:
//  `eclipse instalar` deja Eclipse listo para usar, sin
//  necesitar Rust ni Cargo:
//      1. Copia el programa `eclipse` a ~/.eclipse/bin
//         (en Windows: %USERPROFILE%\.eclipse\bin).
//      2. Agrega esa carpeta al PATH, para que puedas escribir
//         `eclipse` en cualquier terminal (en Windows lo hace
//         con el registro; en Linux y Mac, con tu .bashrc
//         o .zshrc).
//      3. Instala la extensión de VS Code (colores, botón
//         ▶ Ejecutar, autocompletado, diagnósticos): los
//         archivos de la extensión van incrustados dentro
//         del propio programa Eclipse, así que no hace falta
//         descargar nada más.
//
//  `eclipse desinstalar` quita el programa y la extensión.
//
//  Solo usa la librería estándar de Rust.
// ============================================================

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;

// Los archivos de la extensión de VS Code, incrustados
// dentro del programa Eclipse (así `instalar` no necesita
// descargar nada, ni que tengas la carpeta del proyecto).
const EXTENSION_PACKAGE_JSON: &str = include_str!("../../vscode-eclipse/package.json");
const EXTENSION_JS: &str = include_str!("../../vscode-eclipse/extension.js");
const EXTENSION_CONFIG: &str = include_str!("../../vscode-eclipse/language-configuration.json");
const EXTENSION_GRAMMAR: &str =
    include_str!("../../vscode-eclipse/syntaxes/eclipse.tmLanguage.json");

// La versión de la extensión que se instala.
const VERSION: &str = "1.2.0";

// La carpeta de Eclipse en tu casa.
fn carpeta_eclipse() -> Option<PathBuf> {
    let casa = env::var_os("HOME")
        .map(PathBuf::from)
        .or_else(|| env::var_os("USERPROFILE").map(PathBuf::from))?;
    Some(casa.join(".eclipse"))
}

// La carpeta donde VS Code guarda sus extensiones.
fn carpeta_vscode() -> Option<PathBuf> {
    let casa = env::var_os("HOME")
        .map(PathBuf::from)
        .or_else(|| env::var_os("USERPROFILE").map(PathBuf::from))?;
    Some(casa.join(".vscode").join("extensions"))
}

fn nombre_eclipse() -> &'static str {
    if cfg!(windows) {
        "eclipse.exe"
    } else {
        "eclipse"
    }
}

// Agrega la carpeta al PATH del usuario.
// En Windows usa el registro (como rustup); en Linux y Mac
// agrega la línea a tu .bashrc y .zshrc si los tienes.
// Devuelve verdadero si agregó algo (o si ya estaba), y
// falso si no pudo agregar nada (porque no tienes .bashrc
// ni .zshrc: entonces te dice el comando a mano).
fn agregar_al_path(bin: &Path) -> Result<bool, String> {
    let bin_texto = bin.display().to_string();
    if cfg!(windows) {
        let path_actual = env::var("Path").unwrap_or_default();
        if path_actual
            .split(';')
            .any(|p| p.eq_ignore_ascii_case(&bin_texto))
        {
            return Ok(true);
        }
        let nuevo = if path_actual.is_empty() {
            bin_texto.clone()
        } else {
            format!("{};{}", path_actual, bin_texto)
        };
        let salida = process::Command::new("reg")
            .args([
                "add",
                "HKCU\\Environment",
                "/v",
                "Path",
                "/t",
                "REG_EXPAND_SZ",
                "/d",
                &nuevo,
                "/f",
            ])
            .output();
        match salida {
            Ok(o) if o.status.success() => Ok(true),
            _ => Err(format!(
                "no pude agregar '{}' al PATH. Hazlo a mano con PowerShell:\n[Environment]::SetEnvironmentVariable(\"Path\", $env:Path + \";{}\", \"User\")",
                bin_texto, bin_texto
            )),
        }
    } else {
        let mut agrego = false;
        for rc in [".bashrc", ".zshrc"] {
            let casa = match env::var_os("HOME") {
                Some(c) => PathBuf::from(c),
                None => continue,
            };
            let ruta_rc = casa.join(rc);
            if !ruta_rc.exists() {
                continue;
            }
            let contenido = fs::read_to_string(&ruta_rc).unwrap_or_default();
            let linea = "export PATH=\"$HOME/.eclipse/bin:$PATH\"".to_string();
            if contenido.contains(".eclipse/bin") {
                agrego = true;
                continue;
            }
            let mut nuevo = contenido;
            if !nuevo.ends_with('\n') && !nuevo.is_empty() {
                nuevo.push('\n');
            }
            nuevo.push_str(&linea);
            nuevo.push('\n');
            let _ = fs::write(&ruta_rc, nuevo);
            println!("Agregué Eclipse al PATH en {}", ruta_rc.display());
            agrego = true;
        }
        Ok(agrego)
    }
}

// Instala Eclipse y su extensión de VS Code.
pub fn instalar() -> Result<(), String> {
    let carpeta = carpeta_eclipse()
        .ok_or_else(|| "no encontré tu carpeta de usuario (HOME)".to_string())?;
    let bin = carpeta.join("bin");
    fs::create_dir_all(&bin).map_err(|_| "no pude crear la carpeta .eclipse/bin".to_string())?;

    // 1. Copiar el programa eclipse.
    let yo = env::current_exe()
        .map_err(|_| "no pude encontrar el programa eclipse que está corriendo".to_string())?;
    let destino = bin.join(nombre_eclipse());
    fs::copy(&yo, &destino)
        .map_err(|_| format!("no pude copiar Eclipse a '{}'", destino.display()))?;
    println!("Eclipse instalado en: {}", destino.display());

    // 2. Agregar al PATH.
    match agregar_al_path(&bin) {
        Ok(true) => println!("La carpeta {} ya está en tu PATH (o se agregó).", bin.display()),
        Ok(false) => println!(
            "No tienes .bashrc ni .zshrc, así que no pude agregar el PATH solo. Para usar 'eclipse' en cualquier terminal, escribe esto una vez:\n  export PATH=\"$HOME/.eclipse/bin:$PATH\""
        ),
        Err(e) => println!("Aviso: {}", e),
    }
    println!("Cierra y vuelve a abrir la terminal para que el PATH nuevo funcione.");

    // 3. Instalar la extensión de VS Code.
    let vscode = carpeta_vscode()
        .ok_or_else(|| "no encontré tu carpeta de usuario (HOME)".to_string())?;
    fs::create_dir_all(&vscode)
        .map_err(|_| "no pude crear la carpeta de extensiones de VS Code".to_string())?;
    // Quitar versiones viejas de la extensión.
    if let Ok(entradas) = fs::read_dir(&vscode) {
        for entrada in entradas.flatten() {
            let nombre = entrada.file_name().to_string_lossy().to_string();
            if nombre.starts_with("kevin.eclipse-lenguaje-") {
                let _ = fs::remove_dir_all(entrada.path());
            }
        }
    }
    let carpeta_ext = vscode.join(format!("kevin.eclipse-lenguaje-{}", VERSION));
    fs::create_dir_all(carpeta_ext.join("syntaxes"))
        .map_err(|_| "no pude crear la carpeta de la extensión".to_string())?;
    fs::write(carpeta_ext.join("package.json"), EXTENSION_PACKAGE_JSON)
        .map_err(|_| "no pude escribir el package.json de la extensión".to_string())?;
    fs::write(carpeta_ext.join("extension.js"), EXTENSION_JS)
        .map_err(|_| "no pude escribir el extension.js".to_string())?;
    fs::write(
        carpeta_ext.join("language-configuration.json"),
        EXTENSION_CONFIG,
    )
    .map_err(|_| "no pude escribir la configuración del lenguaje".to_string())?;
    fs::write(
        carpeta_ext.join("syntaxes").join("eclipse.tmLanguage.json"),
        EXTENSION_GRAMMAR,
    )
    .map_err(|_| "no pude escribir la gramática de colores".to_string())?;
    println!(
        "Extensión de VS Code instalada en: {}",
        carpeta_ext.display()
    );
    println!("Cierra y vuelve a abrir VS Code: tus archivos .ec ya tendrán colores, autocompletado y el botón ▶ Ejecutar.");
    Ok(())
}

// Quita Eclipse y su extensión.
pub fn desinstalar() -> Result<(), String> {
    let carpeta = carpeta_eclipse()
        .ok_or_else(|| "no encontré tu carpeta de usuario (HOME)".to_string())?;
    let bin = carpeta.join("bin").join(nombre_eclipse());
    if bin.exists() {
        fs::remove_file(&bin).map_err(|_| "no pude borrar el programa Eclipse".to_string())?;
        println!("Borré {}", bin.display());
    } else {
        println!("Eclipse no estaba instalado en {}", bin.display());
    }
    if let Some(vscode) = carpeta_vscode() {
        if let Ok(entradas) = fs::read_dir(&vscode) {
            for entrada in entradas.flatten() {
                let nombre = entrada.file_name().to_string_lossy().to_string();
                if nombre.starts_with("kevin.eclipse-lenguaje-") {
                    let _ = fs::remove_dir_all(entrada.path());
                    println!("Borré la extensión {}", nombre);
                }
            }
        }
    }
    println!("Listo. Tu PATH sigue igual; si quieres quitar la línea de .eclipse/bin, bórrala a mano.");
    Ok(())
}
