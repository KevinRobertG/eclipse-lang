// ============================================================
//  ECLIPSE 1.1 - Hablar con el hardware y el software
//  Archivo: src/motor/sistema.rs
//
//  Qué es esto:
//  Las funciones que dejan a Eclipse hablar con la computadora:
//  qué sistema es, quién eres, en qué carpeta estás, cuántos
//  procesadores tienes, cuánta memoria, y correr otros
//  programas. Todo con la librería estándar de Rust, sin
//  descargar nada.
//
//  Con honestidad: Eclipse corre como un programa normal, no
//  como un controlador del sistema. No puede tocar el hardware
//  directo (eso es trabajo del kernel: sensores, puertos,
//  drivers). Lo que SÍ puede es preguntarle al sistema por
//  sus datos (memoria, procesadores, sistema) y pedirle que
//  corra otros programas. Eso es como hablan los programas
//  normales con la computadora.
// ============================================================

use std::env;
use std::fs;
use std::process::Command;

// Qué sistema operativo es: "Windows", "Linux", "macOS"...
pub fn sistema() -> String {
    match env::consts::OS {
        "windows" => "Windows".to_string(),
        "linux" => "Linux".to_string(),
        "macos" => "macOS".to_string(),
        otro => otro.to_string(),
    }
}

// Quién eres (tu nombre de usuario del sistema).
pub fn usuario() -> String {
    env::var("USER")
        .or_else(|_| env::var("USERNAME"))
        .unwrap_or_else(|_| "?".to_string())
}

// En qué carpeta estás parado.
pub fn carpeta() -> Result<String, String> {
    env::current_dir()
        .map(|p| p.to_string_lossy().to_string())
        .map_err(|_| "no pude saber en qué carpeta estoy".to_string())
}

// El valor de una variable de entorno (como PATH o HOME).
pub fn variable_entorno(nombre: &str) -> Result<String, String> {
    env::var(nombre).map_err(|_| format!("no existe la variable de entorno '{}'", nombre))
}

// Corre otro programa y te devuelve lo que imprimió.
// En Windows usa `cmd /C`, en los demás usa `sh -c`.
pub fn comando(texto: &str) -> Result<String, String> {
    let salida = if cfg!(windows) {
        Command::new("cmd").args(["/C", texto]).output()
    } else {
        Command::new("sh").args(["-c", texto]).output()
    };
    let salida = salida.map_err(|_| format!("no pude correr el comando '{}'", texto))?;
    if !salida.status.success() {
        let codigo = salida.status.code().unwrap_or(-1);
        let error = String::from_utf8_lossy(&salida.stderr);
        let error = error.trim();
        if error.is_empty() {
            return Err(format!("el comando terminó con error (código {})", codigo));
        }
        return Err(format!("el comando terminó con error (código {}): {}", codigo, error));
    }
    Ok(String::from_utf8_lossy(&salida.stdout).trim_end().to_string())
}

// Los datos que le diste al programa al correrlo
// (por ejemplo: eclipse programa.ec hola -> ["eclipse", "programa.ec", "hola"]).
pub fn argumentos() -> Vec<String> {
    env::args().collect()
}

// Cuántos procesadores (núcleos) tiene la computadora.
pub fn procesadores() -> f64 {
    std::thread::available_parallelism()
        .map(|n| n.get() as f64)
        .unwrap_or(1.0)
}

// Cuánta memoria total tiene la computadora, en MB.
// En Linux lee /proc/meminfo; en Windows pregunta con wmic;
// en macOS pregunta con sysctl. Si no se puede, da error.
pub fn memoria_mb() -> Result<f64, String> {
    if cfg!(target_os = "linux") {
        let texto = fs::read_to_string("/proc/meminfo")
            .map_err(|_| "no pude leer la memoria en este sistema".to_string())?;
        for linea in texto.lines() {
            if let Some(resto) = linea.strip_prefix("MemTotal:") {
                let numero: String = resto
                    .chars()
                    .filter(|c| c.is_ascii_digit())
                    .collect();
                if let Ok(kb) = numero.parse::<f64>() {
                    return Ok(kb / 1024.0);
                }
            }
        }
        return Err("no encontré la memoria total en /proc/meminfo".to_string());
    }
    if cfg!(windows) {
        let salida = Command::new("wmic")
            .args(["computersystem", "get", "TotalPhysicalMemory", "/value"])
            .output()
            .map_err(|_| "no pude leer la memoria en este sistema".to_string())?;
        let texto = String::from_utf8_lossy(&salida.stdout);
        for linea in texto.lines() {
            if let Some(resto) = linea.trim().strip_prefix("TotalPhysicalMemory=") {
                if let Ok(bytes) = resto.trim().parse::<f64>() {
                    return Ok(bytes / 1024.0 / 1024.0);
                }
            }
        }
        return Err("no encontré la memoria total (wmic)".to_string());
    }
    if cfg!(target_os = "macos") {
        let salida = Command::new("sysctl")
            .args(["-n", "hw.memsize"])
            .output()
            .map_err(|_| "no pude leer la memoria en este sistema".to_string())?;
        let texto = String::from_utf8_lossy(&salida.stdout);
        if let Ok(bytes) = texto.trim().parse::<f64>() {
            return Ok(bytes / 1024.0 / 1024.0);
        }
        return Err("no encontré la memoria total (sysctl)".to_string());
    }
    Err("no puedo leer la memoria total en este sistema".to_string())
}
