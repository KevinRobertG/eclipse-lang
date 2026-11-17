// ============================================================
//  ECLIPSE 1.1 - La barra de progreso
//  Archivo: src/herramientas/progreso.rs
//
//  Qué hace:
//  Dibuja una barra de progreso en la terminal, como la de
//  Cargo: `[##########----------] 5/10 (50%) nombre.ec`.
//  Se actualiza en su mismo lugar (con \r), para no llenar
//  la pantalla de líneas, y al terminar deja el mensaje
//  final en su propia línea.
//
//  Solo usa la librería estándar de Rust.
// ============================================================

use std::io::{self, Write};

// Una barra de progreso: sabe cuántos pasos hay en total
// y cuántos llevamos, y dibuja la barra en la terminal.
pub struct BarraProgreso {
    total: usize,
    actual: usize,
    // Cuántos caracteres de ancho tiene la barra.
    ancho: usize,
}

impl BarraProgreso {
    // Crea una barra para `total` pasos.
    pub fn nueva(total: usize) -> Self {
        BarraProgreso {
            total: total.max(1),
            actual: 0,
            ancho: 20,
        }
    }

    // Avanza un paso y dibuja la barra con el mensaje de
    // lo que se está haciendo ahora mismo.
    pub fn avanzar(&mut self, mensaje: &str) {
        self.actual += 1;
        self.dibujar(mensaje);
    }

    // Dibuja la barra sin avanzar (para el paso 0).
    pub fn dibujar(&self, mensaje: &str) {
        let porcentaje = self.actual * 100 / self.total;
        let llenos = self.actual * self.ancho / self.total;
        let vacios = self.ancho - llenos;
        let barra: String = "#".repeat(llenos) + &"-".repeat(vacios);
        eprint!(
            "\r[{barra}] {}/{} ({}%) {}",
            self.actual, self.total, porcentaje, mensaje
        );
        let _ = io::stderr().flush();
    }

    // Termina: salta de línea y escribe el mensaje final.
    pub fn terminar(&self, mensaje: &str) {
        eprintln!();
        eprintln!("{}", mensaje);
    }
}
