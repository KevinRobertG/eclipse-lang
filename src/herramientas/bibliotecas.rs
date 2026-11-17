// ============================================================
//  ECLIPSE 1.2 - Las librerías (como las crates de Rust)
//  Archivo: src/herramientas/bibliotecas.rs
//
//  Qué es esto:
//  Una librería es un archivo .ec con funciones que cualquiera
//  puede escribir y compartir: para videojuegos, para dibujar,
//  para matemáticas... Se usa con `usar`:
//
//      usar matematicas;
//      imprimir(factorial(5));
//
//  `usar` solo vale arriba del archivo, antes de cualquier
//  otra línea (eso lo revisa el parser).
//
//  El Registro (un sistema aparte, del mismo proyecto):
//  Las librerías NO viven dentro de Eclipse: viven en el
//  Registro, su propio sistema en ~/.eclipse/registro/:
//      registro/bibliotecas/<nombre>/   las disponibles
//      registro/cola/<nombre>/          las que están en revisión
//      registro/.ultima_subida          cuándo se subió la última
//  Rust solo se encarga de los archivos del proyecto Eclipse
//  (el código de Eclipse está escrito en Rust). Los programas
//  .ec no necesitan Rust para nada: corren en la VM.
//
//  Cómo se sube una librería (con seguridad):
//  1. `eclipse biblioteca publicar mi_libreria.ec` te hace
//     6 preguntas por terminal: nombre de la librería,
//     tu nombre, descripción, versión, categoría y contacto.
//  2. Solo se puede subir una librería cada 5 horas.
//  3. La librería se revisa antes de subirse: tiene que
//     estar escrita SOLO en Eclipse (nada de otros lenguajes),
//     en español, sin cosas peligrosas, y tiene que tener al
//     menos una función (nada de librerías mal hechas).
//  4. Después de subirla, tarda de 1 a 2 días en llegar a
//     Eclipse, sin excepciones: queda en la cola de revisión
//     y solo se puede usar cuando se cumple el tiempo.
//  5. Cada vez que la usas, Eclipse revisa que nadie la haya
//     cambiado (checksum).
//
//  Seguridad (con honestidad):
//  Ningún revisor puede garantizar al 100% que una librería
//  sea segura (igual que en las tiendas de apps de verdad):
//  instala solo librerías de personas en las que confíes.
// ============================================================

use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::lenguaje::ast::Declaracion;
use crate::lenguaje::lexer::{Lexer, TipoToken};
use crate::lenguaje::parser::Parser;

// ---- Palabras en inglés que no valen en una librería ----
// Si el código usa alguna como nombre, la librería no está
// en español y no se publica.
const PALABRAS_INGLES: &[&str] = &[
    "let", "fn", "if", "else", "while", "for", "return", "print", "println", "import",
    "function", "var", "const", "true", "false", "null", "nil", "class", "def", "new",
    "this", "self", "int", "string", "bool", "void", "public", "private", "static",
    "switch", "case", "break", "continue", "do", "try", "catch", "throw", "extends",
    "implements", "package", "namespace", "using", "include", "require", "echo",
    "lambda", "async", "await", "yield", "struct", "enum", "trait", "impl", "match",
    "loop", "from",
];

// ---- Palabras peligrosas para el nombre o la descripción ----
// Una librería no puede llamarse firmware, virus, malware...
const PALABRAS_PELIGROSAS: &[&str] = &[
    "firmware", "bios", "malware", "virus", "troyano", "keylogger", "ransomware", "driver",
    "kernel", "rootkit", "spyware", "gusano",
];

// ---- Palabras de Eclipse que no pueden ser nombre de librería ----
// Si una librería se llamara "si", nadie podría escribir
// `usar si;` porque "si" es una palabra del lenguaje.
const PALABRAS_ECLIPSE: &[&str] = &[
    "sea", "mut", "si", "sino", "mientras", "para", "en", "funcion", "devolver", "imprimir",
    "leer", "verdadero", "falso", "y", "o", "no", "romper", "seguir", "nulo", "importar",
    "numero", "texto", "logico", "segun", "paso", "repetir", "lista", "para_cada",
    "constante", "usar",
];

// Una librería no puede pesar más de 500 KB.
const MAX_TAMANO: usize = 500 * 1024;
// La revisión tarda 1 día y medio (entre 1 y 2 días).
const ESPERA_REVISION_SEGS: u64 = 36 * 3600;
// Solo se puede subir una librería cada 5 horas.
const LIMITE_SUBIDA_SEGS: u64 = 5 * 3600;

// ---- El tiempo ----

// Los segundos desde el 1 de enero de 1970.
fn ahora_segs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

// Cuánto falta, en palabras: "1 día y 12 horas",
// "5 horas y 30 minutos", "45 minutos".
fn falta_bonita(segs: u64) -> String {
    let dias = segs / 86400;
    let horas = (segs % 86400) / 3600;
    let minutos = (segs % 3600) / 60;
    if dias > 0 {
        if horas > 0 {
            format!(
                "{} {} y {} {}",
                dias,
                if dias == 1 { "día" } else { "días" },
                horas,
                if horas == 1 { "hora" } else { "horas" }
            )
        } else {
            format!("{} {}", dias, if dias == 1 { "día" } else { "días" })
        }
    } else if horas > 0 {
        if minutos > 0 {
            format!(
                "{} {} y {} {}",
                horas,
                if horas == 1 { "hora" } else { "horas" },
                minutos,
                if minutos == 1 { "minuto" } else { "minutos" }
            )
        } else {
            format!("{} {}", horas, if horas == 1 { "hora" } else { "horas" })
        }
    } else if minutos > 0 {
        format!("{} {}", minutos, if minutos == 1 { "minuto" } else { "minutos" })
    } else {
        "menos de un minuto".to_string()
    }
}

// ---- Las carpetas del Registro ----

// Tu carpeta de usuario (~/.eclipse).
pub fn carpeta_eclipse() -> Result<PathBuf, String> {
    let casa = env::var_os("HOME")
        .map(PathBuf::from)
        .or_else(|| env::var_os("USERPROFILE").map(PathBuf::from))
        .ok_or_else(|| "no encontré tu carpeta de usuario (HOME)".to_string())?;
    Ok(casa.join(".eclipse"))
}

// El Registro: el sistema aparte donde viven las librerías.
pub fn carpeta_registro() -> Result<PathBuf, String> {
    Ok(carpeta_eclipse()?.join("registro"))
}

// Donde viven las librerías disponibles.
pub fn carpeta_bibliotecas() -> Result<PathBuf, String> {
    Ok(carpeta_registro()?.join("bibliotecas"))
}

// Donde esperan las librerías en revisión.
pub fn carpeta_cola() -> Result<PathBuf, String> {
    Ok(carpeta_registro()?.join("cola"))
}

// La carpeta de una librería disponible: registro/bibliotecas/<nombre>/
pub fn carpeta_biblioteca(nombre: &str) -> Result<PathBuf, String> {
    Ok(carpeta_bibliotecas()?.join(nombre))
}

// La carpeta de un envío en revisión: registro/cola/<nombre>/
pub fn carpeta_envio(nombre: &str) -> Result<PathBuf, String> {
    Ok(carpeta_cola()?.join(nombre))
}

// El archivo principal de una librería disponible: <nombre>.ec
pub fn ruta_biblioteca(nombre: &str) -> Result<PathBuf, String> {
    Ok(carpeta_biblioteca(nombre)?.join(format!("{}.ec", nombre)))
}

// El archivo de un envío en revisión: <nombre>.ec
pub fn ruta_envio(nombre: &str) -> Result<PathBuf, String> {
    Ok(carpeta_envio(nombre)?.join(format!("{}.ec", nombre)))
}

// El manifiesto: biblioteca.toml (nombre, versión, autor, checksum...).
pub fn ruta_manifiesto(nombre: &str) -> Result<PathBuf, String> {
    Ok(carpeta_biblioteca(nombre)?.join("biblioteca.toml"))
}

// Los datos del envío: envio.toml (con enviado_en y listo_en).
pub fn ruta_envio_manifiesto(nombre: &str) -> Result<PathBuf, String> {
    Ok(carpeta_envio(nombre)?.join("envio.toml"))
}

// Cuándo se subió la última librería (para el límite de 5 horas).
fn ruta_ultima_subida() -> Result<PathBuf, String> {
    Ok(carpeta_registro()?.join(".ultima_subida"))
}

// ---- Migración desde Eclipse 1.0 ----
// En la 1.0 las librerías vivían en ~/.eclipse/bibliotecas/.
// En la 1.1 viven en el Registro (~/.eclipse/registro/).
// Si queda alguna en el lugar viejo, se mueve sola.
pub fn migrar_registro() {
    let Ok(vieja) = carpeta_eclipse().map(|c| c.join("bibliotecas")) else {
        return;
    };
    if !vieja.exists() {
        return;
    }
    let Ok(nueva) = carpeta_bibliotecas() else {
        return;
    };
    if fs::create_dir_all(&nueva).is_err() {
        return;
    }
    let Ok(entradas) = fs::read_dir(&vieja) else {
        return;
    };
    for entrada in entradas.flatten() {
        let ruta = entrada.path();
        if !ruta.is_dir() {
            continue;
        }
        let Some(nombre) = ruta.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        let destino = nueva.join(nombre);
        if destino.exists() {
            continue;
        }
        if fs::rename(&ruta, &destino).is_err() {
            continue;
        }
        // El manifiesto viejo no tiene estado: ya está disponible.
        if let Ok(ruta_m) = ruta_manifiesto(nombre) {
            if let Ok(texto) = fs::read_to_string(&ruta_m) {
                if !texto.contains("estado") {
                    let _ = fs::write(&ruta_m, format!("{}estado = \"disponible\"\n", texto));
                }
            }
        }
    }
    // Quitar la carpeta vieja si quedó vacía.
    let _ = fs::remove_dir(&vieja);
}

// ---- El número de control (checksum) ----
// Un hash chiquito (FNV-1a): si el archivo cambia un solo
// carácter, el número cambia por completo.
pub fn hash_simple(texto: &str) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325;
    for b in texto.bytes() {
        hash ^= b as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

// ---- El manifiesto ----

// `estado` y `enviado_en` se guardan en los archivos del Registro
// aunque no se lean en el código: son parte del registro.
#[allow(dead_code)]
pub struct Manifiesto {
    pub nombre: String,
    pub version: String,
    pub autor: String,
    pub descripcion: String,
    pub categoria: String,
    pub contacto: String,
    pub estado: String,
    pub checksum: u64,
}

fn leer_valor(texto: &str, clave: &str) -> Option<String> {
    for linea in texto.lines() {
        let linea = linea.trim();
        if let Some(resto) = linea.strip_prefix(clave) {
            let resto = resto.trim_start_matches([' ', '=']).trim();
            let resto = resto.trim_matches('"');
            return Some(resto.to_string());
        }
    }
    None
}

pub fn leer_manifiesto(nombre: &str) -> Result<Manifiesto, String> {
    let ruta = ruta_manifiesto(nombre)?;
    let texto = fs::read_to_string(&ruta)
        .map_err(|_| format!("la librería '{}' no tiene manifiesto", nombre))?;
    Ok(Manifiesto {
        nombre: leer_valor(&texto, "nombre").unwrap_or_else(|| nombre.to_string()),
        version: leer_valor(&texto, "version").unwrap_or_else(|| "0.1.0".to_string()),
        autor: leer_valor(&texto, "autor").unwrap_or_else(|| "?".to_string()),
        descripcion: leer_valor(&texto, "descripcion").unwrap_or_default(),
        categoria: leer_valor(&texto, "categoria").unwrap_or_else(|| "general".to_string()),
        contacto: leer_valor(&texto, "contacto").unwrap_or_default(),
        estado: leer_valor(&texto, "estado").unwrap_or_else(|| "disponible".to_string()),
        checksum: leer_valor(&texto, "checksum")
            .and_then(|v| v.parse().ok())
            .unwrap_or(0),
    })
}

// ---- El envío (una librería en revisión) ----

#[allow(dead_code)]
pub struct Envio {
    pub nombre: String,
    pub version: String,
    pub autor: String,
    pub descripcion: String,
    pub categoria: String,
    pub contacto: String,
    pub enviado_en: u64,
    pub listo_en: u64,
    pub checksum: u64,
}

fn leer_envio(nombre: &str) -> Result<Envio, String> {
    let ruta = ruta_envio_manifiesto(nombre)?;
    let texto = fs::read_to_string(&ruta)
        .map_err(|_| format!("la librería '{}' no está en revisión", nombre))?;
    Ok(Envio {
        nombre: leer_valor(&texto, "nombre").unwrap_or_else(|| nombre.to_string()),
        version: leer_valor(&texto, "version").unwrap_or_else(|| "0.1.0".to_string()),
        autor: leer_valor(&texto, "autor").unwrap_or_else(|| "?".to_string()),
        descripcion: leer_valor(&texto, "descripcion").unwrap_or_default(),
        categoria: leer_valor(&texto, "categoria").unwrap_or_else(|| "general".to_string()),
        contacto: leer_valor(&texto, "contacto").unwrap_or_default(),
        enviado_en: leer_valor(&texto, "enviado_en")
            .and_then(|v| v.parse().ok())
            .unwrap_or(0),
        listo_en: leer_valor(&texto, "listo_en")
            .and_then(|v| v.parse().ok())
            .unwrap_or(0),
        checksum: leer_valor(&texto, "checksum")
            .and_then(|v| v.parse().ok())
            .unwrap_or(0),
    })
}

// ---- Resolver una librería para `usar` ----
// Revisa que exista, que tenga manifiesto y que el archivo
// no haya cambiado desde que se publicó (checksum).
// Si todavía está en revisión, dice cuánto falta.
pub fn resolver_biblioteca(nombre: &str) -> Result<PathBuf, String> {
    migrar_registro();
    // Primero: si hay envíos que ya cumplieron su tiempo,
    // se aprueban solos.
    let _ = revisar_cola();
    let ruta = ruta_biblioteca(nombre)?;
    if ruta.exists() {
        let manifiesto = leer_manifiesto(nombre)?;
        let fuente = fs::read_to_string(&ruta)
            .map_err(|_| format!("no pude leer la librería '{}'", nombre))?;
        if hash_simple(&fuente) != manifiesto.checksum {
            return Err(format!(
                "la librería '{}' cambió desde que se publicó (su número de control ya no coincide). Vuelve a publicarla o reinstálala.",
                nombre
            ));
        }
        return Ok(ruta);
    }
    // ¿Está en la cola de revisión todavía?
    if let Ok(envio) = leer_envio(nombre) {
        let falta = envio.listo_en.saturating_sub(ahora_segs());
        return Err(format!(
            "la librería '{}' todavía está en revisión: llega a Eclipse en {}. Tienes que esperar, sin excepciones.",
            nombre,
            falta_bonita(falta)
        ));
    }
    Err(format!(
        "no encontré la librería '{}'. ¿Ya la instalaste? Publícala con: eclipse biblioteca publicar {}.ec",
        nombre, nombre
    ))
}

// ---- Revisar la cola ----
// Las librerías que ya cumplieron su tiempo de revisión
// pasan a estar disponibles. Devuelve los nombres de las
// que se aprobaron.
pub fn revisar_cola() -> Vec<String> {
    migrar_registro();
    let mut aprobadas = Vec::new();
    let Ok(cola) = carpeta_cola() else {
        return aprobadas;
    };
    if !cola.exists() {
        return aprobadas;
    }
    let Ok(entradas) = fs::read_dir(&cola) else {
        return aprobadas;
    };
    let ahora = ahora_segs();
    for entrada in entradas.flatten() {
        let ruta = entrada.path();
        if !ruta.is_dir() {
            continue;
        }
        let Some(nombre) = ruta.file_name().and_then(|n| n.to_str()).map(|s| s.to_string())
        else {
            continue;
        };
        let Ok(envio) = leer_envio(&nombre) else {
            continue;
        };
        if ahora < envio.listo_en {
            continue;
        }
        // Aprobar: mover a bibliotecas/ con su manifiesto.
        let Ok(destino) = carpeta_biblioteca(&nombre) else {
            continue;
        };
        if destino.exists() {
            // Ya existe una versión disponible: se queda la
            // vieja y se quita el envío (no se pisa nada).
            let _ = fs::remove_dir_all(&ruta);
            continue;
        }
        if fs::create_dir_all(&destino).is_err() {
            continue;
        }
        let Ok(fuente) = fs::read_to_string(ruta.join(format!("{}.ec", nombre))) else {
            continue;
        };
        let Ok(ruta_lib) = ruta_biblioteca(&nombre) else {
            continue;
        };
        if fs::write(&ruta_lib, &fuente).is_err() {
            continue;
        }
        let manifiesto = format!(
            "nombre = \"{}\"\nversion = \"{}\"\nautor = \"{}\"\ndescripcion = \"{}\"\ncategoria = \"{}\"\ncontacto = \"{}\"\nestado = \"disponible\"\nchecksum = {}\n",
            envio.nombre,
            envio.version,
            envio.autor,
            envio.descripcion,
            envio.categoria,
            envio.contacto,
            envio.checksum
        );
        let Ok(ruta_m) = ruta_manifiesto(&nombre) else {
            continue;
        };
        if fs::write(&ruta_m, manifiesto).is_err() {
            continue;
        }
        let _ = fs::remove_dir_all(&ruta);
        aprobadas.push(nombre);
    }
    aprobadas
}

// ---- Validar una librería antes de publicarla ----
// 1. Que pese menos de 500 KB.
// 2. Que tenga un nombre válido (sin espacios, sin símbolos
//    raros, sin palabras peligrosas ni palabras de Eclipse).
// 3. Que se pueda leer SOLO como Eclipse (lexer + parser):
//    cualquier otro lenguaje no se entiende y no se publica.
// 4. Que esté en español (sin palabras en inglés).
// 5. Que no haga cosas peligrosas (sin `comando`, sin leer ni
//    escribir fuera de su carpeta, sin nombre peligroso).
// 6. Que tenga al menos una función (nada de librerías
//    mal hechas y vacías).
pub fn validar_biblioteca(nombre: &str, fuente: &str) -> Result<Vec<Declaracion>, String> {
    if fuente.len() > MAX_TAMANO {
        return Err(format!(
            "la librería '{}' pesa más de 500 KB ({} bytes). Hazla más chiquita.",
            nombre,
            fuente.len()
        ));
    }
    validar_nombre(nombre)?;
    let tokens = Lexer::nuevo(fuente).tokenizar()?;
    for t in &tokens {
        if let TipoToken::Identificador(palabra) = &t.tipo {
            if PALABRAS_INGLES.contains(&palabra.as_str()) {
                return Err(format!(
                    "esta librería no está en español: encontré la palabra '{}' en la línea {}. Las librerías de Eclipse se escriben en español (usa 'sea' en vez de 'let', 'si' en vez de 'if', 'imprimir' en vez de 'print'...).",
                    palabra, t.linea
                ));
            }
        }
    }
    let declaraciones = Parser::nuevo(tokens).parsear()?;
    revisar_peligros(&declaraciones)?;
    Ok(declaraciones)
}

// El nombre tiene que valer: solo letras, números, _ o -,
// sin palabras peligrosas y sin palabras de Eclipse.
fn validar_nombre(nombre: &str) -> Result<(), String> {
    if nombre.is_empty() {
        return Err("la librería tiene que tener un nombre.".to_string());
    }
    if !nombre
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    {
        return Err(format!(
            "el nombre '{}' no vale: usa solo letras, números, _ o - (sin espacios ni símbolos raros).",
            nombre
        ));
    }
    let minus = nombre.to_lowercase();
    for mala in PALABRAS_PELIGROSAS {
        if minus.contains(mala) {
            return Err(format!(
                "la librería no se puede llamar '{}': las librerías no pueden ser firmware, virus, malware ni nada parecido.",
                nombre
            ));
        }
    }
    if PALABRAS_ECLIPSE.contains(&nombre) {
        return Err(format!(
            "el nombre '{}' no vale: es una palabra de Eclipse, no puede ser el nombre de una librería.",
            nombre
        ));
    }
    Ok(())
}

// Busca cosas peligrosas en el arbolito de la librería.
fn revisar_peligros(declaraciones: &[Declaracion]) -> Result<(), String> {
    for d in declaraciones {
        revisar_declaracion(d)?;
    }
    Ok(())
}

fn revisar_declaracion(d: &Declaracion) -> Result<(), String> {
    match d {
        Declaracion::Expresion { expresion: e, .. }
        | Declaracion::Devolver { valor: Some(e), .. } => revisar_expresion(e),
        Declaracion::Sea { valor: Some(e), .. }
        | Declaracion::Asignacion { valor: e, .. }
        | Declaracion::AsignacionCompuesta { valor: e, .. }
        | Declaracion::Constante { valor: e, .. } => revisar_expresion(e),
        // lista[0] = ... o mapa[k] = ...: el valor (y el
        // índice) pueden esconder una llamada peligrosa.
        Declaracion::AsignarIndice { indice, valor, .. } => {
            revisar_expresion(indice)?;
            revisar_expresion(valor)
        }
        // intentar/si_falla/al_final: revisar los tres
        // cuerpos, que si no se escapa un `comando`.
        Declaracion::Intentar { cuerpo, falla, al_final, .. } => {
            revisar_peligros(cuerpo)?;
            if let Some((_, cuerpo_falla)) = falla {
                revisar_peligros(cuerpo_falla)?;
            }
            if let Some(cuerpo_final) = al_final {
                revisar_peligros(cuerpo_final)?;
            }
            Ok(())
        }
        Declaracion::Si { condicion, entonces, sino, .. } => {
            revisar_expresion(condicion)?;
            revisar_peligros(entonces)?;
            if let Some(otro) = sino {
                revisar_peligros(otro)?;
            }
            Ok(())
        }
        Declaracion::Mientras { condicion, cuerpo, .. } => {
            revisar_expresion(condicion)?;
            revisar_peligros(cuerpo)
        }
        Declaracion::Repetir { cuerpo, condicion, .. } => {
            revisar_peligros(cuerpo)?;
            revisar_expresion(condicion)
        }
        Declaracion::Para { inicio, fin, paso, cuerpo, .. } => {
            revisar_expresion(inicio)?;
            revisar_expresion(fin)?;
            if let Some(p) = paso {
                revisar_expresion(p)?;
            }
            revisar_peligros(cuerpo)
        }
        Declaracion::ParaCada { lista, cuerpo, .. } => {
            revisar_expresion(lista)?;
            revisar_peligros(cuerpo)
        }
        Declaracion::Funcion { cuerpo, .. } => revisar_peligros(cuerpo),
        Declaracion::Segun { valor, brazos, .. } => {
            revisar_expresion(valor)?;
            for b in brazos {
                if let Some(p) = &b.patron {
                    revisar_expresion(p)?;
                }
                revisar_peligros(&b.cuerpo)?;
            }
            Ok(())
        }
        Declaracion::Bloque { declaraciones: ds, .. } => revisar_peligros(ds),
        _ => Ok(()),
    }
}

fn revisar_expresion(e: &crate::lenguaje::ast::Expresion) -> Result<(), String> {
    use crate::lenguaje::ast::Expresion;
    match e {
        Expresion::Llamada { nombre, argumentos } => {
            // `comando` corre programas externos: una librería
            // nunca lo puede usar (es la puerta de los virus).
            if nombre == "comando" {
                return Err(
                    "esta librería quiere correr comandos del sistema (comando). Por seguridad, las librerías no pueden correr programas externos."
                        .to_string(),
                );
            }
            // Leer, escribir, mover o borrar archivos solo
            // vale con rutas relativas (dentro de la carpeta
            // del programa): nada de rutas absolutas ni de
            // `..` para salir. Lo mismo para listar carpetas,
            // crear carpetas y revisar si algo existe.
            if nombre == "leer_archivo"
                || nombre == "escribir_archivo"
                || nombre == "mover_archivo"
                || nombre == "borrar_archivo"
                || nombre == "listar_carpeta"
                || nombre == "crear_carpeta"
                || nombre == "existe"
            {
                for a in argumentos {
                    if let Expresion::Cadena(ruta) = a {
                        if es_ruta_peligrosa(ruta) {
                            return Err(format!(
                                "esta librería quiere usar '{}', fuera de su carpeta. Las librerías solo pueden usar rutas relativas, sin '..' ni rutas absolutas.",
                                ruta
                            ));
                        }
                    }
                }
            }
            for a in argumentos {
                revisar_expresion(a)?;
            }
            Ok(())
        }
        Expresion::Binaria { izquierda, derecha, .. } => {
            revisar_expresion(izquierda)?;
            revisar_expresion(derecha)
        }
        Expresion::Unaria { derecha, .. } => revisar_expresion(derecha),
        Expresion::Agrupada(dentro) => revisar_expresion(dentro),
        Expresion::Interpolada(partes) => {
            for p in partes {
                if let crate::lenguaje::ast::ParteCadena::Expresion(dentro) = p {
                    revisar_expresion(dentro)?;
                }
            }
            Ok(())
        }
        Expresion::Lista(elementos) => {
            for el in elementos {
                revisar_expresion(el)?;
            }
            Ok(())
        }
        Expresion::Indice { lista, indice } => {
            revisar_expresion(lista)?;
            revisar_expresion(indice)
        }
        Expresion::Mapa(pares) => {
            for (clave, valor) in pares {
                revisar_expresion(clave)?;
                revisar_expresion(valor)?;
            }
            Ok(())
        }
        Expresion::Prestar(dentro) | Expresion::PrestarMut(dentro) => {
            revisar_expresion(dentro)
        }
        _ => Ok(()),
    }
}

fn es_ruta_peligrosa(ruta: &str) -> bool {
    ruta.contains("..")
        || ruta.starts_with('/')
        || ruta.starts_with('\\')
        || (ruta.len() >= 2 && ruta.as_bytes()[1] == b':' && ruta.as_bytes()[0].is_ascii_alphabetic())
}

// ---- Preguntar por terminal ----
// Hace una pregunta y lee la respuesta. Si no escribes
// nada (o no hay nada que leer), usa el valor por defecto.
fn preguntar(pregunta: &str, defecto: &str) -> String {
    print!("{} [{}]: ", pregunta, defecto);
    let _ = io::stdout().flush();
    let mut linea = String::new();
    match io::stdin().read_line(&mut linea) {
        Ok(_) => {
            let texto = linea.trim();
            if texto.is_empty() {
                defecto.to_string()
            } else {
                texto.to_string()
            }
        }
        Err(_) => defecto.to_string(),
    }
}

// ---- El límite de 5 horas ----

fn leer_ultima_subida() -> Option<u64> {
    let ruta = ruta_ultima_subida().ok()?;
    let texto = fs::read_to_string(&ruta).ok()?;
    texto.trim().parse().ok()
}

fn escribir_ultima_subida(segs: u64) -> Result<(), String> {
    let ruta = ruta_ultima_subida()?;
    if let Some(padre) = ruta.parent() {
        fs::create_dir_all(padre)
            .map_err(|_| "no pude crear el Registro de librerías".to_string())?;
    }
    fs::write(&ruta, segs.to_string())
        .map_err(|_| "no pude guardar cuándo subiste la última librería".to_string())
}

// ---- Comandos de biblioteca ----

// Publica (envía a revisión) una librería desde un archivo .ec.
// Te hace 6 preguntas por terminal: nombre de la librería,
// tu nombre, descripción, versión, categoría y contacto.
// Después de subirla, tarda de 1 a 2 días en llegar a Eclipse.
pub fn publicar(ruta_archivo: &str) -> Result<String, String> {
    migrar_registro();
    // Límite: solo se puede subir una librería cada 5 horas.
    let ahora = ahora_segs();
    if let Some(ultima) = leer_ultima_subida() {
        if ahora < ultima + LIMITE_SUBIDA_SEGS {
            let falta = ultima + LIMITE_SUBIDA_SEGS - ahora;
            return Err(format!(
                "ya subiste una librería hace poco. Solo se puede subir una librería cada 5 horas: te faltan {}.",
                falta_bonita(falta)
            ));
        }
    }
    let ruta = Path::new(ruta_archivo);
    let nombre_defecto = ruta
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| format!("no entiendo el nombre del archivo '{}'", ruta_archivo))?
        .to_string();
    let fuente = fs::read_to_string(ruta)
        .map_err(|_| format!("no encontré el archivo '{}'", ruta_archivo))?;
    // Primero el nombre, porque la revisión depende de él.
    let nombre = preguntar("¿Cómo se llama la librería?", &nombre_defecto);
    let declaraciones = validar_biblioteca(&nombre, &fuente)?;
    // Una librería sin funciones no sirve para nada.
    if !declaraciones
        .iter()
        .any(|d| matches!(d, Declaracion::Funcion { .. }))
    {
        return Err(
            "esta librería no tiene ninguna función. Una librería sirve para compartir funciones: escribe al menos una con 'funcion'."
                .to_string(),
        );
    }
    // El verificador de préstamos revisa la librería antes
    // de publicarla: si tiene un error de préstamos, no se
    // publica.
    let base = ruta
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."));
    if let Err(e) = crate::motor::verificador::verificar(&declaraciones, &base) {
        return Err(e.texto());
    }
    // Las otras 5 preguntas (solo si la librería pasó la revisión).
    let autor_defecto = env::var("USER")
        .or_else(|_| env::var("USERNAME"))
        .unwrap_or_else(|_| "?".to_string());
    let autor = preguntar("¿Cómo te llamas?", &autor_defecto);
    let descripcion = preguntar("¿Para qué sirve la librería?", "Biblioteca de Eclipse");
    let version = preguntar("¿Qué versión es?", "0.1.0");
    let categoria = preguntar(
        "¿De qué categoría es? (juegos, matematicas, dibujo, utilidades...)",
        "general",
    );
    let contacto = preguntar("¿Cómo te pueden contactar? (opcional)", "");
    // ¿Ya hay un envío con ese nombre en revisión?
    if carpeta_envio(&nombre)?.exists() {
        return Err(format!(
            "la librería '{}' ya está en revisión. Espera a que llegue antes de volver a subirla.",
            nombre
        ));
    }
    // Guardar en la cola de revisión.
    let carpeta = carpeta_envio(&nombre)?;
    fs::create_dir_all(&carpeta)
        .map_err(|_| format!("no pude crear la revisión de la librería '{}'", nombre))?;
    fs::write(ruta_envio(&nombre)?, &fuente)
        .map_err(|_| format!("no pude guardar la librería '{}'", nombre))?;
    let listo_en = ahora + ESPERA_REVISION_SEGS;
    let envio = format!(
        "nombre = \"{}\"\nversion = \"{}\"\nautor = \"{}\"\ndescripcion = \"{}\"\ncategoria = \"{}\"\ncontacto = \"{}\"\nenviado_en = {}\nlisto_en = {}\nchecksum = {}\n",
        nombre,
        version,
        autor,
        descripcion,
        categoria,
        contacto,
        ahora,
        listo_en,
        hash_simple(&fuente)
    );
    fs::write(ruta_envio_manifiesto(&nombre)?, envio)
        .map_err(|_| format!("no pude guardar los datos de la librería '{}'", nombre))?;
    escribir_ultima_subida(ahora)?;
    Ok(format!(
        "Librería '{}' enviada a revisión. Llega a Eclipse en 1 día y medio (entre 1 y 2 días, sin excepciones). Cuando esté lista la podrás usar con: usar {};",
        nombre, nombre
    ))
}

// Lista las librerías disponibles y las que están en revisión.
// Devuelve (nombre, versión, estado).
pub fn listar() -> Result<Vec<(String, String, String)>, String> {
    migrar_registro();
    let _ = revisar_cola();
    let mut lista = Vec::new();
    let carpeta = carpeta_bibliotecas()?;
    if carpeta.exists() {
        let entradas = fs::read_dir(&carpeta)
            .map_err(|_| "no pude leer el Registro de librerías".to_string())?;
        for entrada in entradas.flatten() {
            let ruta = entrada.path();
            if ruta.is_dir() {
                if let Some(nombre) = ruta.file_name().and_then(|n| n.to_str()) {
                    let version = leer_manifiesto(nombre)
                        .map(|m| m.version)
                        .unwrap_or_else(|_| "?".to_string());
                    lista.push((nombre.to_string(), version, "disponible".to_string()));
                }
            }
        }
    }
    let cola = carpeta_cola()?;
    if cola.exists() {
        let entradas = fs::read_dir(&cola)
            .map_err(|_| "no pude leer la cola de revisión".to_string())?;
        for entrada in entradas.flatten() {
            let ruta = entrada.path();
            if ruta.is_dir() {
                if let Some(nombre) = ruta.file_name().and_then(|n| n.to_str()) {
                    let estado = match leer_envio(nombre) {
                        Ok(e) => format!(
                            "en revisión, llega en {}",
                            falta_bonita(e.listo_en.saturating_sub(ahora_segs()))
                        ),
                        Err(_) => "en revisión".to_string(),
                    };
                    let version = leer_envio(nombre)
                        .map(|e| e.version)
                        .unwrap_or_else(|_| "?".to_string());
                    lista.push((nombre.to_string(), version, estado));
                }
            }
        }
    }
    lista.sort();
    Ok(lista)
}

// Los datos de una librería (disponible o en revisión).
pub fn info(nombre: &str) -> Result<String, String> {
    migrar_registro();
    let _ = revisar_cola();
    if ruta_biblioteca(nombre)?.exists() {
        let m = leer_manifiesto(nombre)?;
        let contacto = if m.contacto.is_empty() {
            "(sin contacto)".to_string()
        } else {
            m.contacto
        };
        return Ok(format!(
            "Librería '{}' (disponible)\n  Versión: {}\n  Autor: {}\n  Descripción: {}\n  Categoría: {}\n  Contacto: {}\n  Se usa con: usar {};",
            m.nombre, m.version, m.autor, m.descripcion, m.categoria, contacto, m.nombre
        ));
    }
    if let Ok(e) = leer_envio(nombre) {
        let contacto = if e.contacto.is_empty() {
            "(sin contacto)".to_string()
        } else {
            e.contacto
        };
        return Ok(format!(
            "Librería '{}' (en revisión, llega en {})\n  Versión: {}\n  Autor: {}\n  Descripción: {}\n  Categoría: {}\n  Contacto: {}\n  Se usará con: usar {};",
            e.nombre,
            falta_bonita(e.listo_en.saturating_sub(ahora_segs())),
            e.version,
            e.autor,
            e.descripcion,
            e.categoria,
            contacto,
            e.nombre
        ));
    }
    Err(format!("no encontré la librería '{}'", nombre))
}

// El estado de una librería: disponible o en revisión.
pub fn estado(nombre: &str) -> Result<String, String> {
    migrar_registro();
    let _ = revisar_cola();
    if ruta_biblioteca(nombre)?.exists() {
        let m = leer_manifiesto(nombre)?;
        return Ok(format!(
            "La librería '{}' está disponible (versión {}, por {}). Se usa con: usar {};",
            m.nombre, m.version, m.autor, m.nombre
        ));
    }
    if let Ok(e) = leer_envio(nombre) {
        return Ok(format!(
            "La librería '{}' está en revisión: llega a Eclipse en {}.",
            e.nombre,
            falta_bonita(e.listo_en.saturating_sub(ahora_segs()))
        ));
    }
    Err(format!("no encontré la librería '{}'", nombre))
}

// Quita una librería disponible, o cancela un envío en revisión.
pub fn quitar(nombre: &str) -> Result<String, String> {
    migrar_registro();
    let carpeta = carpeta_biblioteca(nombre)?;
    if carpeta.exists() {
        fs::remove_dir_all(&carpeta)
            .map_err(|_| format!("no pude quitar la librería '{}'", nombre))?;
        return Ok(format!("Librería '{}' quitada.", nombre));
    }
    let envio = carpeta_envio(nombre)?;
    if envio.exists() {
        fs::remove_dir_all(&envio)
            .map_err(|_| format!("no pude quitar la librería '{}'", nombre))?;
        return Ok(format!(
            "Librería '{}' quitada de la revisión: ya no va a llegar a Eclipse.",
            nombre
        ));
    }
    Err(format!("no encontré la librería '{}'", nombre))
}

// Las funciones de una librería (nombre y cuántos datos pide
// cada una), para que el verificador sepa que existen.
pub fn firmas_biblioteca(nombre: &str) -> Result<Vec<(String, usize)>, String> {
    let ruta = resolver_biblioteca(nombre)?;
    let fuente = fs::read_to_string(&ruta)
        .map_err(|_| format!("no pude leer la librería '{}'", nombre))?;
    let tokens = Lexer::nuevo(&fuente).tokenizar()?;
    let declaraciones = Parser::nuevo(tokens).parsear()?;
    let mut firmas = Vec::new();
    for d in &declaraciones {
        if let Declaracion::Funcion { nombre: f, parametros, .. } = d {
            firmas.push((f.clone(), parametros.len()));
        }
    }
    Ok(firmas)
}
