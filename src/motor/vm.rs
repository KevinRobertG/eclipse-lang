// ============================================================
//  ECLIPSE 1.2 - La máquina virtual
//  Archivo: src/vm.rs
//
//  Qué hace:
//  La máquina virtual corre el bytecode que hizo el
//  compilador. Es como un cocinero que sigue la receta
//  paso a paso: cada instrucción saca valores de la pila,
//  trabaja con ellos y pone el resultado.
//
//  Cómo guarda las cosas:
//  - La pila: una torre de valores. Casi todas las
//    instrucciones sacan de arriba y ponen el resultado.
//  - Las variables globales: un array compartido, donde
//    cada variable tiene su numerito de slot.
//  - Los marcos: cada función que se está ejecutando tiene
//    su marco, con sus slots locales (otra vez numeritos).
//    Buscar por numerito es mucho más rápido que buscar
//    por nombre en un mapa, y gasta menos RAM.
//  - La pila nunca se pasa de 10,000 valores, nunca hay
//    más de 1000 funciones anidadas, y nunca se ejecutan
//    más de 10,000,000 de instrucciones: si un programa se
//    vuelve loco, la máquina lo para sola con un error en
//    español.
//
//  Ownership (como Rust): los textos se mueven. Cuando una
//  variable se mueve, la máquina anota A DÓNDE se fue, para
//  que el error te diga dónde buscarla. Los números y los
//  lógicos son fotocopias: no se mueven nunca.
//
//  La máquina virtual hace exactamente lo mismo que el
//  intérprete, solo que por otro camino. Por eso todo lo
//  que corre en el intérprete corre igual aquí.
// ============================================================

use crate::lenguaje::ast::OperadorBinario;
use crate::motor::bytecode::{Instruccion, Pedazo, Programa};
use crate::motor::valor::{aplicar_binario, Valor};
use std::collections::HashMap;
use std::io::{self, Write};
use std::rc::Rc;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

// Límites de estabilidad: si un programa se pasa de
// estos números, la máquina lo para con un error en español.
const MAX_MARCOS: usize = 1000;
const MAX_PILA: usize = 10_000;
const MAX_INSTRUCCIONES: usize = 100_000_000;

// Una variable en su cajón (slot): su valor, si se puede
// cambiar, si es una constante (nunca cambia ni se mueve),
// si está prestada a una función (no se puede mover
// mientras dure el préstamo), y a dónde se movió.
#[derive(Debug, Clone)]
struct Celda {
    valor: Valor,
    mutable: bool,
    constante: bool,
    prestado: bool,
    movido: Option<String>,
}

impl Celda {
    // Una variable nueva: no es constante, no está prestada
    // y no se ha movido.
    fn nueva(valor: Valor, mutable: bool) -> Self {
        Celda {
            valor,
            mutable,
            constante: false,
            prestado: false,
            movido: None,
        }
    }

    // Una constante: nunca se puede cambiar ni se mueve.
    fn constante(valor: Valor) -> Self {
        Celda {
            valor,
            mutable: false,
            constante: true,
            prestado: false,
            movido: None,
        }
    }
}

// Un marco es una función que se está ejecutando: su pedazo
// de bytecode, dónde va (ip), dónde empieza su parte de la
// pila (base_pila) y sus slots locales.
struct Marco {
    pedazo: Rc<Pedazo>,
    ip: usize,
    base_pila: usize,
    slots: Vec<Celda>,
    // Préstamos para cambiar (&mut) que hay que escribir de
    // vuelta cuando esta función termine.
    devoluciones: Vec<Prestamo>,
    nombre: String,
}

// Un manejador para `intentar`: hasta dónde estaba la
// pila y cuántos marcos había cuando se abrió el intentar,
// y a dónde saltar si algo falla (el `si_falla`, si lo hay)
// o para correr el `al_final` (si no hay `si_falla`).
struct Manejador {
    pila: usize,
    marcos: usize,
    salto_falla: Option<usize>,
    salto_final: Option<usize>,
}

// Un préstamo para cambiar: la variable que se prestó
// (slot/global/nombre), a qué dato de la función va (param)
// y qué tipo tenía cuando se prestó.
struct Prestamo {
    slot: usize,
    global: bool,
    nombre_idx: usize,
    param: usize,
    tipo_original: String,
}

pub struct MaquinaVirtual {
    funciones: HashMap<String, Rc<Pedazo>>,
    registradas: HashMap<String, Rc<Pedazo>>,
    pila: Vec<Valor>,
    // Variables globales: un cajón por variable.
    globales: Vec<Celda>,
    marcos: Vec<Marco>,
    instrucciones: usize,
    semilla: u64,
    // Préstamos &mut anotados, esperando a la próxima llamada.
    prestamos_pendientes: Vec<Prestamo>,
    // Préstamos & (solo mirar) anotados, esperando a la
    // próxima llamada: las posiciones de los datos.
    prestados_pendientes: Vec<usize>,
    // Manejadores de `intentar` abiertos (el último es el
    // más adentro) y el error que hay que volver a soltar
    // después de correr un `al_final` sin `si_falla`.
    manejadores: Vec<Manejador>,
    relanzar: Option<String>,
    // Depurador: mostrar cada instrucción (`trazar`) o
    // pararse en cada una y preguntar (`depurar`).
    pub trazar: bool,
    pub depurar: bool,
    // Cuántos pasos se han corrido y cuándo se empezó
    // (para el resumen del final).
    pasos: usize,
    inicio: Option<std::time::Instant>,
}

impl MaquinaVirtual {
    // Crea la máquina con un programa ya compilado.
    // El marco principal es el pedazo principal.
    pub fn nueva(programa: Programa) -> Self {
        let mut vm = MaquinaVirtual {
            funciones: programa
                .funciones
                .into_iter()
                .map(|(k, v)| (k, Rc::new(v)))
                .collect(),
            registradas: HashMap::new(),
            pila: Vec::new(),
            globales: Vec::new(),
            marcos: Vec::new(),
            instrucciones: 0,
            semilla: semilla_inicial(),
            prestamos_pendientes: Vec::new(),
            prestados_pendientes: Vec::new(),
            manejadores: Vec::new(),
            relanzar: None,
            trazar: false,
            depurar: false,
            pasos: 0,
            inicio: None,
        };
        let slots = vec![Celda::nueva(Valor::Nulo, false); programa.principal.max_slots];
        vm.marcos.push(Marco {
            pedazo: Rc::new(programa.principal),
            ip: 0,
            base_pila: 0,
            slots,
            devoluciones: Vec::new(),
            nombre: "principal".to_string(),
        });
        vm
    }

    // Una máquina vacía, para el modo interactivo: sin programa
    // principal. Cada línea se corre con `ejecutar_pedazo`.
    pub fn nueva_vacia() -> Self {
        MaquinaVirtual {
            funciones: HashMap::new(),
            registradas: HashMap::new(),
            pila: Vec::new(),
            globales: Vec::new(),
            marcos: Vec::new(),
            instrucciones: 0,
            semilla: semilla_inicial(),
            prestamos_pendientes: Vec::new(),
            prestados_pendientes: Vec::new(),
            manejadores: Vec::new(),
            relanzar: None,
            trazar: false,
            depurar: false,
            pasos: 0,
            inicio: None,
        }
    }

    // Registra funciones nuevas (las que se definieron en una
    // línea del modo interactivo).
    pub fn agregar_funciones(&mut self, funciones: &HashMap<String, Pedazo>) {
        for (nombre, pedazo) in funciones {
            self.funciones.insert(nombre.clone(), Rc::new(pedazo.to_owned()));
        }
    }

    // Corre un pedazo suelto (una línea del modo interactivo)
    // con las variables globales que ya existen. Si da error,
    // se limpian los marcos para que la siguiente línea
    // empiece limpia.
    pub fn ejecutar_pedazo(&mut self, pedazo: Pedazo) -> Result<(), String> {
        self.instrucciones = 0;
        let slots = vec![Celda::nueva(Valor::Nulo, false); pedazo.max_slots];
        self.marcos.push(Marco {
            pedazo: Rc::new(pedazo),
            ip: 0,
            base_pila: 0,
            slots,
            devoluciones: Vec::new(),
            nombre: "repl".to_string(),
        });
        let resultado = self.correr();
        if resultado.is_err() {
            self.marcos.clear();
        }
        self.manejadores.clear();
        self.relanzar = None;
        // Lo que quedó en la pila era el valor de vuelta del
        // fragmento: se tira, para que no se acumule.
        self.pila.clear();
        resultado
    }

    // El resumen del depurador al final: cuántos pasos y
    // cuánto tiempo tardó.
    pub fn resumen_depurador(&self) {
        if let Some(inicio) = self.inicio {
            println!(
                "--- depurador: {} pasos en {:.2} segundos ---",
                self.pasos,
                inicio.elapsed().as_secs_f64()
            );
        }
    }

    // Corre el programa hasta que se acabe o dé un error.
    pub fn correr(&mut self) -> Result<(), String> {
        loop {
            self.instrucciones += 1;
            // Los límites se revisan cada 1024 instrucciones,
            // no en cada una: el bucle caliente corre más rápido
            // y un bucle sin fin se para igual (1024 después).
            if self.instrucciones & 1023 == 0 {
                if self.instrucciones > MAX_INSTRUCCIONES {
                    return Err(
                        "el programa se pasó de 100,000,000 de instrucciones, ¿tienes un bucle sin fin?"
                            .to_string(),
                    );
                }
                if self.pila.len() > MAX_PILA {
                    return Err("la pila se llenó, ¿tienes demasiados valores a la vez?".to_string());
                }
            }
            if self.marcos.is_empty() {
                return Ok(());
            }
            let (instr, ip) = {
                let marco = self
                    .marcos
                    .last_mut()
                    .ok_or_else(|| "no hay ninguna función corriendo".to_string())?;
                if marco.ip >= marco.pedazo.codigo.len() {
                    self.marcos.pop();
                    // Al cerrar un marco, se van también los
                    // manejadores que se abrieron dentro de él.
                    self.podar_manejadores();
                    continue;
                }
                let instr = marco.pedazo.codigo[marco.ip];
                let ip = marco.ip;
                marco.ip += 1;
                (instr, ip)
            };
            // Depurador: contar el paso y, si toca, mostrarlo
            // o pararse a preguntar qué hacer.
            if self.trazar || self.depurar {
                if self.inicio.is_none() {
                    self.inicio = Some(std::time::Instant::now());
                }
                self.pasos += 1;
                if self.trazar {
                    eprintln!("{:04} {:?}", ip, instr);
                }
                if self.depurar {
                    self.paso_depurador(ip, &instr)?;
                }
            }
            // Si la instrucción falla, un `intentar` abierto
            // puede atrapar el error: se salta a su `si_falla`
            // (o a su `al_final`) en vez de parar el programa.
            // Si no hay ningún `intentar`, el error sube y el
            // programa se detiene, como siempre.
            if let Err(e) = self.ejecutar(instr) {
                if !self.atender_error(e.clone()) {
                    return Err(e);
                }
            }
        }
    }

    // Atiende un error con el `intentar` más adentro que haya
    // abierto: devuelve verdadero si lo atrapó (y se sigue
    // corriendo) o falso si no había ninguno.
    fn atender_error(&mut self, error: String) -> bool {
        while let Some(manejador) = self.manejadores.pop() {
            // Si el manejador no tiene a dónde saltar, no sirve:
            // se sigue buscando uno más afuera (no debería pasar,
            // el compilador siempre pone al menos un destino).
            if manejador.salto_falla.is_none() && manejador.salto_final.is_none() {
                continue;
            }
            // Se vuelve a donde estaba la pila y los marcos
            // cuando se abrió el intentar (se tiran los valores
            // a medias y las funciones que se llamaron dentro).
            self.pila.truncate(manejador.pila);
            self.marcos.truncate(manejador.marcos);
            if let Some(destino) = manejador.salto_falla {
                // Con `si_falla`: el error va en la pila para
                // que el bloque lo guarde en su variable.
                self.pila.push(Valor::Texto(error));
                if let Some(marco) = self.marcos.last_mut() {
                    marco.ip = destino;
                }
                return true;
            }
            // Sin `si_falla`: corre el `al_final` y después se
            // vuelve a soltar el mismo error.
            self.relanzar = Some(error);
            if let Some(destino) = manejador.salto_final {
                if let Some(marco) = self.marcos.last_mut() {
                    marco.ip = destino;
                }
            }
            return true;
        }
        false
    }

    // Quita los manejadores de los marcos que ya se cerraron.
    fn podar_manejadores(&mut self) {
        let marcos = self.marcos.len();
        self.manejadores.retain(|m| m.marcos <= marcos);
    }

    // Un paso del depurador: muestra la instrucción y pregunta
    // qué hacer. Enter = siguiente paso, v = ver variables,
    // p nombre = ver una variable, c = seguir hasta el final,
    // s = salir (detiene el programa).
    fn paso_depurador(&mut self, ip: usize, instr: &Instruccion) -> Result<(), String> {
        loop {
            eprintln!("{:04} {:?}", ip, instr);
            eprint!("depura (Enter=paso, v=variables, p nombre=ver, c=seguir, s=salir)> ");
            let _ = io::stderr().flush();
            let mut linea = String::new();
            match io::stdin().read_line(&mut linea) {
                Ok(0) => return Err("depuración detenida por ti".to_string()),
                Ok(_) => {}
                Err(_) => return Err("depuración detenida por ti".to_string()),
            }
            match linea.trim() {
                "" => return Ok(()),
                "c" => {
                    self.depurar = false;
                    return Ok(());
                }
                "s" => return Err("depuración detenida por ti".to_string()),
                "v" => self.mostrar_variables(),
                orden => {
                    if let Some(nombre) = orden.strip_prefix("p ") {
                        let nombre = nombre.trim();
                        match self.buscar_variable(nombre) {
                            Some(valor) => println!("{} = {}", nombre, valor),
                            None => {
                                println!("no veo una variable llamada '{}' aquí", nombre)
                            }
                        }
                    } else {
                        eprintln!("no conozco esa orden. Enter=paso, v=variables, p nombre=ver, c=seguir, s=salir");
                    }
                }
            }
        }
    }

    // Muestra las variables que se ven ahora: las locales del
    // marco actual y las globales del programa.
    fn mostrar_variables(&self) {
        let mut vistos_locales: Vec<usize> = Vec::new();
        let mut vistos_globales: Vec<usize> = Vec::new();
        if let Some(marco) = self.marcos.last() {
            for instr in &marco.pedazo.codigo {
                if let Some((slot, global, nombre)) = dato_de(instr, &marco.pedazo) {
                    if global {
                        if vistos_globales.contains(&slot) {
                            continue;
                        }
                        vistos_globales.push(slot);
                        if let Some(celda) = self.globales.get(slot) {
                            println!("{} = {}", nombre, celda.valor.a_texto());
                        }
                    } else {
                        if vistos_locales.contains(&slot) {
                            continue;
                        }
                        vistos_locales.push(slot);
                        if let Some(celda) = marco.slots.get(slot) {
                            println!("{} = {}", nombre, celda.valor.a_texto());
                        }
                    }
                }
            }
        }
    }

    // El valor de una variable por su nombre: primero las
    // locales del marco actual, luego las globales.
    fn buscar_variable(&self, nombre: &str) -> Option<String> {
        let marco = self.marcos.last()?;
        for instr in &marco.pedazo.codigo {
            if let Some((slot, global, n)) = dato_de(instr, &marco.pedazo) {
                if n == nombre && !global {
                    return marco.slots.get(slot).map(|c| c.valor.a_texto());
                }
            }
        }
        let principal = self.marcos.first()?;
        for instr in &principal.pedazo.codigo {
            if let Some((slot, global, n)) = dato_de(instr, &principal.pedazo) {
                if n == nombre && global {
                    return self.globales.get(slot).map(|c| c.valor.a_texto());
                }
            }
        }
        None
    }

    // La constante que hay detrás de un numerito, del pedazo
    // que se está ejecutando ahora mismo.
    fn constante_actual(&self, indice: usize) -> Option<crate::motor::valor::Valor> {
        let marco = self.marcos.last()?;
        marco.pedazo.constantes.get(indice).cloned()
    }

    // El nombre que hay detrás de un numerito, del pedazo
    // que se está ejecutando ahora mismo.
    fn nombre_actual(&self, indice: usize) -> Result<String, String> {
        let marco = self
            .marcos
            .last()
            .ok_or_else(|| "no hay ninguna función corriendo".to_string())?;
        Ok(marco.pedazo.nombre(indice))
    }

    // Saca el valor de arriba de la pila (o da un error si
    // la pila está vacía, que no debería pasar nunca).
    fn sacar(&mut self) -> Result<Valor, String> {
        self.pila
            .pop()
            .ok_or_else(|| "la pila se quedó vacía, algo salió mal".to_string())
    }

    // Crea una variable en su slot: en el array global si es
    // global, o en los slots del marco actual si es local.
    fn definir(&mut self, slot: usize, global: bool, valor: Valor, mutable: bool) {
        self.definir_celda(slot, global, Celda::nueva(valor, mutable));
    }

    // Crea una constante en su slot: nunca se puede cambiar
    // ni se mueve.
    fn definir_constante(&mut self, slot: usize, global: bool, valor: Valor) {
        self.definir_celda(slot, global, Celda::constante(valor));
    }

    // Guarda un cajón (Celda) en su slot, creando los huecos
    // que falten en el array global.
    fn definir_celda(&mut self, slot: usize, global: bool, celda: Celda) {
        if global {
            while self.globales.len() <= slot {
                self.globales.push(Celda::nueva(Valor::Nulo, false));
            }
            self.globales[slot] = celda;
        } else if let Some(marco) = self.marcos.last_mut() {
            if slot < marco.slots.len() {
                marco.slots[slot] = celda;
            }
        }
    }

    // Cambia el valor de una variable que ya existe.
    // Asignarle un valor nuevo la revive: ya no está movida.
    // Una constante no se puede cambiar nunca.
    fn asignar(
        &mut self,
        slot: usize,
        global: bool,
        nombre_idx: usize,
        valor: Valor,
    ) -> Result<(), String> {
        // Fase 1: mirar y cambiar. El nombre no se busca aquí:
        // en el camino normal no se clona ningún texto.
        enum Problema {
            Constante,
            Fija,
            NoExiste,
        }
        let problema = {
            let entrada = if global {
                self.globales.get_mut(slot)
            } else {
                self.marcos
                    .last_mut()
                    .and_then(|m| m.slots.get_mut(slot))
            };
            match entrada {
                Some(celda) => {
                    if celda.constante {
                        Problema::Constante
                    } else if celda.mutable {
                        celda.valor = valor;
                        celda.movido = None;
                        return Ok(());
                    } else {
                        Problema::Fija
                    }
                }
                None => Problema::NoExiste,
            }
        };
        // Fase 2: solo si hubo un problema se busca el nombre.
        let nombre = self.nombre_actual(nombre_idx)?;
        match problema {
            Problema::Constante => Err(format!(
                "la constante '{}' no se puede cambiar, nunca. Si quieres una variable que cambie, créala con 'sea mut'",
                nombre
            )),
            Problema::Fija => Err(format!(
                "la variable '{}' es fija, no se puede cambiar. Si quieres cambiarla, créala con 'sea mut'",
                nombre
            )),
            Problema::NoExiste => Err(format!("no conozco la variable '{}'", nombre)),
        }
    }

    // Cambia un lugar dentro de una lista o un mapa que ya
    // existe: lista[0] = 5;  mapa[21] = "FTP";
    // Como `asignar`, una constante o una variable fija no se
    // pueden cambiar, y cambiar un lugar revive la variable
    // si se había movido (igual que asignarle un valor nuevo).
    fn asignar_indice(
        &mut self,
        slot: usize,
        global: bool,
        nombre_idx: usize,
        indice: Valor,
        valor: Valor,
    ) -> Result<(), String> {
        // Fase 1: mirar y cambiar. El nombre no se busca aquí:
        // en el camino normal no se clona ningún texto.
        enum Problema {
            Constante,
            Fija,
            NoExiste,
            Movida(String),
        }
        let problema = {
            let entrada = if global {
                self.globales.get_mut(slot)
            } else {
                self.marcos
                    .last_mut()
                    .and_then(|m| m.slots.get_mut(slot))
            };
            match entrada {
                Some(celda) => {
                    if celda.constante {
                        Problema::Constante
                    } else if !celda.mutable {
                        Problema::Fija
                    } else if let Some(destino) = &celda.movido {
                        Problema::Movida(destino.clone())
                    } else {
                        // Cambiar el lugar dentro del valor.
                        return cambiar_lugar(&mut celda.valor, indice, valor);
                    }
                }
                None => Problema::NoExiste,
            }
        };
        // Fase 2: solo si hubo un problema se busca el nombre.
        let nombre = self.nombre_actual(nombre_idx)?;
        match problema {
            Problema::Constante => Err(format!(
                "la constante '{}' no se puede cambiar, nunca. Si quieres una variable que cambie, créala con 'sea mut'",
                nombre
            )),
            Problema::Fija => Err(format!(
                "la variable '{}' es fija, no se puede cambiar. Si quieres cambiarla, créala con 'sea mut'",
                nombre
            )),
            Problema::NoExiste => Err(format!("no conozco la variable '{}'", nombre)),
            Problema::Movida(destino) => Err(format!(
                "la variable '{}' se movió {}, ya no puedes usarla. Para quedarte con una copia, usa copiar({})",
                nombre, destino, nombre
            )),
        }
    }

    // Busca el valor de una variable. Si se movió, te dice
    // a dónde se fue.
    fn obtener(&self, slot: usize, global: bool, nombre_idx: usize) -> Result<Valor, String> {
        let entrada = if global {
            self.globales.get(slot)
        } else {
            self.marcos
                .last()
                .and_then(|m| m.slots.get(slot))
        };
        match entrada {
            Some(celda) => {
                if let Some(destino) = &celda.movido {
                    let nombre = self.nombre_actual(nombre_idx)?;
                    return Err(format!(
                        "la variable '{}' se movió {}, ya no puedes usarla. Para quedarte con una copia, usa copiar({})",
                        nombre, destino, nombre
                    ));
                }
                Ok(celda.valor.clone())
            }
            None => {
                let nombre = self.nombre_actual(nombre_idx)?;
                Err(format!("no conozco la variable '{}'", nombre))
            }
        }
    }

    // Lee una variable sin mirar si se movió: (valor, mutable).
    // La usa PrestarMut para revisar el préstamo.
    fn leer(&self, slot: usize, global: bool) -> Option<(Valor, bool)> {
        let entrada = if global {
            self.globales.get(slot)
        } else {
            self.marcos.last().and_then(|m| m.slots.get(slot))
        };
        entrada.map(|celda| (celda.valor.clone(), celda.mutable))
    }

    // Mueve una variable a otro dueño: si su valor es texto o
    // lista, la marca como movida y anota a dónde se fue (los
    // números y los lógicos son fotocopias, así que no se
    // mueven nunca). Las constantes nunca se mueven. Y algo
    // prestado no se puede mover mientras dure el préstamo.
    fn mover(
        &mut self,
        slot: usize,
        global: bool,
        nombre_idx: usize,
        destino_idx: usize,
    ) -> Result<(), String> {
        // Fase 1: mirar sin cambiar (préstamo compartido).
        let (prestado, es_texto_o_lista, constante) = {
            let entrada = if global {
                self.globales.get(slot)
            } else {
                self.marcos.last().and_then(|m| m.slots.get(slot))
            };
            match entrada {
                Some(celda) => (
                    celda.prestado,
                    matches!(celda.valor, Valor::Texto(_) | Valor::Lista(_) | Valor::Mapa(_)),
                    celda.constante,
                ),
                None => return Ok(()),
            }
        };
        if constante {
            return Ok(());
        }
        if prestado {
            let nombre = self.nombre_actual(nombre_idx)?;
            return Err(format!(
                "no se puede mover '{}' porque está prestado a una función (se lo pasaste con & o &mut) y el préstamo sigue activo",
                nombre
            ));
        }
        // Fase 2: marcar como movida (préstamo mutable).
        if es_texto_o_lista {
            let destino = self.nombre_actual(destino_idx)?;
            let entrada = if global {
                self.globales.get_mut(slot)
            } else {
                self.marcos
                    .last_mut()
                    .and_then(|m| m.slots.get_mut(slot))
            };
            if let Some(celda) = entrada {
                celda.movido = Some(destino);
            }
        }
        Ok(())
    }

    // Input (`leer nombre;`): lee lo que escribe el usuario y
    // lo guarda en la variable. La variable debe ser `sea mut`.
    // Si ya era un número, convierte el texto a número; si era
    // un lógico, convierte "verdadero" o "falso".
    fn leer_entrada(&mut self, slot: usize, global: bool, nombre_idx: usize) -> Result<(), String> {
        let nombre = self.nombre_actual(nombre_idx)?;
        // `obtener` revisa que la variable no se haya movido.
        let actual = self.obtener(slot, global, nombre_idx)?;
        let mutable = self
            .leer(slot, global)
            .map(|(_, m)| m)
            .ok_or_else(|| format!("no conozco la variable '{}'", nombre))?;
        if !mutable {
            return Err(format!(
                "no se puede leer en '{}' porque es fija. Si quieres que el usuario la escriba, créala con 'sea mut'",
                nombre
            ));
        }
        let mut linea = String::new();
        io::stdin()
            .read_line(&mut linea)
            .map_err(|_| "no pude leer lo que escribiste".to_string())?;
        let texto = linea.trim_end_matches(['\n', '\r']).to_string();
        let valor = match actual {
            Valor::Numero(_) => match texto.trim().parse::<f64>() {
                Ok(n) => Valor::Numero(n),
                Err(_) => return Err(format!("eso no era un número: '{}'", texto)),
            },
            Valor::Logico(_) => match texto.trim() {
                "verdadero" => Valor::Logico(true),
                "falso" => Valor::Logico(false),
                _ => {
                    return Err(format!(
                        "eso no era verdadero ni falso: '{}'",
                        texto
                    ))
                }
            },
            // Una lista: separa lo escrito por comas (igual que
            // en el intérprete: números si la lista era de
            // números o estaba vacía y todo son números).
            Valor::Lista(items) => {
                let partes: Vec<&str> = texto.split(',').map(|p| p.trim()).collect();
                let son_numeros = match items.first() {
                    Some(Valor::Numero(_)) => true,
                    Some(_) => false,
                    None => partes.iter().all(|p| p.parse::<f64>().is_ok()),
                };
                if son_numeros {
                    let mut valores = Vec::new();
                    for p in partes {
                        match p.parse::<f64>() {
                            Ok(n) => valores.push(Valor::Numero(n)),
                            Err(_) => {
                                return Err(format!("eso no era un número: '{}'", p))
                            }
                        }
                    }
                    Valor::lista(valores)
                } else {
                    Valor::lista(
                        partes
                            .iter()
                            .map(|p| Valor::Texto(p.to_string()))
                            .collect(),
                    )
                }
            }
            _ => Valor::Texto(texto),
        };
        self.asignar(slot, global, nombre_idx, valor)
    }

    // Incrementa o decrementa un número en 1 (puntos++ o --).
    fn incrementar(
        &mut self,
        slot: usize,
        global: bool,
        nombre_idx: usize,
        subir: bool,
    ) -> Result<(), String> {
        let actual = self.obtener(slot, global, nombre_idx)?;
        let numero = match actual {
            Valor::Numero(n) => n,
            otro => {
                return Err(format!(
                    "solo se puede usar '{}' con un numero, no con {}",
                    if subir { "++" } else { "--" },
                    otro.tipo()
                ))
            }
        };
        let nuevo = if subir { numero + 1.0 } else { numero - 1.0 };
        self.asignar(slot, global, nombre_idx, Valor::Numero(nuevo))
    }

    // Revisa que una función reciba la cantidad de datos que
    // pide, con el mismo mensaje que da el intérprete.
    fn cantidad(&self, nombre: &str, argumentos: &[Valor], esperado: usize) -> Result<(), String> {
        if argumentos.len() != esperado {
            return Err(format!(
                "'{}' necesita {} {}, pero le diste {}",
                nombre,
                esperado,
                if esperado == 1 { "dato" } else { "datos" },
                argumentos.len()
            ));
        }
        Ok(())
    }

    // Llama a una función con los datos que hay en la pila.
    // Si es una de las que trae Eclipse, la hace aquí mismo.
    // Si es una función tuya, crea un marco nuevo con sus
    // slots y salta a su pedazo.
    //
    // Los mensajes de error son los mismos que da el
    // intérprete, para que un programa se comporte igual
    // en los tres motores.
    fn llamar(&mut self, nombre: &str, cantidad_args: usize) -> Result<(), String> {
        let mut argumentos = Vec::new();
        for _ in 0..cantidad_args {
            argumentos.push(self.sacar()?);
        }
        argumentos.reverse();
        match nombre {
            "imprimir" => {
                let textos: Vec<String> = argumentos.iter().map(|v| v.a_texto()).collect();
                println!("{}", textos.join(" "));
                self.pila.push(Valor::Nulo);
                Ok(())
            }
            "leer" => {
                if !argumentos.is_empty() {
                    return Err("leer no necesita datos entre paréntesis".to_string());
                }
                let mut linea = String::new();
                io::stdin()
                    .read_line(&mut linea)
                    .map_err(|_| "no pude leer lo que escribiste".to_string())?;
                let texto = linea.trim_end_matches(['\n', '\r']).to_string();
                self.pila.push(Valor::Texto(texto));
                Ok(())
            }
            "leer_numero" => {
                if !argumentos.is_empty() {
                    return Err("leer_numero no necesita datos entre paréntesis".to_string());
                }
                let mut linea = String::new();
                io::stdin()
                    .read_line(&mut linea)
                    .map_err(|_| "no pude leer lo que escribiste".to_string())?;
                let texto = linea.trim_end_matches(['\n', '\r']).trim();
                match texto.parse::<f64>() {
                    Ok(n) => {
                        self.pila.push(Valor::Numero(n));
                        Ok(())
                    }
                    Err(_) => Err(format!("eso no era un número: '{}'", texto)),
                }
            }
            "a_numero" => {
                self.cantidad(nombre, &argumentos, 1)?;
                match &argumentos[0] {
                    Valor::Texto(s) => match s.trim().parse::<f64>() {
                        Ok(n) => {
                            self.pila.push(Valor::Numero(n));
                            Ok(())
                        }
                        Err(_) => Err(format!("eso no era un número: '{}'", s)),
                    },
                    Valor::Numero(n) => {
                        self.pila.push(Valor::Numero(*n));
                        Ok(())
                    }
                    otro => Err(format!("a_numero necesita texto, no {}", otro.tipo())),
                }
            }
            "a_texto" => {
                self.cantidad(nombre, &argumentos, 1)?;
                self.pila.push(Valor::Texto(argumentos[0].a_texto()));
                Ok(())
            }
            "leer_archivo" => {
                self.cantidad(nombre, &argumentos, 1)?;
                let ruta = match &argumentos[0] {
                    Valor::Texto(s) => s.clone(),
                    _ => {
                        return Err(
                            "leer_archivo necesita el nombre del archivo entre comillas"
                                .to_string(),
                        )
                    }
                };
                match std::fs::read_to_string(&ruta) {
                    Ok(contenido) => {
                        self.pila.push(Valor::Texto(contenido));
                        Ok(())
                    }
                    Err(_) => Err(format!("no encontré el archivo '{}'", ruta)),
                }
            }
            "escribir_archivo" => {
                self.cantidad(nombre, &argumentos, 2)?;
                let ruta = match &argumentos[0] {
                    Valor::Texto(s) => s.clone(),
                    _ => {
                        return Err(
                            "escribir_archivo necesita el nombre del archivo entre comillas"
                                .to_string(),
                        )
                    }
                };
                let contenido = argumentos[1].a_texto();
                match std::fs::write(&ruta, contenido) {
                    Ok(()) => {
                        self.pila.push(Valor::Nulo);
                        Ok(())
                    }
                    Err(_) => Err(format!("no pude escribir el archivo '{}'", ruta)),
                }
            }
            "largo" => {
                self.cantidad(nombre, &argumentos, 1)?;
                match &argumentos[0] {
                    Valor::Texto(s) => {
                        self.pila.push(Valor::Numero(s.chars().count() as f64));
                        Ok(())
                    }
                    Valor::Lista(elementos) => {
                        self.pila.push(Valor::Numero(elementos.len() as f64));
                        Ok(())
                    }
                    Valor::Mapa(pares) => {
                        self.pila.push(Valor::Numero(pares.len() as f64));
                        Ok(())
                    }
                    otro => Err(format!(
                        "largo necesita texto, lista o mapa, no {}",
                        otro.tipo()
                    )),
                }
            }
            "tipo_de" => {
                self.cantidad(nombre, &argumentos, 1)?;
                self.pila.push(Valor::Texto(argumentos[0].tipo().to_string()));
                Ok(())
            }
            "aleatorio" => {
                if !argumentos.is_empty() {
                    return Err("aleatorio no necesita datos entre paréntesis".to_string());
                }
                let n = self.siguiente_aleatorio() as f64;
                self.pila.push(Valor::Numero(n));
                Ok(())
            }
            "copiar" => {
                self.cantidad(nombre, &argumentos, 1)?;
                self.pila.push(argumentos[0].clone());
                Ok(())
            }
            "min" => {
                self.cantidad(nombre, &argumentos, 2)?;
                match (&argumentos[0], &argumentos[1]) {
                    (Valor::Numero(a), Valor::Numero(b)) => {
                        self.pila.push(Valor::Numero(if a < b { *a } else { *b }));
                        Ok(())
                    }
                    (a, b) => Err(format!(
                        "min necesita dos numeros, no {} y {}",
                        a.tipo(),
                        b.tipo()
                    )),
                }
            }
            "max" => {
                self.cantidad(nombre, &argumentos, 2)?;
                match (&argumentos[0], &argumentos[1]) {
                    (Valor::Numero(a), Valor::Numero(b)) => {
                        self.pila.push(Valor::Numero(if a > b { *a } else { *b }));
                        Ok(())
                    }
                    (a, b) => Err(format!(
                        "max necesita dos numeros, no {} y {}",
                        a.tipo(),
                        b.tipo()
                    )),
                }
            }
            // ahora() -> los segundos desde 1970 (el reloj).
            "ahora" => {
                if !argumentos.is_empty() {
                    return Err("ahora no necesita datos entre paréntesis".to_string());
                }
                let segundos = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs() as f64)
                    .unwrap_or(0.0);
                self.pila.push(Valor::Numero(segundos));
                Ok(())
            }
            // caracter(65) -> "A" (el carácter de ese número).
            "caracter" => {
                self.cantidad(nombre, &argumentos, 1)?;
                match &argumentos[0] {
                    Valor::Numero(n) => match char::from_u32(*n as u32) {
                        Some(c) => {
                            self.pila.push(Valor::Texto(c.to_string()));
                            Ok(())
                        }
                        None => Err(format!("ese número no es un carácter válido: {}", n)),
                    },
                    otro => Err(format!("caracter necesita un numero, no {}", otro.tipo())),
                }
            }
            // sistema() -> "Windows", "Linux" o "macOS".
            "sistema" => {
                if !argumentos.is_empty() {
                    return Err("sistema no necesita datos entre paréntesis".to_string());
                }
                self.pila.push(Valor::Texto(crate::motor::sistema::sistema()));
                Ok(())
            }
            // usuario() -> tu nombre de usuario del sistema.
            "usuario" => {
                if !argumentos.is_empty() {
                    return Err("usuario no necesita datos entre paréntesis".to_string());
                }
                self.pila.push(Valor::Texto(crate::motor::sistema::usuario()));
                Ok(())
            }
            // carpeta() -> la carpeta donde estás parado.
            "carpeta" => {
                if !argumentos.is_empty() {
                    return Err("carpeta no necesita datos entre paréntesis".to_string());
                }
                self.pila.push(Valor::Texto(crate::motor::sistema::carpeta()?));
                Ok(())
            }
            // variable_entorno("PATH") -> el valor de esa variable.
            "variable_entorno" => {
                self.cantidad(nombre, &argumentos, 1)?;
                match &argumentos[0] {
                    Valor::Texto(s) => {
                        let v = crate::motor::sistema::variable_entorno(s)?;
                        self.pila.push(Valor::Texto(v));
                        Ok(())
                    }
                    otro => Err(format!("variable_entorno necesita un texto, no {}", otro.tipo())),
                }
            }
            // comando("dir") -> lo que imprimió ese programa.
            "comando" => {
                self.cantidad(nombre, &argumentos, 1)?;
                match &argumentos[0] {
                    Valor::Texto(s) => {
                        let v = crate::motor::sistema::comando(s)?;
                        self.pila.push(Valor::Texto(v));
                        Ok(())
                    }
                    otro => Err(format!("comando necesita un texto, no {}", otro.tipo())),
                }
            }
            // argumentos() -> la lista de datos que le diste al programa.
            "argumentos" => {
                if !argumentos.is_empty() {
                    return Err("argumentos no necesita datos entre paréntesis".to_string());
                }
                self.pila.push(Valor::lista(
                    crate::motor::sistema::argumentos()
                        .into_iter()
                        .map(Valor::Texto)
                        .collect(),
                ));
                Ok(())
            }
            // procesadores() -> cuántos núcleos tiene la computadora.
            "procesadores" => {
                if !argumentos.is_empty() {
                    return Err("procesadores no necesita datos entre paréntesis".to_string());
                }
                self.pila.push(Valor::Numero(crate::motor::sistema::procesadores()));
                Ok(())
            }
            // memoria() -> cuánta memoria total tiene, en MB.
            "memoria" => {
                if !argumentos.is_empty() {
                    return Err("memoria no necesita datos entre paréntesis".to_string());
                }
                self.pila.push(Valor::Numero(crate::motor::sistema::memoria_mb()?));
                Ok(())
            }
            // redondear(3.7) -> 4 ; absoluto(-5) -> 5
            "redondear" => {
                self.cantidad(nombre, &argumentos, 1)?;
                match &argumentos[0] {
                    Valor::Numero(n) => {
                        self.pila.push(Valor::Numero(n.round()));
                        Ok(())
                    }
                    otro => Err(format!(
                        "redondear necesita un numero, no {}",
                        otro.tipo()
                    )),
                }
            }
            "absoluto" => {
                self.cantidad(nombre, &argumentos, 1)?;
                match &argumentos[0] {
                    Valor::Numero(n) => {
                        self.pila.push(Valor::Numero(n.abs()));
                        Ok(())
                    }
                    otro => Err(format!(
                        "absoluto necesita un numero, no {}",
                        otro.tipo()
                    )),
                }
            }
            // mayusculas("hola") -> "HOLA" ; minusculas("HOLA") -> "hola"
            "mayusculas" => {
                self.cantidad(nombre, &argumentos, 1)?;
                match &argumentos[0] {
                    Valor::Texto(s) => {
                        self.pila.push(Valor::Texto(s.to_uppercase()));
                        Ok(())
                    }
                    otro => Err(format!(
                        "mayusculas necesita un texto, no {}",
                        otro.tipo()
                    )),
                }
            }
            "minusculas" => {
                self.cantidad(nombre, &argumentos, 1)?;
                match &argumentos[0] {
                    Valor::Texto(s) => {
                        self.pila.push(Valor::Texto(s.to_lowercase()));
                        Ok(())
                    }
                    otro => Err(format!(
                        "minusculas necesita un texto, no {}",
                        otro.tipo()
                    )),
                }
            }
            // contiene("hola kevin", "kevin") -> verdadero
            "contiene" => {
                self.cantidad(nombre, &argumentos, 2)?;
                match (&argumentos[0], &argumentos[1]) {
                    (Valor::Texto(t), Valor::Texto(p)) => {
                        self.pila.push(Valor::Logico(t.contains(p.as_str())));
                        Ok(())
                    }
                    (a, b) => Err(format!(
                        "contiene necesita dos textos, no {} y {}",
                        a.tipo(),
                        b.tipo()
                    )),
                }
            }
            // ordenar([3, 1, 2]) es [1, 2, 3]. Con textos los
            // ordena por el abecedario. La lista vieja no cambia.
            "ordenar" => {
                self.cantidad(nombre, &argumentos, 1)?;
                match &argumentos[0] {
                    Valor::Lista(elementos) => {
                        let mut copia = (**elementos).clone();
                        let todos_numeros =
                            copia.iter().all(|v| matches!(v, Valor::Numero(_)));
                        let todos_textos =
                            copia.iter().all(|v| matches!(v, Valor::Texto(_)));
                        if todos_numeros {
                            copia.sort_by(|a, b| match (a, b) {
                                (Valor::Numero(x), Valor::Numero(y)) => {
                                    x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal)
                                }
                                _ => std::cmp::Ordering::Equal,
                            });
                        } else if todos_textos {
                            copia.sort_by(|a, b| match (a, b) {
                                (Valor::Texto(x), Valor::Texto(y)) => x.cmp(y),
                                _ => std::cmp::Ordering::Equal,
                            });
                        } else {
                            return Err(
                                "ordenar necesita una lista de solo números o solo textos"
                                    .to_string(),
                            );
                        }
                        self.pila.push(Valor::lista(copia));
                        Ok(())
                    }
                    otro => Err(format!(
                        "ordenar necesita una lista, no {}",
                        otro.tipo()
                    )),
                }
            }
            // invertir([1, 2, 3]) es [3, 2, 1]; invertir("hola")
            // es "aloh". La lista o el texto viejos no cambian.
            "invertir" => {
                self.cantidad(nombre, &argumentos, 1)?;
                match &argumentos[0] {
                    Valor::Lista(elementos) => {
                        let mut copia = (**elementos).clone();
                        copia.reverse();
                        self.pila.push(Valor::lista(copia));
                        Ok(())
                    }
                    Valor::Texto(s) => {
                        self.pila.push(Valor::Texto(s.chars().rev().collect()));
                        Ok(())
                    }
                    otro => Err(format!(
                        "invertir necesita una lista o un texto, no {}",
                        otro.tipo()
                    )),
                }
            }
            // unir(["a", "b"], ", ") es "a, b".
            "unir" => {
                self.cantidad(nombre, &argumentos, 2)?;
                match (&argumentos[0], &argumentos[1]) {
                    (Valor::Lista(elementos), Valor::Texto(s)) => {
                        let textos: Vec<String> =
                            elementos.iter().map(|v| v.a_texto()).collect();
                        self.pila.push(Valor::Texto(textos.join(s.as_str())));
                        Ok(())
                    }
                    (l, s) => Err(format!(
                        "unir necesita una lista y un texto, no {} y {}",
                        l.tipo(),
                        s.tipo()
                    )),
                }
            }
            // separar("a,b,c", ",") es ["a", "b", "c"].
            "separar" => {
                self.cantidad(nombre, &argumentos, 2)?;
                match (&argumentos[0], &argumentos[1]) {
                    (Valor::Texto(t), Valor::Texto(s)) => {
                        if s.is_empty() {
                            return Err(
                                "el separador no puede estar vacío".to_string()
                            );
                        }
                        self.pila.push(Valor::lista(
                            t.split(s.as_str())
                                .map(|p| Valor::Texto(p.to_string()))
                                .collect(),
                        ));
                        Ok(())
                    }
                    (a, b) => Err(format!(
                        "separar necesita dos textos, no {} y {}",
                        a.tipo(),
                        b.tipo()
                    )),
                }
            }
            // raiz(16) es 4. Un número negativo no tiene raíz.
            "raiz" => {
                self.cantidad(nombre, &argumentos, 1)?;
                match &argumentos[0] {
                    Valor::Numero(n) => {
                        if *n < 0.0 {
                            return Err(format!(
                                "no se puede calcular la raíz de {} porque es negativo",
                                n
                            ));
                        }
                        self.pila.push(Valor::Numero(n.sqrt()));
                        Ok(())
                    }
                    otro => Err(format!(
                        "raiz necesita un numero, no {}",
                        otro.tipo()
                    )),
                }
            }
            "agregar" => {
                self.cantidad(nombre, &argumentos, 2)?;
                // La lista se mueve (no se clona): el intérprete
                // hace lo mismo, así que agregar es O(1) y no O(n).
                let mut it = argumentos.into_iter();
                let lista = it.next().unwrap();
                let valor = it.next().unwrap();
                match lista {
                    Valor::Lista(mut elementos) => {
                        Arc::make_mut(&mut elementos).push(valor);
                        self.pila.push(Valor::Lista(elementos));
                        Ok(())
                    }
                    otro => Err(format!("agregar necesita una lista, no {}", otro.tipo())),
                }
            }
            "elemento" => {
                self.cantidad(nombre, &argumentos, 2)?;
                match &argumentos[0] {
                    Valor::Lista(elementos) => {
                        let i = match &argumentos[1] {
                            Valor::Numero(n) => *n as i64,
                            otro => {
                                return Err(format!(
                                    "el lugar de una lista debe ser un numero, no {}",
                                    otro.tipo()
                                ))
                            }
                        };
                        if i < 0 || i as usize >= elementos.len() {
                            return Err(format!(
                                "ese lugar no existe en la lista: pediste el {} pero la lista tiene {} {}",
                                i,
                                elementos.len(),
                                if elementos.len() == 1 {
                                    "elemento"
                                } else {
                                    "elementos"
                                }
                            ));
                        }
                        self.pila.push(elementos[i as usize].clone());
                        Ok(())
                    }
                    Valor::Mapa(pares) => {
                        let clave = match &argumentos[1] {
                            Valor::Numero(_) | Valor::Texto(_) | Valor::Logico(_) => {
                                &argumentos[1]
                            }
                            otro => {
                                return Err(format!(
                                    "la clave de un mapa tiene que ser un número, un texto o verdadero/falso, no {}",
                                    otro.tipo()
                                ))
                            }
                        };
                        match Valor::buscar_clave(pares, clave) {
                            Some(pos) => {
                                self.pila.push(pares[pos].1.clone());
                                Ok(())
                            }
                            None => Err(format!(
                                "ese mapa no tiene la clave {}",
                                clave.a_texto()
                            )),
                        }
                    }
                    otro => Err(format!(
                        "solo se puede usar [ ] con una lista o un mapa, no con {}",
                        otro.tipo()
                    )),
                }
            }
            // claves({21: "FTP", 22: "SSH"}) es [21, 22].
            "claves" => {
                self.cantidad(nombre, &argumentos, 1)?;
                match &argumentos[0] {
                    Valor::Mapa(pares) => {
                        self.pila.push(Valor::lista(
                            pares.iter().map(|(k, _)| k.clone()).collect(),
                        ));
                        Ok(())
                    }
                    otro => Err(format!("claves necesita un mapa, no {}", otro.tipo())),
                }
            }
            // valores({21: "FTP", 22: "SSH"}) es ["FTP", "SSH"].
            "valores" => {
                self.cantidad(nombre, &argumentos, 1)?;
                match &argumentos[0] {
                    Valor::Mapa(pares) => {
                        self.pila.push(Valor::lista(
                            pares.iter().map(|(_, v)| v.clone()).collect(),
                        ));
                        Ok(())
                    }
                    otro => Err(format!("valores necesita un mapa, no {}", otro.tipo())),
                }
            }
            // tiene(mapa, 21) dice si esa clave está en el mapa.
            "tiene" => {
                self.cantidad(nombre, &argumentos, 2)?;
                match &argumentos[0] {
                    Valor::Mapa(pares) => {
                        let clave = match &argumentos[1] {
                            Valor::Numero(_) | Valor::Texto(_) | Valor::Logico(_) => {
                                &argumentos[1]
                            }
                            otro => {
                                return Err(format!(
                                    "la clave de un mapa tiene que ser un número, un texto o verdadero/falso, no {}",
                                    otro.tipo()
                                ))
                            }
                        };
                        self.pila
                            .push(Valor::Logico(Valor::buscar_clave(pares, clave).is_some()));
                        Ok(())
                    }
                    otro => Err(format!("tiene necesita un mapa, no {}", otro.tipo())),
                }
            }
            // fecha() es "2026-11-17", la fecha de hoy.
            "fecha" => {
                if !argumentos.is_empty() {
                    return Err("fecha no necesita datos entre paréntesis".to_string());
                }
                let (ano, mes, dia, _, _, _) = fecha_hora_partes();
                self.pila.push(Valor::Texto(format!(
                    "{:04}-{:02}-{:02}",
                    ano, mes, dia
                )));
                Ok(())
            }
            // hora() es "14:32:05", la hora de ahora.
            "hora" => {
                if !argumentos.is_empty() {
                    return Err("hora no necesita datos entre paréntesis".to_string());
                }
                let (_, _, _, h, m, s) = fecha_hora_partes();
                self.pila
                    .push(Valor::Texto(format!("{:02}:{:02}:{:02}", h, m, s)));
                Ok(())
            }
            // fecha_hora() es "2026-11-17 14:32:05".
            "fecha_hora" => {
                if !argumentos.is_empty() {
                    return Err("fecha_hora no necesita datos entre paréntesis".to_string());
                }
                let (ano, mes, dia, h, m, s) = fecha_hora_partes();
                self.pila.push(Valor::Texto(format!(
                    "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
                    ano, mes, dia, h, m, s
                )));
                Ok(())
            }
            // dormir(2) espera 2 segundos sin hacer nada.
            // Sirve para automatizar: esperar a que algo pase.
            "dormir" => {
                self.cantidad(nombre, &argumentos, 1)?;
                match &argumentos[0] {
                    Valor::Numero(n) => {
                        if !n.is_finite() || *n < 0.0 {
                            return Err(
                                "dormir necesita un número de segundos, no puede ser negativo"
                                    .to_string(),
                            );
                        }
                        std::thread::sleep(std::time::Duration::from_secs_f64(*n));
                        self.pila.push(Valor::Nulo);
                        Ok(())
                    }
                    otro => Err(format!("dormir necesita un numero, no {}", otro.tipo())),
                }
            }
            // existe("notas.txt") dice si ese archivo o esa
            // carpeta está ahí (verdadero o falso).
            "existe" => {
                self.cantidad(nombre, &argumentos, 1)?;
                match &argumentos[0] {
                    Valor::Texto(ruta) => {
                        self.pila
                            .push(Valor::Logico(std::path::Path::new(ruta).exists()));
                        Ok(())
                    }
                    otro => Err(format!(
                        "existe necesita el nombre entre comillas, no {}",
                        otro.tipo()
                    )),
                }
            }
            // listar_carpeta("mis_fotos") es la lista de archivos
            // y carpetas que hay ahí, en orden alfabético.
            "listar_carpeta" => {
                self.cantidad(nombre, &argumentos, 1)?;
                let ruta = match &argumentos[0] {
                    Valor::Texto(s) => s.clone(),
                    otro => {
                        return Err(format!(
                            "listar_carpeta necesita el nombre entre comillas, no {}",
                            otro.tipo()
                        ))
                    }
                };
                match std::fs::read_dir(&ruta) {
                    Ok(entradas) => {
                        let mut nombres: Vec<String> = Vec::new();
                        for entrada in entradas.flatten() {
                            nombres.push(entrada.file_name().to_string_lossy().to_string());
                        }
                        nombres.sort();
                        self.pila.push(Valor::lista(
                            nombres.into_iter().map(Valor::Texto).collect(),
                        ));
                        Ok(())
                    }
                    Err(_) => Err(format!("no encontré la carpeta '{}'", ruta)),
                }
            }
            // crear_carpeta("respaldo") crea esa carpeta (y las
            // que falten en el camino, si hace falta).
            "crear_carpeta" => {
                self.cantidad(nombre, &argumentos, 1)?;
                let ruta = match &argumentos[0] {
                    Valor::Texto(s) => s.clone(),
                    otro => {
                        return Err(format!(
                            "crear_carpeta necesita el nombre entre comillas, no {}",
                            otro.tipo()
                        ))
                    }
                };
                match std::fs::create_dir_all(&ruta) {
                    Ok(()) => {
                        self.pila.push(Valor::Nulo);
                        Ok(())
                    }
                    Err(_) => Err(format!("no pude crear la carpeta '{}'", ruta)),
                }
            }
            // mover_archivo("viejo.txt", "nuevo.txt") cambia el
            // nombre o mueve el archivo a otra carpeta.
            "mover_archivo" => {
                self.cantidad(nombre, &argumentos, 2)?;
                let viejo = match &argumentos[0] {
                    Valor::Texto(s) => s.clone(),
                    otro => {
                        return Err(format!(
                            "mover_archivo necesita los nombres entre comillas, no {}",
                            otro.tipo()
                        ))
                    }
                };
                let nuevo = match &argumentos[1] {
                    Valor::Texto(s) => s.clone(),
                    otro => {
                        return Err(format!(
                            "mover_archivo necesita los nombres entre comillas, no {}",
                            otro.tipo()
                        ))
                    }
                };
                match std::fs::rename(&viejo, &nuevo) {
                    Ok(()) => {
                        self.pila.push(Valor::Nulo);
                        Ok(())
                    }
                    Err(_) => Err(format!("no pude mover '{}' a '{}'", viejo, nuevo)),
                }
            }
            // borrar_archivo("viejo.txt") borra ese archivo.
            "borrar_archivo" => {
                self.cantidad(nombre, &argumentos, 1)?;
                let ruta = match &argumentos[0] {
                    Valor::Texto(s) => s.clone(),
                    otro => {
                        return Err(format!(
                            "borrar_archivo necesita el nombre entre comillas, no {}",
                            otro.tipo()
                        ))
                    }
                };
                match std::fs::remove_file(&ruta) {
                    Ok(()) => {
                        self.pila.push(Valor::Nulo);
                        Ok(())
                    }
                    Err(_) => Err(format!("no pude borrar el archivo '{}'", ruta)),
                }
            }
            // reemplazar("hola mundo", "mundo", "Kevin") es
            // "hola Kevin": cambia todas las veces que aparece.
            "reemplazar" => {
                self.cantidad(nombre, &argumentos, 3)?;
                match (&argumentos[0], &argumentos[1], &argumentos[2]) {
                    (Valor::Texto(t), Valor::Texto(viejo), Valor::Texto(nuevo)) => {
                        if viejo.is_empty() {
                            return Err(
                                "el texto a reemplazar no puede estar vacío".to_string()
                            );
                        }
                        self.pila
                            .push(Valor::Texto(t.replace(viejo.as_str(), nuevo)));
                        Ok(())
                    }
                    (a, b, c) => Err(format!(
                        "reemplazar necesita tres textos, no {}, {} y {}",
                        a.tipo(),
                        b.tipo(),
                        c.tipo()
                    )),
                }
            }
            _ => {
                // Una función tuya: tiene que estar registrada
                // (definida antes de llamarla). El mensaje es el
                // mismo que da el intérprete.
                let pedazo = match self.registradas.get(nombre) {
                    Some(p) => Rc::clone(p),
                    None => {
                        return Err(format!("no conozco la función '{}'", nombre));
                    }
                };
                if argumentos.len() != pedazo.parametros.len() {
                    return Err(format!(
                        "la función '{}' necesita {} {}, pero le diste {}",
                        nombre,
                        pedazo.parametros.len(),
                        if pedazo.parametros.len() == 1 {
                            "dato"
                        } else {
                            "datos"
                        },
                        argumentos.len()
                    ));
                }
                // Estabilidad: demasiadas funciones anidadas.
                if self.marcos.len() >= MAX_MARCOS {
                    return Err(
                        "demasiadas funciones anidadas, ¿tienes una función que se llama a sí misma sin parar?"
                            .to_string(),
                    );
                }
                // Los parámetros son los primeros slots del marco.
                // Los que se prestaron para cambiar (&mut) son
                // cambiables, para que la función pueda escribir
                // en ellos (y el valor nuevo vuelva al terminar).
                // Y todo dato prestado (& o &mut) se marca como
                // prestado: la función puede mirarlo, pero no
                // puede moverlo a otro dueño mientras dure el
                // préstamo.
                let mut slots = vec![Celda::nueva(Valor::Nulo, false); pedazo.max_slots];
                for (i, valor) in argumentos.into_iter().enumerate() {
                    if i < slots.len() {
                        let mutable = self
                            .prestamos_pendientes
                            .iter()
                            .any(|p| p.param == i);
                        let prestado = mutable
                            || self.prestados_pendientes.contains(&i);
                        let mut celda = Celda::nueva(valor, mutable);
                        celda.prestado = prestado;
                        slots[i] = celda;
                    }
                }
                // Los préstamos de esta llamada viajan con el
                // marco: los &mut para escribirlos de vuelta al
                // volver, y los & solo como marca (ya se anotaron
                // en los cajones). Se limpian las listas.
                let devoluciones = std::mem::take(&mut self.prestamos_pendientes);
                self.prestados_pendientes.clear();
                let base_pila = self.pila.len();
                self.marcos.push(Marco {
                    pedazo,
                    ip: 0,
                    base_pila,
                    slots,
                    devoluciones,
                    nombre: nombre.to_string(),
                });
                Ok(())
            }
        }
    }

    // El siguiente número al azar, entre 0 y 99.
    fn siguiente_aleatorio(&mut self) -> u64 {
        let mut x = self.semilla;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.semilla = x;
        x % 100
    }

    // Ejecuta UNA instrucción.
    fn ejecutar(&mut self, instr: Instruccion) -> Result<(), String> {
        match instr {
            Instruccion::Constante(idx) => {
                let valor = self
                    .marcos
                    .last()
                    .and_then(|m| m.pedazo.constantes.get(idx).cloned())
                    .ok_or_else(|| "esa constante no existe".to_string())?;
                self.pila.push(valor);
                Ok(())
            }
            Instruccion::Nulo => {
                self.pila.push(Valor::Nulo);
                Ok(())
            }
            Instruccion::Verdadero => {
                self.pila.push(Valor::Logico(true));
                Ok(())
            }
            Instruccion::Falso => {
                self.pila.push(Valor::Logico(false));
                Ok(())
            }
            Instruccion::Sumar => {
                let b = self.sacar()?;
                let a = self.sacar()?;
                self.pila.push(aplicar_binario(a, &OperadorBinario::Sumar, b)?);
                Ok(())
            }
            Instruccion::Restar => {
                let b = self.sacar()?;
                let a = self.sacar()?;
                self.pila.push(aplicar_binario(a, &OperadorBinario::Restar, b)?);
                Ok(())
            }
            Instruccion::Multiplicar => {
                let b = self.sacar()?;
                let a = self.sacar()?;
                self.pila
                    .push(aplicar_binario(a, &OperadorBinario::Multiplicar, b)?);
                Ok(())
            }
            Instruccion::Dividir => {
                let b = self.sacar()?;
                let a = self.sacar()?;
                self.pila.push(aplicar_binario(a, &OperadorBinario::Dividir, b)?);
                Ok(())
            }
            Instruccion::Residuo => {
                let b = self.sacar()?;
                let a = self.sacar()?;
                self.pila.push(aplicar_binario(a, &OperadorBinario::Residuo, b)?);
                Ok(())
            }
            Instruccion::Potencia => {
                let b = self.sacar()?;
                let a = self.sacar()?;
                self.pila
                    .push(aplicar_binario(a, &OperadorBinario::Potencia, b)?);
                Ok(())
            }
            Instruccion::Negativo => {
                let a = self.sacar()?;
                match a {
                    Valor::Numero(n) => {
                        self.pila.push(Valor::Numero(-n));
                        Ok(())
                    }
                    _ => Err(format!(
                        "no se puede cambiar el signo de un {}, solo de un numero",
                        a.tipo()
                    )),
                }
            }
            Instruccion::No => {
                let a = self.sacar()?;
                self.pila.push(Valor::Logico(!a.es_verdadero()));
                Ok(())
            }
            Instruccion::ConvertirLogico => {
                let a = self.sacar()?;
                self.pila.push(Valor::Logico(a.es_verdadero()));
                Ok(())
            }
            Instruccion::Igual => {
                let b = self.sacar()?;
                let a = self.sacar()?;
                self.pila.push(Valor::Logico(a == b));
                Ok(())
            }
            Instruccion::Diferente => {
                let b = self.sacar()?;
                let a = self.sacar()?;
                self.pila.push(Valor::Logico(a != b));
                Ok(())
            }
            Instruccion::Menor => {
                let b = self.sacar()?;
                let a = self.sacar()?;
                self.pila.push(aplicar_binario(a, &OperadorBinario::Menor, b)?);
                Ok(())
            }
            Instruccion::MenorIgual => {
                let b = self.sacar()?;
                let a = self.sacar()?;
                self.pila
                    .push(aplicar_binario(a, &OperadorBinario::MenorIgual, b)?);
                Ok(())
            }
            Instruccion::Mayor => {
                let b = self.sacar()?;
                let a = self.sacar()?;
                self.pila.push(aplicar_binario(a, &OperadorBinario::Mayor, b)?);
                Ok(())
            }
            Instruccion::MayorIgual => {
                let b = self.sacar()?;
                let a = self.sacar()?;
                self.pila
                    .push(aplicar_binario(a, &OperadorBinario::MayorIgual, b)?);
                Ok(())
            }
            Instruccion::Sacar => {
                self.sacar()?;
                Ok(())
            }
            Instruccion::DefinirVariable(slot, mutable, global, _) => {
                let valor = self.sacar()?;
                self.definir(slot, global, valor, mutable);
                Ok(())
            }
            Instruccion::DefinirConstante(slot, global, _) => {
                let valor = self.sacar()?;
                self.definir_constante(slot, global, valor);
                Ok(())
            }
            Instruccion::CrearLista(n) => {
                // Sacar n valores y poner una lista con ellos.
                let mut elementos = Vec::new();
                for _ in 0..n {
                    elementos.push(self.sacar()?);
                }
                elementos.reverse();
                self.pila.push(Valor::lista(elementos));
                Ok(())
            }
            Instruccion::Indice => {
                let indice = self.sacar()?;
                let lista = self.sacar()?;
                match lista {
                    Valor::Lista(elementos) => {
                        let i = match indice {
                            Valor::Numero(n) => n as i64,
                            otro => {
                                return Err(format!(
                                    "el lugar de una lista debe ser un numero, no {}",
                                    otro.tipo()
                                ))
                            }
                        };
                        if i < 0 || i as usize >= elementos.len() {
                            return Err(format!(
                                "ese lugar no existe en la lista: pediste el {} pero la lista tiene {} {}",
                                i,
                                elementos.len(),
                                if elementos.len() == 1 {
                                    "elemento"
                                } else {
                                    "elementos"
                                }
                            ));
                        }
                        self.pila.push(elementos[i as usize].clone());
                        Ok(())
                    }
                    Valor::Mapa(pares) => {
                        // La clave de un mapa es un número, un
                        // texto o verdadero/falso (no una lista).
                        let clave = match &indice {
                            Valor::Numero(_) | Valor::Texto(_) | Valor::Logico(_) => {
                                indice.clone()
                            }
                            otro => {
                                return Err(format!(
                                    "la clave de un mapa tiene que ser un número, un texto o verdadero/falso, no {}",
                                    otro.tipo()
                                ))
                            }
                        };
                        match Valor::buscar_clave(&pares, &clave) {
                            Some(pos) => {
                                self.pila.push(pares[pos].1.clone());
                                Ok(())
                            }
                            None => Err(format!(
                                "ese mapa no tiene la clave {}",
                                clave.a_texto()
                            )),
                        }
                    }
                    otro => Err(format!(
                        "solo se puede usar [ ] con una lista o un mapa, no con {}",
                        otro.tipo()
                    )),
                }
            }
            Instruccion::LeerEntrada(slot, global, nombre_idx, mensaje_idx) => {
                if let Some(m) = mensaje_idx {
                    if let Some(crate::motor::valor::Valor::Texto(mensaje)) =
                        self.constante_actual(m)
                    {
                        print!("{}", mensaje);
                        let _ = io::Write::flush(&mut io::stdout());
                    }
                }
                self.leer_entrada(slot, global, nombre_idx)
            }
            Instruccion::Incrementar(slot, global, nombre_idx, subir) => {
                self.incrementar(slot, global, nombre_idx, subir)
            }
            Instruccion::ObtenerVariable(slot, global, nombre_idx) => {
                let valor = self.obtener(slot, global, nombre_idx)?;
                self.pila.push(valor);
                Ok(())
            }
            Instruccion::AsignarVariable(slot, global, nombre_idx) => {
                let valor = self.sacar()?;
                self.asignar(slot, global, nombre_idx, valor)
            }
            Instruccion::Mover(slot, global, nombre_idx, destino_idx) => {
                self.mover(slot, global, nombre_idx, destino_idx)
            }
            Instruccion::PrestarMut(slot, global, nombre_idx, param) => {
                // Prestar para cambiar: revisa que la variable
                // se pueda cambiar (sea mut) y que sea un número,
                // un lógico o una lista. La escritura de vuelta
                // se hace cuando la función termina (Retornar).
                let nombre = self.nombre_actual(nombre_idx)?;
                let (valor, mutable) = self
                    .leer(slot, global)
                    .ok_or_else(|| format!("no conozco la variable '{}'", nombre))?;
                match valor {
                    Valor::Numero(_) | Valor::Logico(_) | Valor::Lista(_) => {}
                    otro => {
                        return Err(format!(
                            "solo se puede prestar para cambiar un número, un lógico o una lista, no {}",
                            otro.tipo()
                        ));
                    }
                }
                if !mutable {
                    return Err(format!(
                        "no se puede prestar para cambiar '{}' porque es fija. Si quieres que una función la cambie, créala con 'sea mut'",
                        nombre
                    ));
                }
                self.prestamos_pendientes.push(Prestamo {
                    slot,
                    global,
                    nombre_idx,
                    param,
                    tipo_original: valor.tipo().to_string(),
                });
                Ok(())
            }
            Instruccion::Prestar(_slot, _global, _nombre_idx, param) => {
                // Prestar solo para mirar: anota qué dato de la
                // función es, para marcarlo como prestado cuando
                // se cree el marco (y que no se pueda mover).
                self.prestados_pendientes.push(param);
                Ok(())
            }
            Instruccion::VerificarTipo(tipo_idx, _slot, _global, nombre_idx) => {
                // El valor que se está definiendo está arriba de
                // la pila (la variable todavía no existe en su
                // slot). Solo se mira, no se saca.
                let tipo = self.nombre_actual(tipo_idx)?;
                let nombre = self.nombre_actual(nombre_idx)?;
                let valor = self
                    .pila
                    .last()
                    .ok_or_else(|| "la pila se quedó vacía, algo salió mal".to_string())?;
                let real = valor.tipo();
                if tipo != real {
                    return Err(format!(
                        "la variable '{}' debe ser {}, pero le diste {}",
                        nombre, tipo, real
                    ));
                }
                Ok(())
            }
            Instruccion::VerificarLista(slot, global, nombre_idx) => {
                // `para_cada` necesita una lista: si la variable
                // es otra cosa, se dice con el mismo mensaje que
                // en el intérprete y en el .exe.
                let (valor, _) = self
                    .leer(slot, global)
                    .ok_or_else(|| "la variable no existe, algo salió mal".to_string())?;
                if !matches!(valor, Valor::Lista(_)) {
                    return Err(format!(
                        "'para_cada' necesita una lista, no {}",
                        valor.tipo()
                    ));
                }
                let _ = nombre_idx;
                Ok(())
            }
            Instruccion::ParaAvanzar(info) => {
                // Avanzar un `para` en un solo paso: la variable
                // es variable + paso; si sigue en rango, volver
                // al cuerpo; si no, seguir adelante (fin del
                // bucle). Es lo mismo que hacían 5 instrucciones
                // (leer, leer, sumar, definir, saltar), pero en
                // una sola, sin tocar la pila.
                let (var_val, _) = self
                    .leer(info.var_slot, info.var_global)
                    .ok_or_else(|| "la variable del bucle no existe, algo salió mal".to_string())?;
                let (fin_val, _) = self
                    .leer(info.fin_slot, info.fin_global)
                    .ok_or_else(|| "el fin del bucle no existe, algo salió mal".to_string())?;
                let (paso_val, _) = self
                    .leer(info.paso_slot, info.paso_global)
                    .ok_or_else(|| "el paso del bucle no existe, algo salió mal".to_string())?;
                let (var, fin, paso) = match (var_val, fin_val, paso_val) {
                    (Valor::Numero(v), Valor::Numero(f), Valor::Numero(p)) => (v, f, p),
                    _ => {
                        return Err(
                            "el bucle 'para' necesita números".to_string(),
                        );
                    }
                };
                let nuevo = var + paso;
                self.definir(
                    info.var_slot,
                    info.var_global,
                    Valor::Numero(nuevo),
                    false,
                );
                let sigue = if paso > 0.0 {
                    if info.inclusivo {
                        nuevo <= fin
                    } else {
                        nuevo < fin
                    }
                } else if paso < 0.0 {
                    if info.inclusivo {
                        nuevo >= fin
                    } else {
                        nuevo > fin
                    }
                } else {
                    false
                };
                if sigue {
                    if let Some(marco) = self.marcos.last_mut() {
                        marco.ip = info.salto_cuerpo;
                    }
                }
                Ok(())
            }
            Instruccion::AgregarEnSitio(slot, global, nombre_idx) => {
                // El atajo de `x = agregar(x, v)`: el valor a
                // meter está arriba de la pila. Las revisiones
                // son las mismas que harían `agregar` y la
                // asignación por separado, en el mismo orden:
                // movida, no es lista, constante, fija.
                let valor = self.sacar()?;
                enum Problema {
                    Bien,
                    Movido(String),
                    NoLista(String),
                    Constante,
                    Fija,
                    NoExiste,
                }
                let problema = {
                    let entrada = if global {
                        self.globales.get_mut(slot)
                    } else {
                        self.marcos
                            .last_mut()
                            .and_then(|m| m.slots.get_mut(slot))
                    };
                    match entrada {
                        Some(celda) => {
                            if let Some(destino) = &celda.movido {
                                Problema::Movido(destino.clone())
                            } else if !matches!(celda.valor, Valor::Lista(_)) {
                                Problema::NoLista(celda.valor.tipo().to_string())
                            } else if celda.constante {
                                Problema::Constante
                            } else if !celda.mutable {
                                Problema::Fija
                            } else {
                                if let Valor::Lista(elementos) = &mut celda.valor {
                                    Arc::make_mut(elementos).push(valor);
                                }
                                Problema::Bien
                            }
                        }
                        None => Problema::NoExiste,
                    }
                };
                match problema {
                    Problema::Bien => Ok(()),
                    Problema::Movido(destino) => {
                        let nombre = self.nombre_actual(nombre_idx)?;
                        Err(format!(
                            "la variable '{}' se movió {}, ya no puedes usarla. Para quedarte con una copia, usa copiar({})",
                            nombre, destino, nombre
                        ))
                    }
                    Problema::NoLista(tipo) => Err(format!(
                        "agregar necesita una lista, no {}",
                        tipo
                    )),
                    Problema::Constante => {
                        let nombre = self.nombre_actual(nombre_idx)?;
                        Err(format!(
                            "la constante '{}' no se puede cambiar, nunca. Si quieres una variable que cambie, créala con 'sea mut'",
                            nombre
                        ))
                    }
                    Problema::Fija => {
                        let nombre = self.nombre_actual(nombre_idx)?;
                        Err(format!(
                            "la variable '{}' es fija, no se puede cambiar. Si quieres cambiarla, créala con 'sea mut'",
                            nombre
                        ))
                    }
                    Problema::NoExiste => {
                        let nombre = self.nombre_actual(nombre_idx)?;
                        Err(format!("no conozco la variable '{}'", nombre))
                    }
                }
            }
            Instruccion::Saltar(destino) => {
                if let Some(marco) = self.marcos.last_mut() {
                    marco.ip = destino;
                }
                Ok(())
            }
            Instruccion::SaltarSiFalso(destino) => {
                let valor = self.sacar()?;
                if !valor.es_verdadero() {
                    if let Some(marco) = self.marcos.last_mut() {
                        marco.ip = destino;
                    }
                }
                Ok(())
            }
            Instruccion::Llamar(nombre_idx, cantidad) => {
                let nombre = self.nombre_actual(nombre_idx)?;
                self.llamar(&nombre, cantidad)
            }
            Instruccion::RegistrarFuncion(nombre_idx) => {
                let nombre = self.nombre_actual(nombre_idx)?;
                if let Some(pedazo) = self.funciones.get(&nombre) {
                    self.registradas.insert(nombre, Rc::clone(pedazo));
                }
                Ok(())
            }
            Instruccion::Retornar => {
                let valor = self.sacar()?;
                let marco = self
                    .marcos
                    .pop()
                    .ok_or_else(|| "no hay ninguna función corriendo".to_string())?;
                // &mut: escribir de vuelta los valores de los
                // préstamos a las variables que los prestaron.
                // Si la función cambió el tipo, da error.
                for prestamo in &marco.devoluciones {
                    if let Some(celda) = marco.slots.get(prestamo.param) {
                        let v = celda.valor.clone();
                        if v.tipo() != prestamo.tipo_original {
                            let nombre = self.nombre_actual(prestamo.nombre_idx)?;
                            return Err(format!(
                                "la función '{}' cambió el tipo de '{}': era {} y ahora es {}",
                                marco.nombre,
                                nombre,
                                prestamo.tipo_original,
                                v.tipo()
                            ));
                        }
                        self.asignar(prestamo.slot, prestamo.global, prestamo.nombre_idx, v)?;
                    }
                }
                self.pila.truncate(marco.base_pila);
                self.pila.push(valor);
                // Al cerrar la función, se van también los
                // manejadores que se abrieron dentro de ella.
                self.podar_manejadores();
                Ok(())
            }
            Instruccion::CrearMapa(n) => {
                // Sacar 2n valores (clave, valor, clave, valor...)
                // y poner un mapa con ellos. Si una clave se
                // repite, el último valor es el que queda.
                let mut pares: Vec<(Valor, Valor)> = Vec::new();
                for _ in 0..n {
                    let valor = self.sacar()?;
                    let clave = self.sacar()?;
                    match &clave {
                        Valor::Numero(_) | Valor::Texto(_) | Valor::Logico(_) => {}
                        otro => {
                            return Err(format!(
                                "la clave de un mapa tiene que ser un número, un texto o verdadero/falso, no {}",
                                otro.tipo()
                            ))
                        }
                    }
                    pares.push((clave, valor));
                }
                pares.reverse();
                let mut unicos: Vec<(Valor, Valor)> = Vec::new();
                for (clave, valor) in pares {
                    match Valor::buscar_clave(&unicos, &clave) {
                        Some(pos) => unicos[pos].1 = valor,
                        None => unicos.push((clave, valor)),
                    }
                }
                self.pila.push(Valor::mapa(unicos));
                Ok(())
            }
            Instruccion::AsignarIndice(slot, global, nombre_idx) => {
                // Sacar el valor nuevo y la clave, y cambiar
                // ese lugar dentro de la variable, en su sitio:
                // lista[0] = 5;  mapa[21] = "FTP";
                let valor = self.sacar()?;
                let indice = self.sacar()?;
                self.asignar_indice(slot, global, nombre_idx, indice, valor)
            }
            Instruccion::Intentar(info) => {
                self.manejadores.push(Manejador {
                    pila: self.pila.len(),
                    marcos: self.marcos.len(),
                    salto_falla: info.salto_falla,
                    salto_final: info.salto_final,
                });
                Ok(())
            }
            Instruccion::QuitarManejador => {
                self.manejadores
                    .pop()
                    .ok_or_else(|| "no hay ningún intentar abierto, algo salió mal".to_string())?;
                Ok(())
            }
            Instruccion::Relanzar => match self.relanzar.take() {
                Some(e) => Err(e),
                None => Ok(()),
            },
        }
    }
}

// La semilla inicial para aleatorio(): los nanosegundos
// del reloj, para que no salga siempre lo mismo.
fn semilla_inicial() -> u64 {
    let ahora = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0x9E3779B97F4A7C15);
    if ahora == 0 {
        0x9E3779B97F4A7C15
    } else {
        ahora
    }
}

// Descarga la salida estándar para que `imprimir` no se
// quede guardado sin salir (lo usa el REPL).
#[allow(dead_code)]
// El dato que nombra una instrucción (si es que nombra una
// variable): su slot, si es global, y su nombre. Sirve para
// que el depurador encuentre variables por su nombre.
fn dato_de(instr: &Instruccion, pedazo: &Pedazo) -> Option<(usize, bool, String)> {
    let (slot, global, idx) = match instr {
        Instruccion::DefinirVariable(s, _, g, n) => (*s, *g, *n),
        Instruccion::DefinirConstante(s, g, n) => (*s, *g, *n),
        Instruccion::ObtenerVariable(s, g, n) => (*s, *g, *n),
        Instruccion::AsignarVariable(s, g, n) => (*s, *g, *n),
        Instruccion::Mover(s, g, n, _) => (*s, *g, *n),
        Instruccion::PrestarMut(s, g, n, _) => (*s, *g, *n),
        Instruccion::Prestar(s, g, n, _) => (*s, *g, *n),
        Instruccion::LeerEntrada(s, g, n, _) => (*s, *g, *n),
        Instruccion::Incrementar(s, g, n, _) => (*s, *g, *n),
        Instruccion::VerificarLista(s, g, n) => (*s, *g, *n),
        Instruccion::AgregarEnSitio(s, g, n) => (*s, *g, *n),
        Instruccion::AsignarIndice(s, g, n) => (*s, *g, *n),
        _ => return None,
    };
    Some((slot, global, pedazo.nombre(idx)))
}

// Cambia un lugar dentro de una lista o un mapa, en su
// sitio: lista[0] = 5;  mapa[21] = "FTP";
// Si otro también tiene esa lista o ese mapa, no se toca
// el suyo: se clona (como hace `agregar`).
fn cambiar_lugar(objetivo: &mut Valor, indice: Valor, valor: Valor) -> Result<(), String> {
    match objetivo {
        Valor::Lista(elementos) => {
            let i = match indice {
                Valor::Numero(n) => n as i64,
                otro => {
                    return Err(format!(
                        "el lugar de una lista debe ser un numero, no {}",
                        otro.tipo()
                    ))
                }
            };
            if i < 0 || i as usize >= elementos.len() {
                return Err(format!(
                    "ese lugar no existe en la lista: pediste el {} pero la lista tiene {} {}",
                    i,
                    elementos.len(),
                    if elementos.len() == 1 {
                        "elemento"
                    } else {
                        "elementos"
                    }
                ));
            }
            Arc::make_mut(elementos)[i as usize] = valor;
            Ok(())
        }
        Valor::Mapa(pares) => {
            let clave = match &indice {
                Valor::Numero(_) | Valor::Texto(_) | Valor::Logico(_) => indice,
                otro => {
                    return Err(format!(
                        "la clave de un mapa tiene que ser un número, un texto o verdadero/falso, no {}",
                        otro.tipo()
                    ))
                }
            };
            let lista = Arc::make_mut(pares);
            match Valor::buscar_clave(lista, &clave) {
                Some(pos) => lista[pos].1 = valor,
                None => lista.push((clave, valor)),
            }
            Ok(())
        }
        otro => Err(format!(
            "solo se puede cambiar un lugar de una lista o un mapa, no de {}",
            otro.tipo()
        )),
    }
}

// La fecha y la hora de ahora: (año, mes, día, hora, minuto,
// segundo). Se calcula desde los segundos del reloj con el
// algoritmo civil de Howard Hinnant (sin librerías de fuera:
// solo la librería estándar de Rust). La hora es UTC, como
// la de `ahora()`: en tu computadora es la hora de Londres,
// no la de México (eso se arregla con tu zona horaria).
fn fecha_hora_partes() -> (i64, u32, u32, u32, u32, u32) {
    let segundos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let dias = segundos.div_euclid(86_400);
    let resto = segundos.rem_euclid(86_400);
    let hora = (resto / 3600) as u32;
    let minuto = ((resto % 3600) / 60) as u32;
    let segundo = (resto % 60) as u32;
    // Días desde 1970-01-01 a fecha civil (año, mes, día).
    let z = dias + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let dia = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let mes = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let ano = if mes <= 2 { y + 1 } else { y };
    (ano, mes, dia, hora, minuto, segundo)
}
