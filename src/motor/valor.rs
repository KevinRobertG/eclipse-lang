// ============================================================
//  ECLIPSE 1.2 - Los valores
//  Archivo: src/valor.rs
//
//  Qué hace:
//  Un valor en Eclipse (número, texto, verdadero/falso,
//  nulo o lista) y las operaciones entre dos valores
//  (+, -, *, /, %, comparaciones). La máquina virtual,
//  el compilador (para doblar constantes) y el paquete
//  (para guardar el bytecode) usan este mismo archivo,
//  así que los tres dicen exactamente lo mismo.
//
//  Ownership (como Rust): cada valor tiene UN solo dueño.
//  Los números y los lógicos son fotocopias: no se mueven
//  nunca. Los textos y las listas son originales: se
//  mueven al pasarlos a otra variable o a una función, y
//  `copiar()` hace una copia nueva.
// ============================================================

use crate::lenguaje::ast::OperadorBinario;
use std::sync::Arc;

// Un valor en Eclipse: número, texto, verdadero/falso, nulo o lista.
#[derive(Debug, Clone, PartialEq)]
pub enum Valor {
    Numero(f64),
    Texto(String),
    Logico(bool),
    Nulo,
    // Una lista: varios valores en orden, como el `Vec` de Rust.
    // Las listas son valores con dueño (como los textos): se
    // mueven al pasarlas a otra variable o a una función, y
    // `copiar()` hace una copia nueva.
    // La lista vive dentro de un Arc: copiar una lista (al
    // leerla de una variable, o al pasarla a una función) solo
    // sube un contador, no copia los elementos. Solo se copia
    // de verdad cuando alguien la cambia y hay más de un
    // dueño (como hace `agregar` con Arc::make_mut).
    // Es un Arc (y no un Rc): copiar una lista solo sube
    // un contador, y `agregar` la copia de verdad solo
    // cuando hay más de un dueño (Arc::make_mut).
    Lista(Arc<Vec<Valor>>),
    // Un mapa: pares clave-valor en orden, como el
    // diccionario de Python. La clave es un número, un
    // texto o verdadero/falso. También se mueve como los
    // textos y las listas (es un original, no una copia).
    Mapa(Arc<Vec<(Valor, Valor)>>),
}

impl Valor {
    // Crea una lista con sus elementos (los envuelve en Arc).
    pub fn lista(elementos: Vec<Valor>) -> Valor {
        Valor::Lista(Arc::new(elementos))
    }

    // Crea un mapa con sus pares clave-valor (en orden).
    pub fn mapa(pares: Vec<(Valor, Valor)>) -> Valor {
        Valor::Mapa(Arc::new(pares))
    }

    // Busca una clave en un mapa: en qué lugar está, si es
    // que está. Las claves se comparan con ==.
    pub fn buscar_clave(pares: &[(Valor, Valor)], clave: &Valor) -> Option<usize> {
        pares.iter().position(|(k, _)| k == clave)
    }

    // El nombre del tipo, en español.
    pub fn tipo(&self) -> &'static str {
        match self {
            Valor::Numero(_) => "numero",
            Valor::Texto(_) => "texto",
            Valor::Logico(_) => "logico",
            Valor::Nulo => "nulo",
            Valor::Lista(_) => "lista",
            Valor::Mapa(_) => "mapa",
        }
    }

    // Convierte el valor a texto para mostrarlo.
    // Los números enteros se muestran sin .0: 5 y no 5.0.
    // Una lista se muestra entre corchetes: [1, 2, 3].
    pub fn a_texto(&self) -> String {
        match self {
            Valor::Numero(n) => {
                if n.fract() == 0.0 && n.is_finite() {
                    format!("{}", *n as i64)
                } else {
                    format!("{}", n)
                }
            }
            Valor::Texto(s) => s.clone(),
            Valor::Logico(b) => {
                if *b {
                    "verdadero".to_string()
                } else {
                    "falso".to_string()
                }
            }
            Valor::Nulo => "nulo".to_string(),
            Valor::Lista(elementos) => {
                let textos: Vec<String> = elementos.iter().map(|v| v.a_texto()).collect();
                format!("[{}]", textos.join(", "))
            }
            Valor::Mapa(pares) => {
                let textos: Vec<String> = pares
                    .iter()
                    .map(|(k, v)| format!("{}: {}", mostrar_clave(k), v.a_texto()))
                    .collect();
                format!("{{{}}}", textos.join(", "))
            }
        }
    }

    // ¿Este valor cuenta como verdadero en un `si` o `mientras`?
    // Solo `falso`, `nulo` y una lista vacía son falsos.
    // Todo lo demás es verdadero.
    pub fn es_verdadero(&self) -> bool {
        match self {
            Valor::Logico(b) => *b,
            Valor::Nulo => false,
            Valor::Lista(elementos) => !elementos.is_empty(),
            Valor::Mapa(pares) => !pares.is_empty(),
            _ => true,
        }
    }
}

pub fn aplicar_binario(izq: Valor, operador: &OperadorBinario, der: Valor) -> Result<Valor, String> {
    match operador {
        OperadorBinario::Igual => Ok(Valor::Logico(izq == der)),
        OperadorBinario::Diferente => Ok(Valor::Logico(izq != der)),
        OperadorBinario::Sumar => match (&izq, &der) {
            (Valor::Numero(a), Valor::Numero(b)) => Ok(Valor::Numero(a + b)),
            // Sumar dos listas las pega: [1, 2] + [3] es [1, 2, 3].
            (Valor::Lista(a), Valor::Lista(b)) => {
                let mut resultado = (**a).clone();
                resultado.extend(b.iter().cloned());
                Ok(Valor::lista(resultado))
            }
            // Sumar textos los pega: "Hola, " + "Kevin"
            _ => Ok(Valor::Texto(format!(
                "{}{}",
                izq.a_texto(),
                der.a_texto()
            ))),
        },
        OperadorBinario::Restar => numeros(izq, der, |a, b| a - b, "-"),
        OperadorBinario::Multiplicar => numeros(izq, der, |a, b| a * b, "*"),
        OperadorBinario::Dividir => {
            if matches!(der, Valor::Numero(n) if n == 0.0) {
                return Err("No se puede dividir entre cero".to_string());
            }
            numeros(izq, der, |a, b| a / b, "/")
        }
        OperadorBinario::Residuo => {
            // El resto entre cero no existe (igual que dividir
            // entre cero): da un error en español, no un NaN.
            if matches!(der, Valor::Numero(n) if n == 0.0) {
                return Err("No se puede calcular el resto entre cero".to_string());
            }
            numeros(izq, der, |a, b| a % b, "%")
        }
        // La potencia: 2 ** 3 es 8.
        OperadorBinario::Potencia => numeros(izq, der, |a, b| a.powf(b), "**"),
        OperadorBinario::Menor => comparar(izq, der, "<", |a, b| a < b),
        OperadorBinario::MenorIgual => comparar(izq, der, "<=", |a, b| a <= b),
        OperadorBinario::Mayor => comparar(izq, der, ">", |a, b| a > b),
        OperadorBinario::MayorIgual => comparar(izq, der, ">=", |a, b| a >= b),
        OperadorBinario::Y | OperadorBinario::O => unreachable!(),
    }
}

// Una operación con dos números, o un error en español.
fn numeros<F>(izq: Valor, der: Valor, f: F, simbolo: &str) -> Result<Valor, String>
where
    F: Fn(f64, f64) -> f64,
{
    match (izq, der) {
        (Valor::Numero(a), Valor::Numero(b)) => Ok(Valor::Numero(f(a, b))),
        (a, b) => Err(format!(
            "no puedo usar '{}' con {} y {}",
            simbolo,
            a.tipo(),
            b.tipo()
        )),
    }
}

// Una comparación entre dos números, o un error en español.
fn comparar<F>(izq: Valor, der: Valor, simbolo: &str, f: F) -> Result<Valor, String>
where
    F: Fn(f64, f64) -> bool,
{
    match (izq, der) {
        (Valor::Numero(a), Valor::Numero(b)) => Ok(Valor::Logico(f(a, b))),
        (a, b) => Err(format!(
            "solo puedo comparar numeros con '{}', no {} con {}",
            simbolo,
            a.tipo(),
            b.tipo()
        )),
    }
}

// Cómo se ve una clave de un mapa: los textos van entre
// comillas (para que "21" no se confunda con 21), los
// números y verdadero/falso se ven como siempre.
fn mostrar_clave(clave: &Valor) -> String {
    match clave {
        Valor::Texto(s) => format!("\"{}\"", s),
        otro => otro.a_texto(),
    }
}
