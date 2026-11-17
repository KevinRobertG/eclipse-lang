// ============================================================
//  ECLIPSE 1.2 - El arbolito (AST)
//  Archivo: src/ast.rs
//
//  Qué es esto:
//  El AST es el arbolito que arma el parser con tu programa.
//  Cada rama del árbol es una orden o un valor.
//
//  Ejemplo, para este programa:
//      si nombre == "Kevin" {
//          imprimir("Hola, Kevin");
//      }
//
//  El arbolito queda así:
//      Programa
//      └── Si (nombre == "Kevin")
//          └── Imprimir("Hola, Kevin")
//
//  En Rust, el arbolito se guarda con `enum`, porque cada
//  rama puede tener una forma distinta: un número, un texto,
//  una suma, un `si`, una función...
//  `Box` es una caja que guarda una rama dentro de otra rama:
//  el árbol tiene ramas dentro de ramas.
// ============================================================

// Los operadores de dos datos:  10 + 5,  a == b,  x y z,  2 ** 3
#[derive(Debug, Clone, PartialEq)]
pub enum OperadorBinario {
    Sumar,       // +
    Restar,      // -
    Multiplicar, // *
    Dividir,     // /
    Residuo,     // %
    Potencia,    // **  (2 ** 3 es 8)
    Igual,       // ==
    Diferente,   // !=
    Menor,       // <
    MenorIgual,  // <=
    Mayor,       // >
    MayorIgual,  // >=
    Y,           // y
    O,           // o
}

// Los operadores de un solo dato:  -5,  no verdadero
#[derive(Debug, Clone, PartialEq)]
pub enum OperadorUnario {
    Negativo, // -
    No,       // no
}

// Una expresión es algo que tiene un valor:
// un número, un texto, una suma, una llamada a función...
#[derive(Debug, Clone)]
pub enum Expresion {
    Numero(f64),
    Cadena(String),
    Logico(bool),
    Nulo,
    Variable(String),
    Agrupada(Box<Expresion>), // ( ... )
    Unaria {
        operador: OperadorUnario,
        derecha: Box<Expresion>,
    },
    Binaria {
        izquierda: Box<Expresion>,
        operador: OperadorBinario,
        derecha: Box<Expresion>,
    },
    Llamada {
        nombre: String,
        argumentos: Vec<Expresion>,
    },
    // "Hola, {nombre}, tienes {puntos} puntos"
    // Un texto con partes: trozos fijos y expresiones
    // que se calculan al correr el programa.
    Interpolada(Vec<ParteCadena>),
    // [1, 2, 3]  una lista: varios valores en orden.
    // Las listas son como el `Vec` de Rust: crecen y se
    // pueden recorrer con `para_cada`.
    Lista(Vec<Expresion>),
    // {21: "FTP", 22: "SSH"}  un mapa: pares clave-valor.
    // La clave es un número, un texto o verdadero/falso.
    // Se lee con corchetes: puertos[21] es "FTP".
    // Como el diccionario de Python, pero en español.
    Mapa(Vec<(Expresion, Expresion)>),
    // mi_lista[0]  el elemento de una lista en esa posición.
    // Empieza en 0: el primero es [0], el segundo es [1].
    Indice {
        lista: Box<Expresion>,
        indice: Box<Expresion>,
    },
    // &nombre  (prestar, no mover)
    // Se usa al llamar a una función: saludar(&nombre)
    // le presta el texto sin que la función se lo quede.
    Prestar(Box<Expresion>),
    // &mut nombre  (prestar para cambiar)
    // Se usa al llamar a una función: cambiar(&mut puntos)
    // le presta el número para que la función lo cambie, y
    // cuando la función termina, el valor nuevo vuelve a tu
    // variable. Solo sirve para números y lógicos (que son
    // fotocopias); para textos da un error.
    PrestarMut(Box<Expresion>),
}

// Una parte de un texto interpolado: un trozo fijo,
// o una expresión que se calcula y se pega como texto.
#[derive(Debug, Clone)]
pub enum ParteCadena {
    Texto(String),
    Expresion(Box<Expresion>),
}

// Una declaración es una orden completa del programa:
// crear una variable, un `si`, un `mientras`, una función...
// Cada declaración guarda su `linea` (la línea del archivo
// donde empieza), para que el verificador de préstamos pueda
// decirte en qué línea está el error, antes de correr nada.
#[derive(Debug, Clone)]
pub enum Declaracion {    // sea x = 5;   sea mut x: numero = 5;   sea x;
    Sea {
        linea: usize,
        nombre: String,
        mutable: bool,
        tipo: Option<String>,
        valor: Option<Expresion>,
    },
    // x = 6;
    Asignacion {
        linea: usize,
        nombre: String,
        valor: Expresion,
    },
    // lista[0] = 5;  mapa[21] = "FTP";
    // Cambia un lugar de una lista o el valor de una
    // clave de un mapa. El objetivo tiene que ser una
    // variable (no se puede cambiar el lugar de algo
    // que te devolvió una función, porque se pierde).
    AsignarIndice {
        linea: usize,
        objetivo: Expresion,
        indice: Expresion,
        valor: Expresion,
    },
    // x += 5;  x -= 2;  x *= 3;  x /= 2;  x %= 4;
    // Es un atajo: x += 5 es lo mismo que x = x + 5.
    AsignacionCompuesta {
        linea: usize,
        nombre: String,
        operador: OperadorBinario,
        valor: Expresion,
    },
    // Una expresión sola, como imprimir("Hola");
    Expresion {
        linea: usize,
        expresion: Expresion,
    },
    // si condicion { ... } sino { ... }
    Si {
        linea: usize,
        condicion: Expresion,
        entonces: Vec<Declaracion>,
        sino: Option<Vec<Declaracion>>,
    },
    // mientras condicion { ... }
    Mientras {
        linea: usize,
        condicion: Expresion,
        cuerpo: Vec<Declaracion>,
    },
    // repetir { ... } mientras condicion;
    // Corre el cuerpo una vez, y luego repite mientras la
    // condición sea verdadera (como el do-while).
    Repetir {
        linea: usize,
        cuerpo: Vec<Declaracion>,
        condicion: Expresion,
    },
    // para i en 1..5 { ... }   para i en 1..=5 { ... }
    // para i en 1..10 paso 2 { ... }
    Para {
        linea: usize,
        variable: String,
        inicio: Expresion,
        fin: Expresion,
        inclusivo: bool,
        paso: Option<Expresion>,
        cuerpo: Vec<Declaracion>,
    },
    // funcion sumar(a, b) { devolver a + b; }
    Funcion {
        linea: usize,
        nombre: String,
        parametros: Vec<String>,
        cuerpo: Vec<Declaracion>,
    },
    // devolver 42;
    Devolver {
        linea: usize,
        valor: Option<Expresion>,
    },
    // romper;  (salir del bucle)
    Romper {
        linea: usize,
    },
    // seguir;  (saltar a la siguiente vuelta)
    Seguir {
        linea: usize,
    },
    // segun puntos { 10 => { ... } _ => { ... } }
    // Elige un brazo según el valor: como el `match` de Rust.
    Segun {
        linea: usize,
        valor: Expresion,
        brazos: Vec<BrazoSegun>,
    },
    // importar "biblioteca.ec";
    Importar {
        linea: usize,
        ruta: String,
    },
    // constante PI = 3.14;   constante NOMBRE: texto = "Kevin";
    // Una constante nunca se puede cambiar ni se mueve:
    // es como una fotocopia que siempre puedes usar.
    Constante {
        linea: usize,
        nombre: String,
        tipo: Option<String>,
        valor: Expresion,
    },
    // para_cada numero en mi_lista { ... }
    // Recorre una lista, un elemento por vuelta, y lo
    // guarda en la variable (que es fija dentro del bucle).
    ParaCada {
        linea: usize,
        variable: String,
        lista: Expresion,
        cuerpo: Vec<Declaracion>,
    },
    // leer nombre;   (Input: lee lo que escribe el usuario
    // y lo guarda en la variable, que debe ser `sea mut`).
    // Si la variable es un numero, convierte el texto a
    // número; si es un logico, convierte "verdadero"/"falso";
    // si es una lista, separa lo escrito por comas.
    // Con mensaje: leer "¿Cómo te llamas?" nombre; muestra
    // el mensaje antes de leer (sin salto de línea).
    Leer {
        linea: usize,
        nombre: String,
        mensaje: Option<String>,
    },
    // puntos++;   puntos--;
    // Suma o resta 1 a un número. Solo sirve para números.
    Incrementar {
        linea: usize,
        nombre: String,
        subir: bool,
    },
    // { ... }  un bloque solo, con su propio ámbito
    Bloque {
        linea: usize,
        declaraciones: Vec<Declaracion>,
    },
    // usar matematicas;  (usar una librería, solo arriba)
    Usar {
        linea: usize,
        nombre: String,
    },
    // intentar { ... } si_falla { ... } al_final { ... }
    // Corre el cuerpo; si algo falla adentro, corre el
    // bloque `si_falla` (con el error en una variable, si
    // le pones nombre: si_falla e { ... }) en vez de parar
    // el programa. El bloque `al_final` corre siempre:
    // si todo salió bien, si falló, si devolvió, si rompió.
    // Al menos uno de los dos tiene que estar: un intentar
    // sin si_falla y sin al_final no sirve para nada.
    Intentar {
        linea: usize,
        cuerpo: Vec<Declaracion>,
        falla: Option<(Option<String>, Vec<Declaracion>)>,
        al_final: Option<Vec<Declaracion>>,
    },
}

impl Declaracion {
    // La línea del archivo donde empieza esta orden.
    pub fn linea(&self) -> usize {
        match self {
            Declaracion::Sea { linea, .. }
            | Declaracion::Asignacion { linea, .. }
            | Declaracion::AsignacionCompuesta { linea, .. }
            | Declaracion::Expresion { linea, .. }
            | Declaracion::Si { linea, .. }
            | Declaracion::Mientras { linea, .. }
            | Declaracion::Repetir { linea, .. }
            | Declaracion::Para { linea, .. }
            | Declaracion::Funcion { linea, .. }
            | Declaracion::Devolver { linea, .. }
            | Declaracion::Romper { linea, .. }
            | Declaracion::Seguir { linea, .. }
            | Declaracion::Segun { linea, .. }
            | Declaracion::Importar { linea, .. }
            | Declaracion::Constante { linea, .. }
            | Declaracion::ParaCada { linea, .. }
            | Declaracion::Leer { linea, .. }
            | Declaracion::Incrementar { linea, .. }
            | Declaracion::Bloque { linea, .. }
            | Declaracion::Usar { linea, .. }
            | Declaracion::AsignarIndice { linea, .. }
            | Declaracion::Intentar { linea, .. } => *linea,
        }
    }
}

// Un brazo de un `segun`: un patrón (un número, un texto,
// verdadero/falso, o `_` que significa "cualquier otro")
// y el bloque que corre si ese patrón coincide.
#[derive(Debug, Clone)]
pub struct BrazoSegun {
    pub patron: Option<Expresion>,
    pub cuerpo: Vec<Declaracion>,
}
