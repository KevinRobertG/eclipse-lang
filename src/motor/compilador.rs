// ============================================================
//  ECLIPSE 1.2 - El compilador
//  Archivo: src/compilador.rs
//
//  Qué hace:
//  El compilador traduce el arbolito (AST) a bytecode: una
//  lista de instrucciones chiquitas que la máquina virtual
//  sabe ejecutar.
//
//  Ejemplo, para:  sea x = 10 + 5;
//      0000 Constante 0 (15)        <- ya doblado: 10 + 5 = 15
//      0001 DefinirVariable x en slot 0 (global)
//
//  Cómo traduce cada cosa:
//  - Un `si` se vuelve dos saltos: si la condición es falsa,
//    salta al `sino`; si no, corre el bloque y salta al final.
//  - Un `mientras` se vuelve un salto hacia atrás: corre el
//    cuerpo y vuelve a preguntar la condición.
//  - Un `para` se desarma en un `mientras` con variables
//    escondidas: la que cuenta, el fin ($fin) y el paso
//    ($paso). La condición depende del signo del paso: con
//    paso positivo se sube, con paso negativo se baja.
//  - Un `segun` guarda el valor en una variable escondida
//    ($segun) y compara brazo por brazo.
//  - Una `funcion` se compila aparte, en su propio pedazo,
//    y se registra para poder llamarla cuando el programa
//    llegue a su definición (igual que el intérprete: no se
//    puede llamar a una función antes de definirla).
//  - `romper` y `seguir` son saltos que se parchean cuando
//    ya se sabe a dónde van.
//
//  Slots (poca RAM, más rápido):
//  Las variables ya no se buscan por su nombre: cada una
//  tiene un numerito de slot. Las variables globales (las
//  que creas arriba del todo) viven en un array compartido;
//  las demás viven en los slots del marco de la función.
//  Buscar por numerito es mucho más rápido que buscar por
//  nombre, y gasta menos RAM. Los slots se reutilizan cuando
//  un bloque se cierra.
//  Una función solo puede usar las variables globales que
//  ya existían cuando se definió (como en Rust: si usas una
//  variable que todavía no existe, el compilador te lo dice
//  en español antes de correr nada).
//
//  Optimizaciones:
//  - Doblado de constantes: 10 + 5 se calcula al compilar
//    y queda solo el 15. También funciona anidado.
//  - Nombres internados: cada nombre se guarda una sola vez.
//  - Peephole: quita los saltos que van a la instrucción
//    siguiente.
//
//  Ownership (como Rust): si le das un texto de una variable
//  a otra variable o a una función, el compilador escribe una
//  instrucción Mover que anota A DÓNDE se fue. Los números y
//  los lógicos son fotocopias: no se mueven. `&nombre` presta
//  el texto sin moverlo. Las funciones que trae Eclipse solo
//  miran el texto, no se lo quedan.
//
//  El compilador NO cambia la lógica de Eclipse: el intérprete
//  y la máquina virtual hacen exactamente lo mismo, solo que
//  por caminos distintos.
// ============================================================

use crate::lenguaje::ast::{Declaracion, Expresion, OperadorBinario, OperadorUnario, ParteCadena};
use crate::motor::bytecode::{InfoParaAvanzar, Instruccion, Pedazo, Programa};
use crate::motor::valor::Valor;
use crate::lenguaje::lexer::Lexer;
use crate::lenguaje::parser::Parser;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

// Un ámbito local: nombre -> (slot, mutable), y el siguiente
// slot que había cuando se abrió (para liberarlos al cerrar).
type AmbitoLocal = (HashMap<String, (usize, bool)>, usize);

// La información de un bucle que se está compilando:
// a dónde salta `seguir`, y los saltos que hay que
// parchear al final.
struct InfoBucle {
    inicio: usize,
    es_para: bool,
    es_repetir: bool,
    rupturas: Vec<usize>,
    continuaciones: Vec<usize>,
    // Cuántos manejadores de `intentar` había abiertos cuando
    // empezó el bucle: al romper o seguir, solo se quitan los
    // que se abrieron dentro del bucle (y corren sus al_final).
    profundidad_manejador: usize,
}

pub struct Compilador {
    pedazo: Pedazo,
    funciones: HashMap<String, Pedazo>,
    bucles: Vec<InfoBucle>,
    // Cuántos manejadores de `intentar` hay abiertos ahora
    // (uno por cada `intentar` que rodea este punto).
    manejadores: usize,
    // El `al_final` de cada `intentar` abierto (None si ese
    // intentar solo tiene `si_falla`): uno por manejador, en
    // el mismo orden, del más afuera al más adentro.
    finallys: Vec<Option<Vec<Declaracion>>>,
    en_funcion: bool,
    importados: HashSet<PathBuf>,
    pre_escaneados: HashSet<PathBuf>,
    directorio_base: PathBuf,
    // Ámbitos locales del pedazo que se está compilando.
    locales: Vec<AmbitoLocal>,
    siguiente_slot: usize,
    max_slots: usize,
    // Variables globales (compartidas por todo el programa).
    globales: HashMap<String, (usize, bool)>,
    siguiente_global: usize,
}

// Compila un archivo .ec entero: lexer -> parser -> bytecode.
// `nombre` es el nombre del archivo, para ponerlo en los errores.
pub fn compilar(fuente: &str, base: PathBuf, nombre: &str) -> Result<Programa, String> {
    let tokens = Lexer::nuevo(fuente)
        .tokenizar()
        .map_err(|e| format!("{}: {}", nombre, e))?;
    let declaraciones = Parser::nuevo(tokens)
        .parsear()
        .map_err(|e| format!("{}: {}", nombre, e))?;
    // El verificador de préstamos te regaña ANTES de correr.
    crate::motor::verificador::verificar(&declaraciones, &base)
        .map_err(|e| format!("{}: {}", nombre, e.texto()))?;
    let mut c = Compilador::nuevo(base);
    // Primera pasada: apuntar las variables globales de arriba
    // del todo (también las de los archivos importados), para
    // que una función pueda usar una global que se define
    // después de ella, igual que en el intérprete.
    c.pre_escanear(&declaraciones)?;
    for d in &declaraciones {
        c.compilar_declaracion(d)?;
    }
    // Todo programa termina devolviendo nulo.
    c.emitir(Instruccion::Nulo);
    c.emitir(Instruccion::Retornar);
    c.pedazo.max_slots = c.max_slots;
    let mut programa = Programa {
        principal: c.pedazo,
        funciones: c.funciones,
    };
    // Optimización final: quitar los saltos inútiles.
    programa.principal.optimizar();
    for pedazo in programa.funciones.values_mut() {
        pedazo.optimizar();
    }
    Ok(programa)
}

impl Compilador {
    // Un compilador nuevo, que recuerda las variables globales
    // y las funciones de todo lo que ya compiló (para el modo
    // interactivo, donde cada línea usa lo de las anteriores).
    pub fn nuevo(directorio_base: PathBuf) -> Self {
        Compilador {
            pedazo: Pedazo::nuevo(),
            funciones: HashMap::new(),
            bucles: Vec::new(),
            manejadores: 0,
            finallys: Vec::new(),
            en_funcion: false,
            importados: HashSet::new(),
            pre_escaneados: HashSet::new(),
            directorio_base,
            locales: vec![(HashMap::new(), 0)],
            siguiente_slot: 0,
            max_slots: 0,
            globales: HashMap::new(),
            siguiente_global: 0,
        }
    }

    // Las funciones conocidas hasta ahora (para registrarlas
    // en la máquina virtual del modo interactivo).
    pub fn funciones(&self) -> &HashMap<String, Pedazo> {
        &self.funciones
    }

    // Compila unas líneas sueltas (para el modo interactivo):
    // sin verificador, porque usan variables de líneas
    // anteriores. Devuelve el pedazo con esas líneas.
    // Las variables globales nuevas se quedan apuntadas
    // (con su slot) para la siguiente línea, y las funciones
    // nuevas se guardan en `funciones`.
    pub fn compilar_fragmento(
        &mut self,
        declaraciones: &[Declaracion],
    ) -> Result<Pedazo, String> {
        // Estado limpio para el fragmento: los ámbitos y los
        // bucles no se heredan de la línea anterior.
        self.locales = vec![(HashMap::new(), 0)];
        self.siguiente_slot = 0;
        self.max_slots = 0;
        self.bucles.clear();
        self.manejadores = 0;
        self.finallys.clear();
        self.en_funcion = false;
        self.pedazo = Pedazo::nuevo();
        self.pre_escanear(declaraciones)?;
        for d in declaraciones {
            self.compilar_declaracion(d)?;
        }
        // Todo fragmento termina devolviendo nulo.
        self.emitir(Instruccion::Nulo);
        self.emitir(Instruccion::Retornar);
        self.pedazo.max_slots = self.max_slots;
        let mut pedazo = std::mem::replace(&mut self.pedazo, Pedazo::nuevo());
        pedazo.optimizar();
        for pedazo_fn in self.funciones.values_mut() {
            pedazo_fn.optimizar();
        }
        Ok(pedazo)
    }
}

// ¿Es una de las funciones que trae Eclipse? Esas solo
// MIRAN el texto que les pasas, no se lo quedan.
fn es_builtin(nombre: &str) -> bool {
    matches!(
        nombre,
        "imprimir"
            | "leer"
            | "leer_numero"
            | "a_numero"
            | "a_texto"
            | "leer_archivo"
            | "escribir_archivo"
            | "largo"
            | "tipo_de"
            | "aleatorio"
            | "copiar"
            | "min"
            | "max"
            | "agregar"
            | "elemento"
            | "ahora"
            | "sistema"
            | "usuario"
            | "carpeta"
            | "variable_entorno"
            | "comando"
            | "argumentos"
            | "procesadores"
            | "memoria"
            | "caracter"
            | "redondear"
            | "absoluto"
            | "mayusculas"
            | "minusculas"
            | "contiene"
            | "ordenar"
            | "invertir"
            | "unir"
            | "separar"
            | "raiz"
            | "claves"
            | "valores"
            | "tiene"
            | "fecha"
            | "hora"
            | "fecha_hora"
            | "dormir"
            | "existe"
            | "listar_carpeta"
            | "crear_carpeta"
            | "mover_archivo"
            | "borrar_archivo"
            | "reemplazar"
    )
}

// Si una expresión es solo valores fijos (números, textos,
// verdadero/falso, nulo, sumas de valores fijos...),
// calcula su valor aquí, en el compilador. Si aparece una
// variable o una llamada, devuelve None: ese valor solo se
// sabe cuando el programa corre.
fn evaluar_constante(e: &Expresion) -> Option<Valor> {
    match e {
        Expresion::Numero(n) => Some(Valor::Numero(*n)),
        Expresion::Cadena(s) => Some(Valor::Texto(s.clone())),
        Expresion::Logico(b) => Some(Valor::Logico(*b)),
        Expresion::Nulo => Some(Valor::Nulo),
        Expresion::Agrupada(interior) => evaluar_constante(interior),
        // [1, 2, 3]: si todos los elementos son fijos, la
        // lista entera es fija.
        Expresion::Lista(elementos) => {
            let mut valores = Vec::new();
            for e in elementos {
                valores.push(evaluar_constante(e)?);
            }
            Some(Valor::lista(valores))
        }
        // {21: "FTP", 22: "SSH"}: si todas las claves y los
        // valores son fijos, el mapa entero es fijo.
        Expresion::Mapa(pares) => {
            let mut valores = Vec::new();
            for (clave, valor) in pares {
                let clave = evaluar_constante(clave)?;
                let valor = evaluar_constante(valor)?;
                match &clave {
                    Valor::Numero(_) | Valor::Texto(_) | Valor::Logico(_) => {}
                    _ => return None,
                }
                valores.push((clave, valor));
            }
            let mut unicos: Vec<(Valor, Valor)> = Vec::new();
            for (clave, valor) in valores {
                match Valor::buscar_clave(&unicos, &clave) {
                    Some(pos) => unicos[pos].1 = valor,
                    None => unicos.push((clave, valor)),
                }
            }
            Some(Valor::mapa(unicos))
        }
        // mi_lista[0] o mi_mapa[21]: si la colección y el
        // índice son fijos, el elemento también lo es.
        Expresion::Indice { lista, indice } => {
            let lista = evaluar_constante(lista)?;
            let indice = evaluar_constante(indice)?;
            match lista {
                Valor::Lista(elementos) => {
                    let i = match indice {
                        Valor::Numero(n) => n as i64,
                        _ => return None,
                    };
                    if i < 0 || i as usize >= elementos.len() {
                        None
                    } else {
                        Some(elementos[i as usize].clone())
                    }
                }
                Valor::Mapa(pares) => {
                    let clave = match &indice {
                        Valor::Numero(_) | Valor::Texto(_) | Valor::Logico(_) => indice,
                        _ => return None,
                    };
                    Valor::buscar_clave(&pares, &clave).map(|pos| pares[pos].1.clone())
                }
                _ => None,
            }
        }
        Expresion::Unaria { operador, derecha } => {
            let v = evaluar_constante(derecha)?;
            match operador {
                OperadorUnario::Negativo => match v {
                    Valor::Numero(n) => Some(Valor::Numero(-n)),
                    _ => None,
                },
                OperadorUnario::No => Some(Valor::Logico(!v.es_verdadero())),
            }
        }
        Expresion::Binaria {
            izquierda,
            operador,
            derecha,
        } => {
            match operador {
                // `y` con el primero falso ya es falso,
                // sin mirar el segundo.
                OperadorBinario::Y => {
                    let a = evaluar_constante(izquierda)?;
                    if !a.es_verdadero() {
                        return Some(Valor::Logico(false));
                    }
                    let b = evaluar_constante(derecha)?;
                    Some(Valor::Logico(b.es_verdadero()))
                }
                // `o` con el primero verdadero ya es verdadero.
                OperadorBinario::O => {
                    let a = evaluar_constante(izquierda)?;
                    if a.es_verdadero() {
                        return Some(Valor::Logico(true));
                    }
                    let b = evaluar_constante(derecha)?;
                    Some(Valor::Logico(b.es_verdadero()))
                }
                _ => {
                    let a = evaluar_constante(izquierda)?;
                    let b = evaluar_constante(derecha)?;
                    doblar(&a, operador, &b)
                }
            }
        }
        _ => None,
    }
}

// Calcula una operación con dos valores fijos, en el
// compilador. Si no se puede doblar (por ejemplo, dividir
// entre cero, que es un error de ejecución), devuelve None
// y el cálculo queda para la máquina virtual.
fn doblar(izq: &Valor, operador: &OperadorBinario, der: &Valor) -> Option<Valor> {
    match operador {
        OperadorBinario::Sumar => match (izq, der) {
            (Valor::Numero(a), Valor::Numero(b)) => Some(Valor::Numero(a + b)),
            // Sumar dos listas las pega: [1, 2] + [3] es [1, 2, 3].
            (Valor::Lista(a), Valor::Lista(b)) => {
                let mut resultado = (**a).clone();
                resultado.extend(b.iter().cloned());
                Some(Valor::lista(resultado))
            }
            _ => Some(Valor::Texto(format!(
                "{}{}",
                izq.a_texto(),
                der.a_texto()
            ))),
        },
        OperadorBinario::Restar => match (izq, der) {
            (Valor::Numero(a), Valor::Numero(b)) => Some(Valor::Numero(a - b)),
            _ => None,
        },
        OperadorBinario::Multiplicar => match (izq, der) {
            (Valor::Numero(a), Valor::Numero(b)) => Some(Valor::Numero(a * b)),
            _ => None,
        },
        OperadorBinario::Dividir => match (izq, der) {
            (Valor::Numero(a), Valor::Numero(b)) if *b != 0.0 => Some(Valor::Numero(a / b)),
            _ => None,
        },
        OperadorBinario::Residuo => match (izq, der) {
            // El resto entre cero no se dobla: si lo dobláramos,
            // saldría NaN en vez del error en español.
            (Valor::Numero(a), Valor::Numero(b)) if *b != 0.0 => {
                Some(Valor::Numero(a % b))
            }
            _ => None,
        },
        OperadorBinario::Potencia => match (izq, der) {
            (Valor::Numero(a), Valor::Numero(b)) => Some(Valor::Numero(a.powf(*b))),
            _ => None,
        },
        OperadorBinario::Igual => Some(Valor::Logico(izq == der)),
        OperadorBinario::Diferente => Some(Valor::Logico(izq != der)),
        OperadorBinario::Menor => match (izq, der) {
            (Valor::Numero(a), Valor::Numero(b)) => Some(Valor::Logico(a < b)),
            _ => None,
        },
        OperadorBinario::MenorIgual => match (izq, der) {
            (Valor::Numero(a), Valor::Numero(b)) => Some(Valor::Logico(a <= b)),
            _ => None,
        },
        OperadorBinario::Mayor => match (izq, der) {
            (Valor::Numero(a), Valor::Numero(b)) => Some(Valor::Logico(a > b)),
            _ => None,
        },
        OperadorBinario::MayorIgual => match (izq, der) {
            (Valor::Numero(a), Valor::Numero(b)) => Some(Valor::Logico(a >= b)),
            _ => None,
        },
        OperadorBinario::Y | OperadorBinario::O => None,
    }
}

// La condición de un `para` como arbolito: depende del signo
// del paso. Con paso positivo se sube, con paso negativo se
// baja, con paso 0 la condición es falsa (el bucle no corre).
//   (paso > 0 y variable <= fin) o (paso < 0 y variable >= fin)
// (con `..` en vez de `..=` es < y > en vez de <= y >=).
// La usa también el transpilador (para el .exe).
// La condición del `para` cuando el paso es fijo: si el
// paso es positivo, solo hay que ver si la variable llegó
// al fin por arriba; si es negativo, por abajo; si es 0,
// el bucle no corre nunca. Es la misma lógica que el
// intérprete usa en su bucle nativo.
pub(crate) fn condicion_para_optimizada(
    variable: &str,
    inclusivo: bool,
    paso: &Option<Expresion>,
) -> Expresion {
    let numero_paso = match paso {
        None => Some(1.0),
        Some(Expresion::Numero(n)) => Some(*n),
        Some(Expresion::Unaria {
            operador: OperadorUnario::Negativo,
            derecha: interior,
        }) => match &**interior {
            Expresion::Numero(n) => Some(-n),
            _ => None,
        },
        _ => None,
    };
    if let Some(n) = numero_paso {
        if n == 0.0 {
            return Expresion::Logico(false);
        }
        let op = if n > 0.0 {
            if inclusivo {
                OperadorBinario::MenorIgual
            } else {
                OperadorBinario::Menor
            }
        } else if inclusivo {
            OperadorBinario::MayorIgual
        } else {
            OperadorBinario::Mayor
        };
        return Expresion::Binaria {
            izquierda: Box::new(Expresion::Variable(variable.to_string())),
            operador: op,
            derecha: Box::new(Expresion::Variable("$fin".to_string())),
        };
    }
    condicion_para(variable, inclusivo)
}

pub(crate) fn condicion_para(variable: &str, inclusivo: bool) -> Expresion {
    let var = || Expresion::Variable(variable.to_string());
    let fin = || Expresion::Variable("$fin".to_string());
    let paso = || Expresion::Variable("$paso".to_string());
    let cero = || Expresion::Numero(0.0);
    let op_pos = if inclusivo {
        OperadorBinario::MenorIgual
    } else {
        OperadorBinario::Menor
    };
    let op_neg = if inclusivo {
        OperadorBinario::MayorIgual
    } else {
        OperadorBinario::Mayor
    };
    let cond_pos = Expresion::Binaria {
        izquierda: Box::new(var()),
        operador: op_pos,
        derecha: Box::new(fin()),
    };
    let cond_neg = Expresion::Binaria {
        izquierda: Box::new(var()),
        operador: op_neg,
        derecha: Box::new(fin()),
    };
    let rama_pos = Expresion::Binaria {
        izquierda: Box::new(Expresion::Binaria {
            izquierda: Box::new(paso()),
            operador: OperadorBinario::Mayor,
            derecha: Box::new(cero()),
        }),
        operador: OperadorBinario::Y,
        derecha: Box::new(cond_pos),
    };
    let rama_neg = Expresion::Binaria {
        izquierda: Box::new(Expresion::Binaria {
            izquierda: Box::new(paso()),
            operador: OperadorBinario::Menor,
            derecha: Box::new(cero()),
        }),
        operador: OperadorBinario::Y,
        derecha: Box::new(cond_neg),
    };
    Expresion::Binaria {
        izquierda: Box::new(rama_pos),
        operador: OperadorBinario::O,
        derecha: Box::new(rama_neg),
    }
}

impl Compilador {
    // Escribe una instrucción al final del pedazo.
    fn emitir(&mut self, instr: Instruccion) {
        self.pedazo.codigo.push(instr);
    }

    // Escribe un salto con destino 0 y devuelve su posición,
    // para parchearlo después cuando ya se sepa a dónde va.
    fn emitir_salto(&mut self, instr: Instruccion) -> usize {
        let pos = self.pedazo.codigo.len();
        self.pedazo.codigo.push(instr);
        pos
    }

    // Pone el destino verdadero en un salto que estaba en 0.
    fn parchear(&mut self, pos: usize, destino: usize) {
        match &mut self.pedazo.codigo[pos] {
            Instruccion::Saltar(d) => *d = destino,
            Instruccion::SaltarSiFalso(d) => *d = destino,
            _ => {}
        }
    }

    // Pone un número en la pila.
    fn constante_numero(&mut self, n: f64) {
        let idx = self.pedazo.agregar_constante(Valor::Numero(n));
        self.emitir(Instruccion::Constante(idx));
    }

    // Pone un valor fijo en la pila (número, texto o lógico).
    fn emitir_constante(&mut self, valor: Valor) {
        match valor {
            Valor::Logico(true) => self.emitir(Instruccion::Verdadero),
            Valor::Logico(false) => self.emitir(Instruccion::Falso),
            Valor::Nulo => self.emitir(Instruccion::Nulo),
            _ => {
                let idx = self.pedazo.agregar_constante(valor);
                self.emitir(Instruccion::Constante(idx));
            }
        }
    }

    // El numerito del nombre en la tabla del pedazo.
    fn nombre(&mut self, nombre: &str) -> usize {
        self.pedazo.internar(nombre)
    }

    // Busca una variable: primero en los ámbitos locales
    // (del más adentro al más afuera), luego en las globales.
    // Devuelve (slot, es_global).
    fn resolver(&self, nombre: &str) -> Option<(usize, bool)> {
        for (ambito, _) in self.locales.iter().rev() {
            if let Some((slot, _)) = ambito.get(nombre) {
                return Some((*slot, false));
            }
        }
        if let Some((slot, _)) = self.globales.get(nombre) {
            return Some((*slot, true));
        }
        None
    }

    // Resuelve una variable y devuelve (slot, es_global,
    // numerito del nombre). Si no existe, da un error en
    // español (como en Rust: usar algo que no existe es un
    // error de compilación, no de ejecución).
    fn variable(&mut self, nombre: &str) -> Result<(usize, bool, usize), String> {
        let (slot, global) = self
            .resolver(nombre)
            .ok_or_else(|| format!("no conozco la variable '{}'", nombre))?;
        let idx = self.pedazo.internar(nombre);
        Ok((slot, global, idx))
    }

    // Crea una variable nueva y devuelve (slot, es_global,
    // numerito del nombre). Arriba del todo (en el programa
    // principal, fuera de cualquier bloque) es global; en
    // cualquier otro lugar es local. Si la global ya estaba
    // apuntada en la primera pasada, se reutiliza su slot.
    fn definir(&mut self, nombre: &str, mutable: bool) -> (usize, bool, usize) {
        if !self.en_funcion && self.locales.len() == 1 {
            if let Some((slot, _)) = self.globales.get(nombre) {
                let slot = *slot;
                if let Some((_, m)) = self.globales.get_mut(nombre) {
                    *m = mutable;
                }
                let idx = self.pedazo.internar(nombre);
                return (slot, true, idx);
            }
            let slot = self.siguiente_global;
            self.siguiente_global += 1;
            self.globales.insert(nombre.to_string(), (slot, mutable));
            let idx = self.pedazo.internar(nombre);
            return (slot, true, idx);
        }
        let slot = self.siguiente_slot;
        self.siguiente_slot += 1;
        if self.siguiente_slot > self.max_slots {
            self.max_slots = self.siguiente_slot;
        }
        if let Some((ambito, _)) = self.locales.last_mut() {
            ambito.insert(nombre.to_string(), (slot, mutable));
        }
        let idx = self.pedazo.internar(nombre);
        (slot, false, idx)
    }

    // Reserva un slot para una constante (siempre fija).
    // La orden que la crea emite DefinirConstante, para que
    // la máquina virtual sepa que nunca se puede cambiar.
    fn definir_constante(&mut self, nombre: &str) -> (usize, bool, usize) {
        self.definir(nombre, false)
    }

    // Primera pasada: apunta las variables globales de arriba
    // del todo (y las de los archivos importados) antes de
    // compilar nada. Así una función puede usar una global
    // que se define después de ella, igual que en el
    // intérprete. Si un archivo no se puede leer o tiene un
    // error, se salta: el error saldrá al compilar de verdad.
    fn pre_escanear(&mut self, declaraciones: &[Declaracion]) -> Result<(), String> {
        for d in declaraciones {
            match d {
                Declaracion::Sea { nombre, .. } | Declaracion::Constante { nombre, .. } => {
                    if !self.globales.contains_key(nombre) {
                        let slot = self.siguiente_global;
                        self.siguiente_global += 1;
                        self.globales.insert(nombre.clone(), (slot, false));
                    }
                }
                Declaracion::Importar { ruta, .. } => {
                    let completa = if Path::new(ruta).is_absolute() {
                        PathBuf::from(ruta)
                    } else {
                        self.directorio_base.join(ruta)
                    };
                    let canonica = match fs::canonicalize(&completa) {
                        Ok(c) => c,
                        Err(_) => continue,
                    };
                    if self.pre_escaneados.contains(&canonica) {
                        continue;
                    }
                    self.pre_escaneados.insert(canonica.clone());
                    let fuente = match fs::read_to_string(&canonica) {
                        Ok(f) => f,
                        Err(_) => continue,
                    };
                    let tokens = match Lexer::nuevo(&fuente).tokenizar() {
                        Ok(t) => t,
                        Err(_) => continue,
                    };
                    let decls = match Parser::nuevo(tokens).parsear() {
                        Ok(d) => d,
                        Err(_) => continue,
                    };
                    self.pre_escanear(&decls)?;
                }
                _ => {}
            }
        }
        Ok(())
    }

    // Abre un ámbito local nuevo (al entrar a un bloque).
    fn entrar_ambito(&mut self) {
        let base = self.siguiente_slot;
        self.locales.push((HashMap::new(), base));
    }

    // Cierra el ámbito local actual y libera sus slots
    // para que otro bloque los pueda reutilizar.
    fn salir_ambito(&mut self) {
        if let Some((_, base)) = self.locales.pop() {
            self.siguiente_slot = base;
        }
    }

    // Compila un bloque { ... } con su propio ámbito local.
    fn compilar_bloque(&mut self, cuerpo: &[Declaracion]) -> Result<(), String> {
        self.entrar_ambito();
        for d in cuerpo {
            self.compilar_declaracion(d)?;
        }
        self.salir_ambito();
        Ok(())
    }

    // Compila una sola orden.
    fn compilar_declaracion(&mut self, d: &Declaracion) -> Result<(), String> {
        match d {
            Declaracion::Sea {
                nombre,
                mutable,
                tipo,
                valor,
                ..
            } => {
                match valor {
                    Some(e) => self.compilar_expresion(e)?,
                    None => self.emitir(Instruccion::Nulo),
                }
                // La variable se crea antes de verificar el tipo
                // y de mover, porque esas instrucciones necesitan
                // su slot.
                let (slot, global, idx_nombre) = self.definir(nombre, *mutable);
                // El tipo solo se revisa cuando sí hay valor.
                if let Some(t) = tipo {
                    if valor.is_some() {
                        let idx_tipo = self.nombre(t);
                        self.emitir(Instruccion::VerificarTipo(
                            idx_tipo, slot, global, idx_nombre,
                        ));
                    }
                }
                // Ownership: si el valor venía de una variable,
                // esa variable se mueve a la nueva.
                if let Some(Expresion::Variable(fuente)) = valor {
                    if fuente != nombre {
                        let (slot_f, global_f, idx_f) = self.variable(fuente)?;
                        let destino = format!("a la variable '{}'", nombre);
                        let idx_destino = self.nombre(&destino);
                        self.emitir(Instruccion::Mover(
                            slot_f, global_f, idx_f, idx_destino,
                        ));
                    }
                }
                self.emitir(Instruccion::DefinirVariable(
                    slot, *mutable, global, idx_nombre,
                ));
                Ok(())
            }
            Declaracion::Asignacion { nombre, valor, .. } => {
                // Atajo: `x = agregar(x, v)` mete el valor en su
                // sitio, sin clonar la lista entera. Solo vale
                // cuando el primer dato es justo la variable a
                // la que se asigna; cualquier otro `agregar`
                // sigue el camino normal (y clona si hace falta).
                if let Expresion::Llamada {
                    nombre: f,
                    argumentos,
                } = valor
                {
                    if f == "agregar" && argumentos.len() == 2 {
                        if let Expresion::Variable(fuente) = &argumentos[0] {
                            if fuente == nombre {
                                self.compilar_expresion(&argumentos[1])?;
                                let (slot, global, idx) = self.variable(nombre)?;
                                self.emitir(Instruccion::AgregarEnSitio(
                                    slot, global, idx,
                                ));
                                return Ok(());
                            }
                        }
                    }
                }
                self.compilar_expresion(valor)?;
                // Ownership: si el valor venía de otra variable,
                // esa variable se mueve a esta.
                if let Expresion::Variable(fuente) = valor {
                    if fuente != nombre {
                        let (slot_f, global_f, idx_f) = self.variable(fuente)?;
                        let destino = format!("a la variable '{}'", nombre);
                        let idx_destino = self.nombre(&destino);
                        self.emitir(Instruccion::Mover(
                            slot_f, global_f, idx_f, idx_destino,
                        ));
                    }
                }
                let (slot, global, idx) = self.variable(nombre)?;
                self.emitir(Instruccion::AsignarVariable(slot, global, idx));
                Ok(())
            }
            Declaracion::AsignacionCompuesta {
                nombre,
                operador,
                valor,
                ..
            } => {
                // x += 5 es lo mismo que x = x + 5.
                let (slot, global, idx) = self.variable(nombre)?;
                self.emitir(Instruccion::ObtenerVariable(slot, global, idx));
                self.compilar_expresion(valor)?;
                let instr = match operador {
                    OperadorBinario::Sumar => Instruccion::Sumar,
                    OperadorBinario::Restar => Instruccion::Restar,
                    OperadorBinario::Multiplicar => Instruccion::Multiplicar,
                    OperadorBinario::Dividir => Instruccion::Dividir,
                    OperadorBinario::Residuo => Instruccion::Residuo,
                    OperadorBinario::Potencia => Instruccion::Potencia,
                    _ => unreachable!(),
                };
                self.emitir(instr);
                self.emitir(Instruccion::AsignarVariable(slot, global, idx));
                Ok(())
            }
            Declaracion::AsignarIndice {
                objetivo,
                indice,
                valor,
                ..
            } => {
                // lista[0] = 5;  mapa[21] = "FTP";
                // Solo vale si el objetivo es una variable: si
                // es un lugar dentro de otro lugar (m[0][1] = 5),
                // guárdalo primero en una variable.
                let nombre = match objetivo {
                    Expresion::Variable(n) => n.clone(),
                    _ => {
                        return Err(
                            "para cambiar un lugar dentro de otro lugar, guárdalo primero en una variable".to_string(),
                        )
                    }
                };
                self.compilar_expresion(indice)?;
                self.compilar_expresion(valor)?;
                let (slot, global, idx) = self.variable(&nombre)?;
                self.emitir(Instruccion::AsignarIndice(slot, global, idx));
                Ok(())
            }
            Declaracion::Constante {
                nombre,
                tipo,
                valor,
                ..
            } => {
                // Una constante: se calcula su valor y se guarda
                // con DefinirConstante (nunca se puede cambiar).
                self.compilar_expresion(valor)?;
                let (slot, global, idx_nombre) = self.definir_constante(nombre);
                if let Some(t) = tipo {
                    let idx_tipo = self.nombre(t);
                    self.emitir(Instruccion::VerificarTipo(
                        idx_tipo, slot, global, idx_nombre,
                    ));
                }
                self.emitir(Instruccion::DefinirConstante(slot, global, idx_nombre));
                Ok(())
            }
            Declaracion::ParaCada {
                variable,
                lista,
                cuerpo,
                ..
            } => {
                // El `para_cada` se desarma en un `mientras` con
                // variables escondidas: la lista ($lista), su
                // largo ($largo) y el índice ($indice).
                self.entrar_ambito();
                self.compilar_expresion(lista)?;
                let (slot_lista, global_lista, idx_lista) = self.definir("$lista", false);
                self.emitir(Instruccion::DefinirVariable(
                    slot_lista, false, global_lista, idx_lista,
                ));
                // `para_cada` necesita una lista: si no lo es,
                // se dice con el mismo mensaje que el intérprete.
                self.emitir(Instruccion::VerificarLista(
                    slot_lista, global_lista, idx_lista,
                ));
                // $largo = largo($lista)
                self.emitir(Instruccion::ObtenerVariable(
                    slot_lista, global_lista, idx_lista,
                ));
                let idx_largo_fn = self.nombre("largo");
                self.emitir(Instruccion::Llamar(idx_largo_fn, 1));
                let (slot_largo, global_largo, idx_largo) = self.definir("$largo", false);
                self.emitir(Instruccion::DefinirVariable(
                    slot_largo, false, global_largo, idx_largo,
                ));
                // $indice = 0
                self.constante_numero(0.0);
                let (slot_ind, global_ind, idx_ind) = self.definir("$indice", false);
                self.emitir(Instruccion::DefinirVariable(
                    slot_ind, false, global_ind, idx_ind,
                ));
                // Mientras $indice < $largo: la variable del
                // bucle es $lista[$indice].
                let inicio_cond = self.pedazo.codigo.len();
                self.emitir(Instruccion::ObtenerVariable(
                    slot_ind, global_ind, idx_ind,
                ));
                self.emitir(Instruccion::ObtenerVariable(
                    slot_largo, global_largo, idx_largo,
                ));
                self.emitir(Instruccion::Menor);
                let salto_fin = self.emitir_salto(Instruccion::SaltarSiFalso(0));
                self.bucles.push(InfoBucle {
                    profundidad_manejador: self.manejadores,
                    inicio: inicio_cond,
                    es_para: true,
                    es_repetir: false,
                    rupturas: Vec::new(),
                    continuaciones: Vec::new(),
                });
                // La variable del bucle vive en su propio ámbito,
                // y el cuerpo en otro más adentro (igual que en
                // el `para`: así una variable del cuerpo es nueva
                // en cada vuelta).
                self.entrar_ambito();
                self.emitir(Instruccion::ObtenerVariable(
                    slot_lista, global_lista, idx_lista,
                ));
                self.emitir(Instruccion::ObtenerVariable(
                    slot_ind, global_ind, idx_ind,
                ));
                self.emitir(Instruccion::Indice);
                let (slot_var, global_var, idx_var) = self.definir(variable, false);
                self.emitir(Instruccion::DefinirVariable(
                    slot_var, false, global_var, idx_var,
                ));
                self.compilar_bloque(cuerpo)?;
                self.salir_ambito();
                // Aquí va el incremento: $indice = $indice + 1
                let inicio_inc = self.pedazo.codigo.len();
                self.emitir(Instruccion::ObtenerVariable(
                    slot_ind, global_ind, idx_ind,
                ));
                self.constante_numero(1.0);
                self.emitir(Instruccion::Sumar);
                self.emitir(Instruccion::DefinirVariable(
                    slot_ind, false, global_ind, idx_ind,
                ));
                self.emitir(Instruccion::Saltar(inicio_cond));
                let fin = self.pedazo.codigo.len();
                self.parchear(salto_fin, fin);
                let bucle = self.bucles.pop().ok_or_else(|| {
                    "error interno: se perdió el bucle que se estaba compilando".to_string()
                })?;
                for pos in bucle.rupturas {
                    self.parchear(pos, fin);
                }
                for pos in bucle.continuaciones {
                    self.parchear(pos, inicio_inc);
                }
                self.salir_ambito();
                Ok(())
            }
            Declaracion::Leer { nombre, mensaje, .. } => {
                // Input: la máquina virtual lee y guarda. El mensaje
                // (si hay) va a la tabla de constantes, para que la
                // VM lo muestre antes de leer.
                let (slot, global, idx) = self.variable(nombre)?;
                let mensaje_idx = match mensaje {
                    Some(m) => Some(
                        self.pedazo
                            .agregar_constante(crate::motor::valor::Valor::Texto(
                                m.clone(),
                            )),
                    ),
                    None => None,
                };
                self.emitir(Instruccion::LeerEntrada(slot, global, idx, mensaje_idx));
                Ok(())
            }
            Declaracion::Incrementar { nombre, subir, .. } => {
                // puntos++ / puntos--: la máquina virtual revisa
                // que sea un número y suma o resta 1.
                let (slot, global, idx) = self.variable(nombre)?;
                self.emitir(Instruccion::Incrementar(slot, global, idx, *subir));
                Ok(())
            }
            Declaracion::Expresion { expresion: e, .. } => {
                self.compilar_expresion(e)?;
                self.emitir(Instruccion::Sacar);
                Ok(())
            }
            Declaracion::Si {
                condicion,
                entonces,
                sino,
                ..
            } => {
                self.compilar_expresion(condicion)?;
                let salto_else = self.emitir_salto(Instruccion::SaltarSiFalso(0));
                self.compilar_bloque(entonces)?;
                let salto_fin = self.emitir_salto(Instruccion::Saltar(0));
                self.parchear(salto_else, self.pedazo.codigo.len());
                if let Some(sino) = sino {
                    self.compilar_bloque(sino)?;
                }
                self.parchear(salto_fin, self.pedazo.codigo.len());
                Ok(())
            }
            Declaracion::Mientras { condicion, cuerpo, .. } => {
                let inicio = self.pedazo.codigo.len();
                self.compilar_expresion(condicion)?;
                let salto_fin = self.emitir_salto(Instruccion::SaltarSiFalso(0));
                self.bucles.push(InfoBucle {
                    profundidad_manejador: self.manejadores,
                    inicio,
                    es_para: false,
                    es_repetir: false,
                    rupturas: Vec::new(),
                    continuaciones: Vec::new(),
                });
                self.compilar_bloque(cuerpo)?;
                self.emitir(Instruccion::Saltar(inicio));
                let fin = self.pedazo.codigo.len();
                self.parchear(salto_fin, fin);
                let bucle = self.bucles.pop().ok_or_else(|| {
                    "error interno: se perdió el bucle que se estaba compilando".to_string()
                })?;
                for pos in bucle.rupturas {
                    self.parchear(pos, fin);
                }
                Ok(())
            }
            Declaracion::Repetir { cuerpo, condicion, .. } => {
                // El cuerpo corre una vez, y luego repite
                // mientras la condición sea verdadera.
                // `seguir` salta a la condición (se parchea
                // después del cuerpo), `romper` salta al final.
                let inicio = self.pedazo.codigo.len();
                self.bucles.push(InfoBucle {
                    profundidad_manejador: self.manejadores,
                    inicio,
                    es_para: false,
                    es_repetir: true,
                    rupturas: Vec::new(),
                    continuaciones: Vec::new(),
                });
                self.compilar_bloque(cuerpo)?;
                let inicio_cond = self.pedazo.codigo.len();
                self.compilar_expresion(condicion)?;
                let salto_fin = self.emitir_salto(Instruccion::SaltarSiFalso(0));
                self.emitir(Instruccion::Saltar(inicio));
                let fin = self.pedazo.codigo.len();
                self.parchear(salto_fin, fin);
                let bucle = self.bucles.pop().ok_or_else(|| {
                    "error interno: se perdió el bucle que se estaba compilando".to_string()
                })?;
                for pos in bucle.rupturas {
                    self.parchear(pos, fin);
                }
                for pos in bucle.continuaciones {
                    self.parchear(pos, inicio_cond);
                }
                Ok(())
            }
            Declaracion::Para {
                variable,
                inicio,
                fin,
                inclusivo,
                paso,
                cuerpo,
                ..
            } => {
                // El `para` se desarma en un `mientras` con
                // variables escondidas: la que cuenta, el fin
                // ($fin) y el paso ($paso).
                self.entrar_ambito();
                self.compilar_expresion(inicio)?;
                let (slot_var, global_var, idx_var) = self.definir(variable, false);
                self.emitir(Instruccion::DefinirVariable(
                    slot_var, false, global_var, idx_var,
                ));
                self.compilar_expresion(fin)?;
                let (slot_fin, global_fin, idx_fin) = self.definir("$fin", false);
                self.emitir(Instruccion::DefinirVariable(
                    slot_fin, false, global_fin, idx_fin,
                ));
                // El paso es 1 si no se escribió.
                match paso {
                    Some(e) => self.compilar_expresion(e)?,
                    None => self.constante_numero(1.0),
                }
                let (slot_paso, global_paso, idx_paso) = self.definir("$paso", false);
                self.emitir(Instruccion::DefinirVariable(
                    slot_paso, false, global_paso, idx_paso,
                ));
                // La condición depende del signo del paso.
                // Solo se revisa al entrar: después, el avance
                // (ParaAvanzar) revisa si sigue en rango.
                let inicio_cond = self.pedazo.codigo.len();
                let cond = condicion_para_optimizada(variable, *inclusivo, paso);
                self.compilar_expresion(&cond)?;
                let salto_fin = self.emitir_salto(Instruccion::SaltarSiFalso(0));
                // Aquí empieza el cuerpo: a aquí vuelve
                // ParaAvanzar cuando el bucle sigue.
                let inicio_cuerpo = self.pedazo.codigo.len();
                self.bucles.push(InfoBucle {
                    profundidad_manejador: self.manejadores,
                    inicio: inicio_cond,
                    es_para: true,
                    es_repetir: false,
                    rupturas: Vec::new(),
                    continuaciones: Vec::new(),
                });
                self.compilar_bloque(cuerpo)?;
                // Aquí va el avance en un solo paso: variable =
                // variable + paso, y si sigue en rango, volver
                // al cuerpo. Antes eran 5 instrucciones (leer,
                // leer, sumar, definir, saltar).
                let inicio_inc = self.pedazo.codigo.len();
                self.emitir(Instruccion::ParaAvanzar(InfoParaAvanzar {
                    var_slot: slot_var,
                    var_global: global_var,
                    fin_slot: slot_fin,
                    fin_global: global_fin,
                    paso_slot: slot_paso,
                    paso_global: global_paso,
                    salto_cuerpo: inicio_cuerpo,
                    inclusivo: *inclusivo,
                }));
                let fin = self.pedazo.codigo.len();
                self.parchear(salto_fin, fin);
                let bucle = self.bucles.pop().ok_or_else(|| {
                    "error interno: se perdió el bucle que se estaba compilando".to_string()
                })?;
                for pos in bucle.rupturas {
                    self.parchear(pos, fin);
                }
                for pos in bucle.continuaciones {
                    self.parchear(pos, inicio_inc);
                }
                self.salir_ambito();
                Ok(())
            }
            Declaracion::Segun { valor, brazos, .. } => {
                // El valor se guarda en una variable escondida
                // ($segun) y se compara brazo por brazo. El
                // primer brazo que coincida corre su bloque.
                self.entrar_ambito();
                self.compilar_expresion(valor)?;
                let (slot_segun, global_segun, idx_segun) = self.definir("$segun", false);
                self.emitir(Instruccion::DefinirVariable(
                    slot_segun, false, global_segun, idx_segun,
                ));
                let mut saltos_fin = Vec::new();
                let mut salto_falso_pendiente: Option<usize> = None;
                for brazo in brazos {
                    if let Some(pos) = salto_falso_pendiente.take() {
                        self.parchear(pos, self.pedazo.codigo.len());
                    }
                    if let Some(patron) = &brazo.patron {
                        self.emitir(Instruccion::ObtenerVariable(
                            slot_segun, global_segun, idx_segun,
                        ));
                        self.compilar_expresion(patron)?;
                        self.emitir(Instruccion::Igual);
                        salto_falso_pendiente =
                            Some(self.emitir_salto(Instruccion::SaltarSiFalso(0)));
                    }
                    self.compilar_bloque(&brazo.cuerpo)?;
                    saltos_fin.push(self.emitir_salto(Instruccion::Saltar(0)));
                }
                if let Some(pos) = salto_falso_pendiente.take() {
                    self.parchear(pos, self.pedazo.codigo.len());
                }
                let pos_fin = self.pedazo.codigo.len();
                for pos in saltos_fin {
                    self.parchear(pos, pos_fin);
                }
                self.salir_ambito();
                Ok(())
            }
            Declaracion::Funcion {
                nombre,
                parametros,
                cuerpo,
                ..
            } => {
                self.compilar_funcion(nombre, parametros, cuerpo)?;
                // La función se puede llamar a partir de aquí.
                let idx = self.nombre(nombre);
                self.emitir(Instruccion::RegistrarFuncion(idx));
                Ok(())
            }
            Declaracion::Devolver { valor, .. } => {
                if !self.en_funcion {
                    return Err("no puedes usar 'devolver' fuera de una función".to_string());
                }
                match valor {
                    Some(e) => self.compilar_expresion(e)?,
                    None => self.emitir(Instruccion::Nulo),
                }
                // Al devolver se sale de todos los `intentar`
                // que rodean este punto: corren sus `al_final`.
                self.salir_de_intentars_hasta(0)?;
                self.emitir(Instruccion::Retornar);
                Ok(())
            }
            Declaracion::Romper { .. } => {
                if self.bucles.is_empty() {
                    return Err("no puedes usar 'romper' fuera de un bucle".to_string());
                }
                // Al romper se sale de los `intentar` que están
                // dentro del bucle: corren sus `al_final`.
                let nivel = self
                    .bucles
                    .last()
                    .map(|b| b.profundidad_manejador)
                    .unwrap_or(0);
                self.salir_de_intentars_hasta(nivel)?;
                let pos = self.emitir_salto(Instruccion::Saltar(0));
                if let Some(bucle) = self.bucles.last_mut() {
                    bucle.rupturas.push(pos);
                }
                Ok(())
            }
            Declaracion::Seguir { .. } => {
                let (inicio, es_para, es_repetir, nivel) = match self.bucles.last() {
                    Some(b) => (
                        b.inicio,
                        b.es_para,
                        b.es_repetir,
                        b.profundidad_manejador,
                    ),
                    None => {
                        return Err("no puedes usar 'seguir' fuera de un bucle".to_string());
                    }
                };
                // Al seguir se sale de los `intentar` que están
                // dentro del bucle: corren sus `al_final`.
                self.salir_de_intentars_hasta(nivel)?;
                if es_para || es_repetir {
                    // En un `para`, `seguir` salta al incremento.
                    // En un `repetir`, `seguir` salta a la condición.
                    let pos = self.emitir_salto(Instruccion::Saltar(0));
                    if let Some(bucle) = self.bucles.last_mut() {
                        bucle.continuaciones.push(pos);
                    }
                } else {
                    // En un `mientras`, `seguir` vuelve a la condición.
                    self.emitir(Instruccion::Saltar(inicio));
                }
                Ok(())
            }
            Declaracion::Importar { ruta, .. } => self.compilar_importar(ruta),
            Declaracion::Usar { nombre, .. } => {
                let ruta = crate::herramientas::bibliotecas::resolver_biblioteca(nombre)?;
                let ruta = ruta.to_string_lossy().to_string();
                self.compilar_importar(&ruta)
            }
            Declaracion::Intentar {
                cuerpo,
                falla,
                al_final,
                ..
            } => {
                // La instrucción Intentar va primero, con los
                // destinos todavía en blanco: se parchean al
                // final, cuando ya se sabe dónde está cada bloque.
                use crate::motor::bytecode::InfoManejador;
                let pos_intentar = self.emitir_salto(Instruccion::Intentar(InfoManejador {
                    salto_falla: None,
                    salto_final: None,
                }));
                self.manejadores += 1;
                // El al_final que rodea este punto (para
                // devolver, romper y seguir): si este intentar
                // tiene al_final, se apunta; si no, se apunta
                // None (hay que quitar el manejador igual, pero
                // no hay nada que correr).
                self.finallys.push(al_final.clone());
                self.compilar_bloque(cuerpo)?;
                self.finallys.pop();
                // Salida normal: quitar el manejador, correr el
                // al_final y saltar al final.
                self.emitir(Instruccion::QuitarManejador);
                self.manejadores -= 1;
                if let Some(final_cuerpo) = al_final {
                    self.compilar_bloque(final_cuerpo)?;
                }
                let salto_despues = self.emitir_salto(Instruccion::Saltar(0));
                // El si_falla: la máquina ya puso el error en
                // la pila, para guardarlo en su variable.
                let mut salto_falla = None;
                if let Some((nombre_error, cuerpo_falla)) = falla {
                    salto_falla = Some(self.pedazo.codigo.len());
                    match nombre_error {
                        Some(nombre_var) => {
                            // El error va en su variable, dentro
                            // del ámbito del si_falla.
                            self.entrar_ambito();
                            let (slot, global, idx) = self.definir(nombre_var, false);
                            self.emitir(Instruccion::DefinirVariable(
                                slot, false, global, idx,
                            ));
                            self.compilar_bloque(cuerpo_falla)?;
                            self.salir_ambito();
                        }
                        None => {
                            self.emitir(Instruccion::Sacar);
                            self.compilar_bloque(cuerpo_falla)?;
                        }
                    }
                    // Después del si_falla, corre el al_final
                    // y se sigue con el programa.
                    if let Some(final_cuerpo) = al_final {
                        self.compilar_bloque(final_cuerpo)?;
                    }
                }
                // Sin si_falla: el al_final corre en el camino
                // del error y después se vuelve a soltar el
                // mismo error.
                let mut salto_final = None;
                if falla.is_none() {
                    if let Some(final_cuerpo) = al_final {
                        salto_final = Some(self.pedazo.codigo.len());
                        self.compilar_bloque(final_cuerpo)?;
                        self.emitir(Instruccion::Relanzar);
                    }
                }
                let pos_despues = self.pedazo.codigo.len();
                self.parchear(salto_despues, pos_despues);
                self.parchear_intentar(pos_intentar, salto_falla, salto_final);
                Ok(())
            }
            Declaracion::Bloque { declaraciones: cuerpo, .. } => self.compilar_bloque(cuerpo),
        }
    }

    // Pone los destinos en una instrucción Intentar que se
    // emitió antes con los destinos en blanco.
    fn parchear_intentar(
        &mut self,
        pos: usize,
        salto_falla: Option<usize>,
        salto_final: Option<usize>,
    ) {
        if let Some(Instruccion::Intentar(info)) = self.pedazo.codigo.get_mut(pos) {
            info.salto_falla = salto_falla;
            info.salto_final = salto_final;
        }
    }

    // Quita manejadores hasta dejar `nivel`: se usa al salir
    // de un `intentar` por un salto (devolver, romper, seguir).
    fn quitar_manejadores_hasta(&mut self, nivel: usize) {
        while self.manejadores > nivel {
            self.emitir(Instruccion::QuitarManejador);
            self.manejadores -= 1;
        }
    }

    // Corre los `al_final` de los `intentar` que rodean este
    // punto, del más adentro al más afuera, y quita sus
    // manejadores. `nivel` dice cuántos se quedan (los de
    // afuera): al devolver es 0 (todos), al romper o seguir
    // es los que había cuando empezó el bucle (solo los de
    // dentro del bucle). El estado del compilador queda como
    // estaba: esto solo afecta al camino del salto, no a las
    // órdenes que sigan después en el mismo bloque.
    fn salir_de_intentars_hasta(&mut self, nivel: usize) -> Result<(), String> {
        let total = self.finallys.len();
        if nivel >= total {
            return Ok(());
        }
        // Sacar el trozo que se va a correr: mientras se
        // compila, un devolver/romper/seguir dentro de un
        // al_final no lo vuelve a correr (el trozo ya no
        // está en la pila, solo quedan los de afuera).
        let guardados: Vec<Option<Vec<Declaracion>>> = self.finallys.split_off(nivel);
        let manejadores_viejos = self.manejadores;
        for cuerpo in guardados.iter().rev() {
            self.quitar_manejadores_hasta(self.manejadores.saturating_sub(1));
            if let Some(cuerpo) = cuerpo {
                self.compilar_bloque(cuerpo)?;
            }
        }
        self.finallys.extend(guardados);
        self.manejadores = manejadores_viejos;
        Ok(())
    }

    // Compila una función en su propio pedazo, aparte.
    // Guarda el pedazo actual, compila el cuerpo en uno
    // nuevo, y restaura el pedazo actual al terminar.
    // Los parámetros son los primeros slots del marco.
    fn compilar_funcion(
        &mut self,
        nombre: &str,
        parametros: &[String],
        cuerpo: &[Declaracion],
    ) -> Result<(), String> {
        let pedazo_viejo = std::mem::replace(&mut self.pedazo, Pedazo::nuevo());
        let bucles_viejos = std::mem::take(&mut self.bucles);
        let manejadores_viejos = self.manejadores;
        let finallys_viejos = std::mem::take(&mut self.finallys);
        self.manejadores = 0;
        self.finallys = Vec::new();
        let locales_viejos = std::mem::take(&mut self.locales);
        let siguiente_slot_viejo = self.siguiente_slot;
        let max_slots_viejo = self.max_slots;
        let en_funcion_viejo = self.en_funcion;
        // La función empieza con un ámbito base vacío.
        self.locales = vec![(HashMap::new(), 0)];
        self.siguiente_slot = 0;
        self.max_slots = 0;
        self.en_funcion = true;
        self.pedazo.parametros = parametros.to_vec();
        // Los parámetros son los primeros slots: la máquina
        // virtual los pone ahí al llamar a la función.
        for param in parametros {
            self.definir(param, false);
        }
        for d in cuerpo {
            self.compilar_declaracion(d)?;
        }
        // Si la función no devuelve nada, devuelve nulo.
        self.emitir(Instruccion::Nulo);
        self.emitir(Instruccion::Retornar);
        let mut pedazo_funcion = std::mem::replace(&mut self.pedazo, pedazo_viejo);
        pedazo_funcion.max_slots = self.max_slots;
        self.bucles = bucles_viejos;
        self.manejadores = manejadores_viejos;
        self.finallys = finallys_viejos;
        self.locales = locales_viejos;
        self.siguiente_slot = siguiente_slot_viejo;
        self.max_slots = max_slots_viejo;
        self.en_funcion = en_funcion_viejo;
        self.funciones.insert(nombre.to_string(), pedazo_funcion);
        Ok(())
    }

    // Carga otro archivo .ec y compila sus órdenes aquí mismo.
    // Si ya se importó antes, no lo vuelve a compilar.
    fn compilar_importar(&mut self, ruta: &str) -> Result<(), String> {
        let completa = if Path::new(ruta).is_absolute() {
            PathBuf::from(ruta)
        } else {
            self.directorio_base.join(ruta)
        };
        let canonica = fs::canonicalize(&completa)
            .map_err(|_| format!("no encontré el archivo '{}'", ruta))?;
        if self.importados.contains(&canonica) {
            return Ok(());
        }
        self.importados.insert(canonica.clone());
        let fuente = fs::read_to_string(&canonica)
            .map_err(|_| format!("no pude leer el archivo '{}'", ruta))?;
        let tokens = Lexer::nuevo(&fuente).tokenizar()?;
        let declaraciones = Parser::nuevo(tokens).parsear()?;
        // Los importar de este archivo se resuelven desde SU
        // carpeta (como el transpilador), no desde la del
        // programa principal.
        let base_vieja = self.directorio_base.clone();
        if let Some(padre) = canonica.parent() {
            self.directorio_base = padre.to_path_buf();
        }
        for d in &declaraciones {
            self.compilar_declaracion(d)?;
        }
        self.directorio_base = base_vieja;
        Ok(())
    }

    // Compila una expresión: deja su valor en la pila.
    // Doblado de constantes: si la expresión es solo valores
    // fijos, se calcula aquí mismo y queda un solo valor.
    fn compilar_expresion(&mut self, e: &Expresion) -> Result<(), String> {
        if let Some(v) = evaluar_constante(e) {
            self.emitir_constante(v);
            return Ok(());
        }
        match e {
            Expresion::Numero(n) => {
                self.constante_numero(*n);
                Ok(())
            }
            Expresion::Cadena(s) => {
                let idx = self.pedazo.agregar_constante(Valor::Texto(s.clone()));
                self.emitir(Instruccion::Constante(idx));
                Ok(())
            }
            Expresion::Logico(true) => {
                self.emitir(Instruccion::Verdadero);
                Ok(())
            }
            Expresion::Logico(false) => {
                self.emitir(Instruccion::Falso);
                Ok(())
            }
            Expresion::Nulo => {
                self.emitir(Instruccion::Nulo);
                Ok(())
            }
            Expresion::Variable(nombre) => {
                let (slot, global, idx) = self.variable(nombre)?;
                self.emitir(Instruccion::ObtenerVariable(slot, global, idx));
                Ok(())
            }
            Expresion::Agrupada(interior) => self.compilar_expresion(interior),
            Expresion::Unaria { operador, derecha } => {
                self.compilar_expresion(derecha)?;
                match operador {
                    OperadorUnario::Negativo => self.emitir(Instruccion::Negativo),
                    OperadorUnario::No => self.emitir(Instruccion::No),
                }
                Ok(())
            }
            Expresion::Binaria {
                izquierda,
                operador,
                derecha,
            } => {
                match operador {
                    // `y`: si el primero es falso, el resultado es
                    // falso sin mirar el segundo (cortocircuito).
                    OperadorBinario::Y => {
                        self.compilar_expresion(izquierda)?;
                        let salto_falso = self.emitir_salto(Instruccion::SaltarSiFalso(0));
                        self.compilar_expresion(derecha)?;
                        self.emitir(Instruccion::ConvertirLogico);
                        let salto_fin = self.emitir_salto(Instruccion::Saltar(0));
                        self.parchear(salto_falso, self.pedazo.codigo.len());
                        self.emitir(Instruccion::Falso);
                        self.parchear(salto_fin, self.pedazo.codigo.len());
                        Ok(())
                    }
                    // `o`: si el primero es verdadero, el resultado
                    // es verdadero sin mirar el segundo.
                    OperadorBinario::O => {
                        self.compilar_expresion(izquierda)?;
                        let salto_falso = self.emitir_salto(Instruccion::SaltarSiFalso(0));
                        self.emitir(Instruccion::Verdadero);
                        let salto_fin = self.emitir_salto(Instruccion::Saltar(0));
                        self.parchear(salto_falso, self.pedazo.codigo.len());
                        self.compilar_expresion(derecha)?;
                        self.emitir(Instruccion::ConvertirLogico);
                        self.parchear(salto_fin, self.pedazo.codigo.len());
                        Ok(())
                    }
                    _ => {
                        self.compilar_expresion(izquierda)?;
                        self.compilar_expresion(derecha)?;
                        let instr = match operador {
                            OperadorBinario::Sumar => Instruccion::Sumar,
                            OperadorBinario::Restar => Instruccion::Restar,
                            OperadorBinario::Multiplicar => Instruccion::Multiplicar,
                            OperadorBinario::Dividir => Instruccion::Dividir,
                            OperadorBinario::Residuo => Instruccion::Residuo,
                            OperadorBinario::Potencia => Instruccion::Potencia,
                            OperadorBinario::Igual => Instruccion::Igual,
                            OperadorBinario::Diferente => Instruccion::Diferente,
                            OperadorBinario::Menor => Instruccion::Menor,
                            OperadorBinario::MenorIgual => Instruccion::MenorIgual,
                            OperadorBinario::Mayor => Instruccion::Mayor,
                            OperadorBinario::MayorIgual => Instruccion::MayorIgual,
                            OperadorBinario::Y | OperadorBinario::O => unreachable!(),
                        };
                        self.emitir(instr);
                        Ok(())
                    }
                }
            }
            Expresion::Llamada { nombre, argumentos } => {
                for arg in argumentos {
                    self.compilar_expresion(arg)?;
                }
                // Ownership: si le pasas un texto de una variable
                // a una función tuya, esa variable se mueve.
                // Las funciones que trae Eclipse solo miran el
                // texto, así que con ellas no se mueve nada.
                if !es_builtin(nombre) {
                    for arg in argumentos {
                        if let Expresion::Variable(fuente) = arg {
                            let (slot, global, idx) = self.variable(fuente)?;
                            let destino = format!("a la función '{}'", nombre);
                            let idx_destino = self.nombre(&destino);
                            self.emitir(Instruccion::Mover(slot, global, idx, idx_destino));
                        }
                    }
                    // &mut: préstamos para cambiar. Se anota qué
                    // variable se presta y a qué dato va, para que
                    // la máquina virtual escriba el valor nuevo de
                    // vuelta cuando la función termine.
                    for (i, arg) in argumentos.iter().enumerate() {
                        if let Expresion::PrestarMut(interior) = arg {
                            if let Expresion::Variable(fuente) = &**interior {
                                let (slot, global, idx) = self.variable(fuente)?;
                                self.emitir(Instruccion::PrestarMut(slot, global, idx, i));
                            }
                        }
                        // &: préstamos solo para mirar. Se anota
                        // qué dato es, para que la máquina virtual
                        // lo marque como prestado (y no se pueda
                        // mover mientras corre la función).
                        if let Expresion::Prestar(interior) = arg {
                            if let Expresion::Variable(fuente) = &**interior {
                                let (slot, global, idx) = self.variable(fuente)?;
                                self.emitir(Instruccion::Prestar(slot, global, idx, i));
                            }
                        }
                    }
                }
                let idx = self.nombre(nombre);
                self.emitir(Instruccion::Llamar(idx, argumentos.len()));
                Ok(())
            }
            // "Hola, {nombre}": compila cada parte y las pega
            // con Sumar (que pega textos con a_texto).
            Expresion::Interpolada(partes) => {
                for (i, parte) in partes.iter().enumerate() {
                    match parte {
                        ParteCadena::Texto(s) => {
                            let idx =
                                self.pedazo.agregar_constante(Valor::Texto(s.clone()));
                            self.emitir(Instruccion::Constante(idx));
                        }
                        ParteCadena::Expresion(e) => self.compilar_expresion(e)?,
                    }
                    if i > 0 {
                        self.emitir(Instruccion::Sumar);
                    }
                }
                Ok(())
            }
            // [1, 2, 3]: compila cada elemento y arma la lista.
            Expresion::Lista(elementos) => {
                for e in elementos {
                    self.compilar_expresion(e)?;
                }
                self.emitir(Instruccion::CrearLista(elementos.len()));
                Ok(())
            }
            Expresion::Mapa(pares) => {
                for (clave, valor) in pares {
                    self.compilar_expresion(clave)?;
                    self.compilar_expresion(valor)?;
                }
                self.emitir(Instruccion::CrearMapa(pares.len()));
                Ok(())
            }
            // mi_lista[0]: compila la lista y el índice, y saca
            // el elemento con la instrucción Indice.
            Expresion::Indice { lista, indice } => {
                self.compilar_expresion(lista)?;
                self.compilar_expresion(indice)?;
                self.emitir(Instruccion::Indice);
                Ok(())
            }
            // &nombre: prestar, no mover. Solo se calcula el
            // valor; la regla de mover no se aplica aquí.
            Expresion::Prestar(interior) => self.compilar_expresion(interior),
            // &mut nombre: prestar para cambiar. Solo se calcula
            // el valor; la escritura de vuelta la hace la máquina
            // virtual con la instrucción PrestarMut.
            Expresion::PrestarMut(interior) => self.compilar_expresion(interior),
        }
    }
}
