// ============================================================
//  ECLIPSE 1.2 - El bytecode
//  Archivo: src/bytecode.rs
//
//  Qué es esto:
//  El bytecode es la lista de instrucciones chiquitas en las
//  que el compilador traduce tu programa. Es como una receta
//  paso a paso para la máquina virtual.
//
//  Ejemplo, para:  sea x = 10 + 5;
//      0000 Constante 0 (15)        <- ya doblado: 10 + 5 = 15
//      0001 DefinirVariable x en slot 0 (global)
//
//  Un Pedazo es una de esas listas, con su tabla de
//  constantes (los números y textos que usa), su tabla de
//  nombres (cada nombre guardado UNA sola vez, para gastar
//  poca RAM) y sus parámetros (si es una función). También
//  guarda cuántos slots (cajones numerados) necesita.
//
//  Las variables ya no se buscan por su nombre: cada una
//  tiene un numerito de slot. Las variables globales (las
//  que creas arriba del todo) viven en un array compartido;
//  las demás viven en los slots del marco de la función
//  que está corriendo. Buscar por numerito es mucho más
//  rápido que buscar por nombre en un mapa, y gasta menos RAM.
//
//  Un Programa es el pedazo principal más los pedazos
//  de cada función.
//
//  Para ver el bytecode de un programa:
//      cargo run -- --bytecode ejemplos/hola.ec
// ============================================================

use crate::motor::valor::Valor;
use std::collections::HashMap;

// Una instrucción es un paso chiquito que la máquina
// virtual sabe hacer. La máquina tiene una pila: casi
// todas las instrucciones sacan valores de arriba de
// la pila, trabajan con ellos y ponen el resultado.
//
// Los nombres (variables y funciones) no van escritos en
// cada instrucción: van como numeritos que apuntan a la
// tabla de nombres del pedazo. Así un nombre largo como
// "mi_variable_favorita" se guarda una sola vez aunque
// se use cien veces: poca RAM.
//
// Las variables se guardan por slot (numerito):
//  - slot: el cajón numerado donde vive la variable.
//  - global: verdadero si es una variable global (creada
//    arriba del todo, visible para todas las funciones),
//    falso si es local (de la función que está corriendo).
//  - nombre: el numerito del nombre en la tabla, solo para
//    que el --bytecode se pueda leer con tus ojos.
#[derive(Debug, Clone, Copy)]
pub enum Instruccion {
    Constante(usize), // Poner una constante de la tabla en la pila
    Nulo,             // Poner nulo en la pila
    Verdadero,        // Poner verdadero en la pila
    Falso,            // Poner falso en la pila
    Sumar,            // Sacar dos valores, sumarlos, poner el resultado
    Restar,
    Multiplicar,
    Dividir,
    Residuo,
    Potencia, // Sacar dos valores, elevar uno al otro (2 ** 3 = 8)
    Negativo, // Cambiar el signo del valor de arriba
    No,       // Negar el valor de arriba
    ConvertirLogico, // Convertir el valor de arriba a verdadero/falso
    Igual,
    Diferente,
    Menor,
    MenorIgual,
    Mayor,
    MayorIgual,
    Sacar, // Tirar el valor de arriba de la pila
    // slot, mutable, global, nombre
    DefinirVariable(usize, bool, bool, usize),
    // slot, global, nombre
    // Una constante: nunca se puede cambiar ni se mueve.
    DefinirConstante(usize, bool, usize),
    // slot, global, nombre
    ObtenerVariable(usize, bool, usize),
    // slot, global, nombre
    AsignarVariable(usize, bool, usize),
    // slot, global, nombre, destino (a dónde se mueve)
    Mover(usize, bool, usize, usize),
    // slot, global, nombre, param (qué dato de la función es)
    // Presta la variable para cambiar: la función la recibe y
    // cuando termina, el valor nuevo vuelve a tu variable.
    // Sirve para números, lógicos y listas.
    PrestarMut(usize, bool, usize, usize),
    // slot, global, nombre, param (qué dato de la función es)
    // Presta la variable solo para mirar: la función la recibe
    // pero no puede moverla a otro dueño (el préstamo sigue
    // activo mientras corre la función).
    Prestar(usize, bool, usize, usize),
    // tipo, slot, global, nombre
    VerificarTipo(usize, usize, bool, usize),
    // Sacar n valores y poner una lista con ellos.
    CrearLista(usize),
    // Sacar el índice y la lista, poner el elemento.
    Indice,
    // slot, global, nombre, mensaje (el índice del mensaje
    // en la tabla de constantes, o nada si no hay mensaje)
    // Input: leer lo que escribe el usuario y guardarlo en
    // la variable (convirtiendo a número o lógico si la
    // variable ya era de ese tipo, o separando por comas
    // si era una lista). Con mensaje, lo muestra antes.
    LeerEntrada(usize, bool, usize, Option<usize>),
    // slot, global, nombre, subir (verdadero = ++, falso = --)
    // Sumar o restar 1 a un número.
    Incrementar(usize, bool, usize, bool),
    Saltar(usize),        // Saltar a otra instrucción
    SaltarSiFalso(usize), // Sacar un valor; si es falso, saltar
    Llamar(usize, usize), // Llamar a una función con estos datos
    RegistrarFuncion(usize), // Hacer que una función se pueda llamar
    Retornar,             // Salir de la función con el valor de arriba
    // slot, global, nombre
    // Revisar que una variable sea una lista (para `para_cada`):
    // si no lo es, da error en español.
    VerificarLista(usize, bool, usize),
    // Avanzar un bucle `para` en un solo paso: suma el paso a
    // la variable, revisa si sigue en rango y, si sí, vuelve
    // al cuerpo. Sin esta instrucción, cada vuelta gastaba 5
    // instrucciones (leer, leer, sumar, definir, saltar).
    ParaAvanzar(InfoParaAvanzar),
    // slot, global, nombre
    // Meter un valor al final de una lista, en su sitio:
    // es el atajo de `x = agregar(x, v)`. Si la lista solo
    // la tiene esta variable, no se clona nada; si otro
    // también la tiene, se clona (la otra no cambia).
    AgregarEnSitio(usize, bool, usize),
    // Sacar 2n valores (clave, valor, clave, valor...) y
    // poner un mapa con ellos.
    CrearMapa(usize),
    // slot, global, nombre
    // Cambiar un lugar de una lista o un mapa, en su sitio:
    // lista[0] = 5;  mapa[21] = "FTP";  (la clave y el valor
    // nuevo vienen en la pila). Si otro también tiene esa
    // lista o ese mapa, no se toca el suyo: se clona.
    AsignarIndice(usize, bool, usize),
    // Empujar un manejador para `intentar`: si algo falla
    // de aquí en adelante, la máquina salta al `si_falla`
    // (o al `al_final`, si no hay `si_falla`) en vez de
    // parar el programa.
    Intentar(InfoManejador),
    // Quitar el último manejador (el `intentar` salió bien).
    QuitarManejador,
    // Volver a soltar el error que se guardó al entrar al
    // `al_final` de un error sin `si_falla`.
    Relanzar,
}

/// Los datos de la instrucción Intentar: a dónde saltar si
/// algo falla (el `si_falla`, si lo hay) y a dónde saltar
/// para correr el `al_final` cuando no hay `si_falla`.
/// `None` significa que esa parte no existe.
#[derive(Debug, Clone, Copy)]
pub struct InfoManejador {
    pub salto_falla: Option<usize>,
    pub salto_final: Option<usize>,
}

/// Los datos de la instrucción ParaAvanzar: dónde está la
/// variable del bucle, el fin y el paso, a dónde volver si
/// sigue en rango, y si el fin se incluye.
#[derive(Debug, Clone, Copy)]
pub struct InfoParaAvanzar {
    pub var_slot: usize,
    pub var_global: bool,
    pub fin_slot: usize,
    pub fin_global: bool,
    pub paso_slot: usize,
    pub paso_global: bool,
    pub salto_cuerpo: usize,
    pub inclusivo: bool,
}

#[derive(Debug, Clone)]
pub struct Pedazo {
    pub codigo: Vec<Instruccion>,
    pub constantes: Vec<Valor>,
    pub nombres: Vec<String>,
    pub parametros: Vec<String>,
    // Cuántos slots locales necesita este pedazo como máximo.
    pub max_slots: usize,
}

impl Pedazo {
    pub fn nuevo() -> Self {
        Pedazo {
            codigo: Vec::new(),
            constantes: Vec::new(),
            nombres: Vec::new(),
            parametros: Vec::new(),
            max_slots: 0,
        }
    }

    // Guarda una constante en la tabla y devuelve su número.
    // Si ya estaba, devuelve el mismo número (así no se repite).
    pub fn agregar_constante(&mut self, valor: Valor) -> usize {
        if let Some(i) = self.constantes.iter().position(|c| *c == valor) {
            return i;
        }
        self.constantes.push(valor);
        self.constantes.len() - 1
    }

    // Guarda un nombre en la tabla y devuelve su número.
    // Si ya estaba, devuelve el mismo número: cada nombre
    // se guarda una sola vez, aunque se use cien veces.
    pub fn internar(&mut self, nombre: &str) -> usize {
        if let Some(i) = self.nombres.iter().position(|n| n == nombre) {
            return i;
        }
        self.nombres.push(nombre.to_string());
        self.nombres.len() - 1
    }

    // El nombre que hay detrás de un numerito.
    pub fn nombre(&self, indice: usize) -> String {
        self.nombres
            .get(indice)
            .cloned()
            .unwrap_or_else(|| "?".to_string())
    }

    // Quita los saltos que van a la instrucción siguiente:
    // saltar al siguiente paso es lo mismo que no saltar.
    // Repara los destinos de los saltos que quedan.
    pub fn optimizar(&mut self) {
        let total = self.codigo.len();
        let mut quitar = vec![false; total];
        for (i, instr) in self.codigo.iter().enumerate() {
            if let Instruccion::Saltar(destino) = instr {
                if *destino == i + 1 {
                    quitar[i] = true;
                }
            }
        }
        if !quitar.iter().any(|q| *q) {
            return;
        }
        let mut nuevo_indice = vec![0usize; total + 1];
        let mut nuevo_codigo = Vec::new();
        for (i, instr) in self.codigo.iter().enumerate() {
            nuevo_indice[i] = nuevo_codigo.len();
            if !quitar[i] {
                nuevo_codigo.push(*instr);
            }
        }
        nuevo_indice[total] = nuevo_codigo.len();
        for instr in nuevo_codigo.iter_mut() {
            match instr {
                Instruccion::Saltar(d) | Instruccion::SaltarSiFalso(d) => {
                    *d = nuevo_indice[*d];
                }
                // ParaAvanzar también salta: si no se mueve su
                // destino, el bucle volvería una instrucción
                // tarde y la pila se quedaría vacía.
                Instruccion::ParaAvanzar(info) => {
                    info.salto_cuerpo = nuevo_indice[info.salto_cuerpo];
                }
                // Intentar también salta: si no se mueven sus
                // destinos, un error caería en el lugar viejo.
                Instruccion::Intentar(info) => {
                    if let Some(d) = info.salto_falla {
                        info.salto_falla = Some(nuevo_indice[d]);
                    }
                    if let Some(d) = info.salto_final {
                        info.salto_final = Some(nuevo_indice[d]);
                    }
                }
                _ => {}
            }
        }
        self.codigo = nuevo_codigo;
    }

    // Muestra el bytecode como una lista numerada, para que
    // puedas ver qué hizo el compilador con tu programa.
    pub fn mostrar(&self, nombre: &str) {
        println!("== {} ==", nombre);
        for (i, instr) in self.codigo.iter().enumerate() {
            match instr {
                Instruccion::Constante(idx) => {
                    let valor = self
                        .constantes
                        .get(*idx)
                        .map(mostrar_valor)
                        .unwrap_or_else(|| "?".to_string());
                    println!("{:04} Constante {} ({})", i, idx, valor);
                }
                Instruccion::DefinirVariable(slot, m, global, n) => println!(
                    "{:04} DefinirVariable {} en slot {} ({}, {})",
                    i,
                    self.nombre(*n),
                    slot,
                    if *global { "global" } else { "local" },
                    if *m { "cambia" } else { "fija" }
                ),
                Instruccion::DefinirConstante(slot, global, n) => println!(
                    "{:04} DefinirConstante {} en slot {} ({})",
                    i,
                    self.nombre(*n),
                    slot,
                    if *global { "global" } else { "local" }
                ),
                Instruccion::CrearLista(n) => {
                    println!("{:04} CrearLista con {} {}", i, n, if *n == 1 { "elemento" } else { "elementos" })
                }
                Instruccion::Indice => println!("{:04} Indice", i),
                Instruccion::LeerEntrada(slot, global, n, mensaje) => println!(
                    "{:04} LeerEntrada en {} del slot {} ({}){}",
                    i,
                    self.nombre(*n),
                    slot,
                    if *global { "global" } else { "local" },
                    match mensaje {
                        Some(m) => format!(
                            " con mensaje {}",
                            self.constantes
                                .get(*m)
                                .map(mostrar_valor)
                                .unwrap_or_else(|| "?".to_string())
                        ),
                        None => String::new(),
                    }
                ),
                Instruccion::Incrementar(slot, global, n, subir) => println!(
                    "{:04} Incrementar {} en slot {} ({}) {}",
                    i,
                    self.nombre(*n),
                    slot,
                    if *global { "global" } else { "local" },
                    if *subir { "++" } else { "--" }
                ),
                Instruccion::ObtenerVariable(slot, global, n) => println!(
                    "{:04} ObtenerVariable {} del slot {} ({})",
                    i,
                    self.nombre(*n),
                    slot,
                    if *global { "global" } else { "local" }
                ),
                Instruccion::AsignarVariable(slot, global, n) => println!(
                    "{:04} AsignarVariable {} en slot {} ({})",
                    i,
                    self.nombre(*n),
                    slot,
                    if *global { "global" } else { "local" }
                ),
                Instruccion::Mover(slot, global, n, destino) => println!(
                    "{:04} Mover {} del slot {} ({}) -> {}",
                    i,
                    self.nombre(*n),
                    slot,
                    if *global { "global" } else { "local" },
                    self.nombre(*destino)
                ),
                Instruccion::PrestarMut(slot, global, n, param) => println!(
                    "{:04} PrestarMut {} del slot {} ({}) -> dato {} de la función",
                    i,
                    self.nombre(*n),
                    slot,
                    if *global { "global" } else { "local" },
                    param
                ),
                Instruccion::Prestar(slot, global, n, param) => println!(
                    "{:04} Prestar {} del slot {} ({}) -> dato {} de la función",
                    i,
                    self.nombre(*n),
                    slot,
                    if *global { "global" } else { "local" },
                    param
                ),
                Instruccion::VerificarTipo(t, slot, global, n) => println!(
                    "{:04} VerificarTipo {} para {} en slot {} ({})",
                    i,
                    self.nombre(*t),
                    self.nombre(*n),
                    slot,
                    if *global { "global" } else { "local" }
                ),
                Instruccion::VerificarLista(slot, global, n) => println!(
                    "{:04} VerificarLista {} en slot {} ({})",
                    i,
                    self.nombre(*n),
                    slot,
                    if *global { "global" } else { "local" }
                ),
                Instruccion::ParaAvanzar(info) => println!(
                    "{:04} ParaAvanzar var slot {} fin slot {} paso slot {} -> {} ({})",
                    i,
                    info.var_slot,
                    info.fin_slot,
                    info.paso_slot,
                    info.salto_cuerpo,
                    if info.inclusivo { "incluye fin" } else { "sin fin" }
                ),
                Instruccion::AgregarEnSitio(slot, global, n) => println!(
                    "{:04} AgregarEnSitio {} en slot {} ({})",
                    i,
                    self.nombre(*n),
                    slot,
                    if *global { "global" } else { "local" }
                ),
                Instruccion::CrearMapa(n) => {
                    println!("{:04} CrearMapa con {} {}", i, n, if *n == 1 { "par" } else { "pares" })
                }
                Instruccion::AsignarIndice(slot, global, n) => println!(
                    "{:04} AsignarIndice {} en slot {} ({})",
                    i,
                    self.nombre(*n),
                    slot,
                    if *global { "global" } else { "local" }
                ),
                Instruccion::Intentar(info) => println!(
                    "{:04} Intentar -> falla {:?} final {:?}",
                    i, info.salto_falla, info.salto_final
                ),
                Instruccion::QuitarManejador => println!("{:04} QuitarManejador", i),
                Instruccion::Relanzar => println!("{:04} Relanzar", i),
                Instruccion::Saltar(d) => println!("{:04} Saltar -> {}", i, d),
                Instruccion::SaltarSiFalso(d) => {
                    println!("{:04} SaltarSiFalso -> {}", i, d)
                }
                Instruccion::Llamar(n, a) => println!(
                    "{:04} Llamar {} con {} {}",
                    i,
                    self.nombre(*n),
                    a,
                    if *a == 1 { "dato" } else { "datos" }
                ),
                Instruccion::RegistrarFuncion(n) => {
                    println!("{:04} RegistrarFuncion {}", i, self.nombre(*n))
                }
                otra => println!("{:04} {:?}", i, otra),
            }
        }
        if !self.constantes.is_empty() {
            println!("-- constantes --");
            for (i, c) in self.constantes.iter().enumerate() {
                println!("  {} = {}", i, mostrar_valor(c));
            }
        }
        if self.max_slots > 0 {
            println!("-- slots locales: {} --", self.max_slots);
        }
    }
}

// Cómo se ve un valor en el bytecode: los textos con comillas.
pub fn mostrar_valor(v: &Valor) -> String {
    match v {
        Valor::Texto(s) => format!("\"{}\"", s),
        otro => otro.a_texto(),
    }
}

#[derive(Debug)]
pub struct Programa {
    pub principal: Pedazo,
    pub funciones: HashMap<String, Pedazo>,
}

impl Programa {
    pub fn mostrar(&self) {
        self.principal.mostrar("principal");
        let mut nombres: Vec<&String> = self.funciones.keys().collect();
        nombres.sort();
        for nombre in nombres {
            println!();
            self.funciones[nombre].mostrar(nombre);
        }
    }
}
