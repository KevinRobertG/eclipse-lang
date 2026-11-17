//  ECLIPSE 1.2 - El banco de pruebas de velocidad (todo en Rust)
//  Mide cuánto tarda cada motor en los mismos programas y dice
//  cuántas veces más rápido es uno que otro. Los números son
//  reales, medidos en esta computadora, no promesas.
//  Uso: cargo run --bin bench
//  (el .exe solo se mide si hay rustc en el PATH).

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

fn eclipse_bin() -> PathBuf {
    let mut p = std::env::current_exe().expect("no sé dónde estoy");
    p.pop();
    p.push("eclipse");
    p
}

fn medir(cmd: &mut Command, repeticiones: u32) -> Option<Duration> {
    let mut mejor = Duration::MAX;
    for _ in 0..repeticiones {
        let inicio = Instant::now();
        let salida = cmd.output().ok()?;
        if !salida.status.success() {
            return None;
        }
        let t = inicio.elapsed();
        if t < mejor {
            mejor = t;
        }
    }
    Some(mejor)
}

fn ms(d: Duration) -> f64 {
    d.as_secs_f64() * 1000.0
}

fn main() {
    let eclipse = eclipse_bin();
    let con_exe = Command::new("rustc").arg("--version").output().is_ok();
    let archivos = ["fib.ec", "bucle.ec", "para.ec", "texto.ec", "lista.ec"];
    let repeticiones = 3;

    println!("Eclipse 1.2 - banco de velocidad");
    println!("(el mejor de {} corridas, en milisegundos)", repeticiones);
    println!();
    println!(
        "{:<10} {:>12} {:>12} {:>12}",
        "programa", "VM", ".exe", "paquete"
    );
    println!("{}", "-".repeat(50));

    for nombre in archivos {
        let ruta = Path::new("banco").join(nombre);
        // La VM es el modo normal (el intérprete ya no existe).
        let t_vm = medir(Command::new(&eclipse).arg(&ruta), repeticiones);
        let t_exe = if con_exe {
            // Generar el .exe una vez (eso es compilar, no correr).
            let gen = Command::new(&eclipse)
                .arg("--exe")
                .arg(&ruta)
                .output();
            let ruta_exe = ruta.with_extension("");
            let t = match gen {
                Ok(s) if s.status.success() => {
                    medir(&mut Command::new(&ruta_exe), repeticiones)
                }
                _ => None,
            };
            let _ = std::fs::remove_file(&ruta_exe);
            let tallo = ruta.file_stem().unwrap().to_string_lossy().to_string();
            let _ = std::fs::remove_dir_all(format!("banco/{}_eclipse", tallo));
            t
        } else {
            None
        };
        // El paquete: empaquetar una vez y medir el archivo solo.
        let paquete_ruta = ruta.with_extension("").with_extension("paq");
        let t_paq = match Command::new(&eclipse)
            .arg("empaquetar")
            .arg(&ruta)
            .arg(&paquete_ruta)
            .output()
        {
            Ok(s) if s.status.success() => medir(&mut Command::new(&paquete_ruta), repeticiones),
            _ => None,
        };
        let _ = std::fs::remove_file(&paquete_ruta);

        let f = |t: Option<Duration>| match t {
            Some(d) => format!("{:>12.2}", ms(d)),
            None => format!("{:>12}", "-"),
        };
        println!(
            "{:<10} {} {} {}",
            nombre, f(t_vm), f(t_exe), f(t_paq)
        );
        if let (Some(v), Some(e)) = (t_vm, t_exe) {
            println!(
                "           -> el .exe es {:.1}x más rápido que la VM",
                ms(v) / ms(e)
            );
        }
    }
    println!();
    println!("Nota honesta: el .exe es código Rust generado por Eclipse,");
    println!("por eso corre a velocidad Rust. La VM corre bytecode, así");
    println!("que nunca va a ser más rápida que el .exe en general.");
    println!("Pero la VM es EL motor de Eclipse: todo corre con ella.");
}
