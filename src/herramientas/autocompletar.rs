// ============================================================
//  ECLIPSE 1.2 - Autocompletado inteligente
//  Archivo: src/herramientas/autocompletar.rs
//
//  Qué hace:
//  Lee un archivo .ec y te dice qué puedes escribir en una
//  posición (línea, columna). Eso es lo que usa la extensión
//  de VS Code para sugerirte mientras escribes.
//
//  Qué sugiere:
//  - Las variables que se ven desde el cursor, con su tipo
//    si se conoce (de `sea x: numero` o del valor que le
//    diste: `sea x = 5` es un numero).
//  - Las funciones que definiste, con cuántos datos necesitan.
//  - Las 34 palabras clave y las funciones que trae Eclipse.
//
//  Cómo lo sabe:
//  Trabaja con el lexer (los tokens), no con el parser, así
//  que funciona aunque el archivo tenga errores o esté a
//  medias (que es justo cuando escribes). Va recorriendo los
//  tokens en orden, abre y cierra ámbitos con { y }, y anota
//  cada `sea` y cada `funcion` con el ámbito donde vive. Al
//  llegar al cursor, las variables visibles son las que están
//  en un ámbito que sigue abierto.
//
//  Se usa así:
//      eclipse --completar programa.ec 5 10
//  Y responde una sugerencia por línea:
//      nombre<TAB>tipo<TAB>descripción
// ============================================================

use crate::lenguaje::lexer::{Lexer, TipoToken, Token};
use std::collections::HashSet;

// Una sugerencia: qué escribir, de qué tipo es y qué hace.
#[derive(Debug, Clone)]
pub struct Completado {
    pub nombre: String,
    pub tipo: String,
    pub descripcion: String,
}

// Una variable o función que se definió en el archivo.
// `linea` es la línea donde se definió (empieza en 1).
struct Definicion {
    nombre: String,
    tipo: String,
    descripcion: String,
    ambito: usize,
    linea: usize,
}

// Un símbolo para el esquema del documento (LSP
// documentSymbol): su nombre, su clase (12 = función,
// 13 = variable) y la línea donde se definió.
#[derive(Debug, Clone)]
pub struct Simbolo {
    pub nombre: String,
    pub kind: u8,
    pub linea: usize,
}

// Las 34 palabras clave, con su explicación en español.
fn palabras_clave() -> Vec<(&'static str, &'static str)> {
    vec![
        ("sea", "Crea una variable fija"),
        ("mut", "La variable sí se puede cambiar"),
        ("si", "Si pasa esto..."),
        ("sino", "Si no pasa..."),
        ("mientras", "Repite mientras algo sea verdad"),
        ("para", "Repite con contador"),
        ("en", "Dentro de un para"),
        ("paso", "El paso de un para"),
        ("repetir", "Repite el cuerpo y luego revisa la condición"),
        ("funcion", "Crea una función"),
        ("devolver", "Regresa un valor de una función"),
        ("verdadero", "Sí"),
        ("falso", "No"),
        ("y", "Y lógico"),
        ("o", "O lógico"),
        ("no", "Negar"),
        ("romper", "Sale del bucle"),
        ("seguir", "Salta a la siguiente vuelta"),
        ("nulo", "Nada, vacío"),
        ("importar", "Carga otro archivo .ec"),
        ("numero", "Tipo: número"),
        ("texto", "Tipo: texto"),
        ("logico", "Tipo: verdadero/falso"),
        ("segun", "Elige según un valor"),
        ("leer", "Lee lo que escribe el usuario y lo guarda"),
        ("lista", "Tipo: lista de valores"),
        ("para_cada", "Repite con cada elemento de una lista"),
        ("constante", "Crea una constante (nunca cambia)"),
        ("usar", "Usar una librería (solo arriba del archivo)"),
        ("mapa", "Tipo: mapa de claves y valores"),
        ("intentar", "Intenta algo y atrapa el error si falla"),
        ("si_falla", "Qué hacer si el intentar falló"),
        ("al_final", "Qué hacer siempre, pase lo que pase"),
    ]
}

// Las funciones que trae Eclipse, con su explicación.
fn builtins() -> Vec<(&'static str, &'static str)> {
    vec![
        ("imprimir", "Muestra valores en pantalla"),
        ("leer", "Lee lo que escribe el usuario"),
        ("leer_numero", "Lee un número que escribe el usuario"),
        ("a_numero", "Convierte texto a número"),
        ("a_texto", "Convierte un valor a texto"),
        ("leer_archivo", "Lee un archivo y devuelve su texto"),
        ("escribir_archivo", "Escribe texto en un archivo"),
        ("largo", "Cuántas letras tiene un texto"),
        ("tipo_de", "El tipo: numero, texto, logico o nulo"),
        ("aleatorio", "Un número al azar entre 0 y 99"),
        ("copiar", "Una copia nueva del texto"),
        ("min", "El más chico de dos números"),
        ("max", "El más grande de dos números"),
        ("agregar", "Devuelve una lista nueva con un valor al final"),
        ("elemento", "El elemento de una lista en un lugar"),
        ("ahora", "Los segundos desde 1970 (el reloj)"),
        ("redondear", "Redondea un número (3.7 -> 4)"),
        ("absoluto", "El valor sin el signo (-5 -> 5)"),
        ("mayusculas", "El texto en MAYÚSCULAS"),
        ("minusculas", "El texto en minúsculas"),
        ("contiene", "Verdadero si el texto tiene esa parte"),
        ("ordenar", "Una lista NUEVA ordenada (números o textos)"),
        ("invertir", "Una lista o un texto al revés"),
        ("unir", "Pega una lista con un separador"),
        ("separar", "Parte un texto en una lista"),
        ("raiz", "La raíz cuadrada de un número"),
        ("sistema", "Qué sistema operativo es (Windows, Linux, macOS)"),
        ("usuario", "Tu nombre de usuario del sistema"),
        ("carpeta", "La carpeta donde estás parado"),
        ("variable_entorno", "El valor de una variable de entorno"),
        ("comando", "Corre otro programa y te da lo que imprimió"),
        ("argumentos", "La lista de datos que le diste al programa"),
        ("procesadores", "Cuántos núcleos tiene la computadora"),
        ("memoria", "Cuánta memoria total tiene, en MB"),
        ("caracter", "El carácter de un número (caracter(65) -> A)"),
        ("claves", "Las claves de un mapa, en una lista"),
        ("valores", "Los valores de un mapa, en una lista"),
        ("tiene", "Verdadero si el mapa tiene esa clave"),
        ("fecha", "La fecha de hoy (UTC): año-mes-día"),
        ("hora", "La hora de ahora (UTC): horas:minutos:segundos"),
        ("fecha_hora", "La fecha y la hora de ahora (UTC)"),
        ("dormir", "Espera esos segundos sin hacer nada"),
        ("existe", "Verdadero si ese archivo o carpeta existe"),
        ("listar_carpeta", "Los archivos de una carpeta, en orden"),
        ("crear_carpeta", "Crea una carpeta (y las que falten)"),
        ("mover_archivo", "Mueve o cambia el nombre de un archivo"),
        ("borrar_archivo", "Borra un archivo"),
        ("reemplazar", "Cambia un texto por otro, todas las veces"),
    ]
}

// La palabra que se está escribiendo en (linea, columna):
// los caracteres de nombre que hay justo antes del cursor.
fn prefijo(fuente: &str, linea: usize, columna: usize) -> String {
    let lineas: Vec<&str> = fuente.lines().collect();
    if linea == 0 || linea > lineas.len() {
        return String::new();
    }
    let texto = lineas[linea - 1];
    let antes: String = texto.chars().take(columna.saturating_sub(1)).collect();
    let mut prefijo = String::new();
    for c in antes.chars().rev() {
        if c.is_alphanumeric() || c == '_' {
            prefijo.insert(0, c);
        } else {
            break;
        }
    }
    prefijo
}

// ¿Está este token antes del cursor?
fn antes_del_cursor(t: &Token, linea: usize, columna: usize) -> bool {
    t.linea < linea || (t.linea == linea && t.columna < columna)
}

// Recorre los tokens hasta el cursor y anota qué variables
// y funciones se definieron y en qué ámbitos. Devuelve las
// definiciones y los ámbitos que siguen abiertos en el cursor.
// Si el archivo tiene un error de lexer, devuelve listas vacías.
fn recolectar(fuente: &str, linea: usize, columna: usize) -> (Vec<Definicion>, Vec<usize>) {
    let tokens = match Lexer::nuevo(fuente).tokenizar() {
        Ok(t) => t,
        Err(_) => return (Vec::new(), vec![0]),
    };
    // La pila de ámbitos abiertos. El 0 es el de todo el archivo.
    let mut ambitos: Vec<usize> = vec![0];
    let mut siguiente_id: usize = 1;
    let mut definiciones: Vec<Definicion> = Vec::new();
    // La variable de un `para` o `para_cada`, que se
    // registra cuando se abre la llave del cuerpo (con la
    // línea donde se escribió el bucle).
    let mut variable_pendiente: Option<(String, usize)> = None;
    // Los datos de una `funcion`, que se registran cuando se
    // abre la llave del cuerpo (con sus líneas).
    let mut parametros_pendientes: Vec<(String, usize)> = Vec::new();
    let mut i = 0;
    while i < tokens.len() {
        let t = &tokens[i];
        // Solo miramos los tokens que están antes del cursor.
        if !antes_del_cursor(t, linea, columna) {
            break;
        }
        match &t.tipo {
            TipoToken::LlaveIzq => {
                let id = siguiente_id;
                siguiente_id += 1;
                ambitos.push(id);
                if let Some((nombre, linea)) = variable_pendiente.take() {
                    definiciones.push(Definicion {
                        nombre,
                        tipo: "variable".to_string(),
                        descripcion: "variable del bucle".to_string(),
                        ambito: id,
                        linea,
                    });
                }
                for (nombre, linea) in parametros_pendientes.drain(..) {
                    definiciones.push(Definicion {
                        nombre,
                        tipo: "variable".to_string(),
                        descripcion: "dato de la función".to_string(),
                        ambito: id,
                        linea,
                    });
                }
            }
            TipoToken::LlaveDer => {
                if ambitos.len() > 1 {
                    ambitos.pop();
                }
            }
            TipoToken::Sea | TipoToken::Constante => {
                // sea [mut] nombre [: tipo] [= valor]
                // constante nombre [: tipo] = valor
                let linea_def = t.linea;
                let es_constante = matches!(t.tipo, TipoToken::Constante);
                let mut j = i + 1;
                if j < tokens.len() && matches!(tokens[j].tipo, TipoToken::Mut) {
                    j += 1;
                }
                if j < tokens.len() {
                    if let TipoToken::Identificador(nombre) = &tokens[j].tipo {
                        if !antes_del_cursor(&tokens[j], linea, columna) {
                            i += 1;
                            continue;
                        }
                        let nombre = nombre.clone();
                        let mut tipo = "variable".to_string();
                        let mut k = j + 1;
                        // : tipo
                        if k < tokens.len() && matches!(tokens[k].tipo, TipoToken::DosPuntos) {
                            k += 1;
                            if k < tokens.len() {
                                tipo = match &tokens[k].tipo {
                                    TipoToken::Numero => "numero".to_string(),
                                    TipoToken::Texto => "texto".to_string(),
                                    TipoToken::Logico => "logico".to_string(),
                                    TipoToken::Lista => "lista".to_string(),
                                    _ => "variable".to_string(),
                                };
                                k += 1;
                            }
                        }
                        // = valor (para adivinar el tipo)
                        if tipo == "variable"
                            && k < tokens.len()
                            && matches!(tokens[k].tipo, TipoToken::Igual)
                        {
                            k += 1;
                            if k < tokens.len() {
                                tipo = match &tokens[k].tipo {
                                    TipoToken::NumeroLit(_) => "numero".to_string(),
                                    TipoToken::Cadena(_) => "texto".to_string(),
                                    TipoToken::Verdadero | TipoToken::Falso => {
                                        "logico".to_string()
                                    }
                                    TipoToken::CorcheteIzq => "lista".to_string(),
                                    _ => "variable".to_string(),
                                };
                            }
                        }
                        definiciones.push(Definicion {
                            nombre,
                            tipo,
                            descripcion: if es_constante {
                                "constante".to_string()
                            } else {
                                "variable".to_string()
                            },
                            ambito: ambitos.last().copied().unwrap_or(0),
                            linea: linea_def,
                        });
                    }
                }
            }
            TipoToken::Funcion => {
                let linea_def = t.linea;
                let j = i + 1;
                if j < tokens.len() {
                    if let TipoToken::Identificador(nombre) = &tokens[j].tipo {
                        if !antes_del_cursor(&tokens[j], linea, columna) {
                            i += 1;
                            continue;
                        }
                        let nombre = nombre.clone();
                        // Contar los parámetros entre ( y ) y
                        // apuntarlos para el cuerpo de la función.
                        let mut params = 0;
                        let mut k = j + 1;
                        if k < tokens.len() && matches!(tokens[k].tipo, TipoToken::ParenIzq) {
                            k += 1;
                            while k < tokens.len()
                                && !matches!(tokens[k].tipo, TipoToken::ParenDer)
                            {
                                if let TipoToken::Identificador(p) = &tokens[k].tipo {
                                    params += 1;
                                    parametros_pendientes
                                        .push((p.clone(), tokens[k].linea));
                                }
                                k += 1;
                            }
                        }
                        definiciones.push(Definicion {
                            nombre,
                            tipo: format!(
                                "función ({} {})",
                                params,
                                if params == 1 { "dato" } else { "datos" }
                            ),
                            descripcion: "función que definiste".to_string(),
                            ambito: ambitos.last().copied().unwrap_or(0),
                            linea: linea_def,
                        });
                    }
                }
            }
            TipoToken::Para | TipoToken::ParaCada => {
                // para nombre en ... { ... }
                // para_cada nombre en lista { ... }
                let j = i + 1;
                if j < tokens.len() {
                    if let TipoToken::Identificador(nombre) = &tokens[j].tipo {
                        if antes_del_cursor(&tokens[j], linea, columna) {
                            variable_pendiente = Some((nombre.clone(), t.linea));
                        }
                    }
                }
            }
            _ => {}
        }
        i += 1;
    }
    (definiciones, ambitos)
}

// Dice qué puedes escribir en (linea, columna) de un archivo.
// La línea y la columna empiezan en 1, como en VS Code.
pub fn completar(fuente: &str, linea: usize, columna: usize) -> Vec<Completado> {
    let prefijo = prefijo(fuente, linea, columna);
    let (definiciones, ambitos) = recolectar(fuente, linea, columna);

    let mut resultado: Vec<Completado> = Vec::new();
    // Primero las variables y funciones que se ven desde
    // el cursor (las que están en un ámbito abierto).
    for d in &definiciones {
        if ambitos.contains(&d.ambito) && d.nombre.starts_with(&prefijo) {
            resultado.push(Completado {
                nombre: d.nombre.clone(),
                tipo: d.tipo.clone(),
                descripcion: d.descripcion.clone(),
            });
        }
    }
    // Luego las palabras clave.
    for (nombre, descripcion) in palabras_clave() {
        if nombre.starts_with(&prefijo) {
            resultado.push(Completado {
                nombre: nombre.to_string(),
                tipo: "palabra clave".to_string(),
                descripcion: descripcion.to_string(),
            });
        }
    }
    // Y las funciones que trae Eclipse.
    for (nombre, descripcion) in builtins() {
        if nombre.starts_with(&prefijo) {
            resultado.push(Completado {
                nombre: nombre.to_string(),
                tipo: "función".to_string(),
                descripcion: descripcion.to_string(),
            });
        }
    }
    // Sin duplicados (por nombre): la variable gana.
    let mut vistos = HashSet::new();
    resultado.retain(|c| vistos.insert(c.nombre.clone()));
    resultado
}

// Qué es una palabra en (linea, columna): una variable, una
// función, una palabra clave o una función de Eclipse.
// Es lo que muestra el hover del LSP cuando pasas el ratón
// por encima de una palabra. Si no se sabe, devuelve None.
pub fn describir(fuente: &str, linea: usize, columna: usize, palabra: &str) -> Option<Completado> {
    let (definiciones, ambitos) = recolectar(fuente, linea, columna);
    // La definición visible más reciente con ese nombre gana
    // (una variable de adentro tapa a una de afuera).
    for d in definiciones.iter().rev() {
        if d.nombre == palabra && ambitos.contains(&d.ambito) {
            return Some(Completado {
                nombre: d.nombre.clone(),
                tipo: d.tipo.clone(),
                descripcion: d.descripcion.clone(),
            });
        }
    }
    for (nombre, descripcion) in palabras_clave() {
        if nombre == palabra {
            return Some(Completado {
                nombre: nombre.to_string(),
                tipo: "palabra clave".to_string(),
                descripcion: descripcion.to_string(),
            });
        }
    }
    for (nombre, descripcion) in builtins() {
        if nombre == palabra {
            return Some(Completado {
                nombre: nombre.to_string(),
                tipo: "función".to_string(),
                descripcion: descripcion.to_string(),
            });
        }
    }
    None
}

// El esquema del documento: todas las variables, constantes
// y funciones que se definieron en el archivo, con la línea
// donde se definieron. Es lo que usa el LSP (documentSymbol)
// para el esquema que ves en tu editor. Se recolecta con el
// cursor al final, para que salga todo el archivo.
pub fn simbolos(fuente: &str) -> Vec<Simbolo> {
    let (definiciones, _) = recolectar(fuente, usize::MAX, usize::MAX);
    definiciones
        .iter()
        .map(|d| Simbolo {
            nombre: d.nombre.clone(),
            // 12 = función, 13 = variable (en LSP).
            kind: if d.descripcion == "función que definiste" {
                12
            } else {
                13
            },
            linea: d.linea,
        })
        .collect()
}

// Todos los lugares donde aparece una palabra, con su
// línea y su columna (base 1, la columna donde empieza).
// Sirve para las referencias del LSP: saltar a cada uso.
// Si el lexer falla, no hay lugares.
pub fn ocurrencias(fuente: &str, palabra: &str) -> Vec<(usize, usize)> {
    let tokens = match crate::lenguaje::lexer::Lexer::nuevo(fuente).tokenizar() {
        Ok(t) => t,
        Err(_) => return Vec::new(),
    };
    let mut lugares = Vec::new();
    for token in tokens {
        // El nombre detrás del token: un identificador, o una
        // palabra nueva usada como nombre de variable.
        let nombre = match &token.tipo {
            crate::lenguaje::lexer::TipoToken::Identificador(n) => n.clone(),
            crate::lenguaje::lexer::TipoToken::Lista => "lista".to_string(),
            crate::lenguaje::lexer::TipoToken::ParaCada => "para_cada".to_string(),
            crate::lenguaje::lexer::TipoToken::Constante => "constante".to_string(),
            crate::lenguaje::lexer::TipoToken::Leer => "leer".to_string(),
            _ => continue,
        };
        if nombre == palabra {
            // La columna del token es la de DESPUÉS del token:
            // el inicio es la columna menos el largo del nombre.
            let inicio = token
                .columna
                .saturating_sub(nombre.chars().count());
            lugares.push((token.linea, inicio));
        }
    }
    lugares
}

// La palabra que hay en (linea, columna): los caracteres
// de nombre alrededor del cursor.
pub fn palabra_en(fuente: &str, linea: usize, columna: usize) -> String {
    let lineas: Vec<&str> = fuente.lines().collect();
    if linea == 0 || linea > lineas.len() {
        return String::new();
    }
    let texto = lineas[linea - 1];
    let chars: Vec<char> = texto.chars().collect();
    let pos = columna.saturating_sub(1).min(chars.len());
    let mut inicio = pos;
    while inicio > 0 && (chars[inicio - 1].is_alphanumeric() || chars[inicio - 1] == '_') {
        inicio -= 1;
    }
    let mut fin = pos;
    while fin < chars.len() && (chars[fin].is_alphanumeric() || chars[fin] == '_') {
        fin += 1;
    }
    chars[inicio..fin].iter().collect()
}
