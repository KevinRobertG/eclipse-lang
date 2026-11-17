// ============================================================
//  ECLIPSE 1.2 - El verificador de préstamos (Borrow Checker)
//  Archivo: src/motor/verificador.rs
//
//  Qué es esto:
//  Antes de correr un programa, este revisor lee el arbolito
//  y te regaña en español si algo está mal, SIN correr nada:
//
//      Oye, no puedes usar 'a' aquí: se la regalaste a 'b'
//      en la línea 3. Si quieres quedarte con una copia,
//      usa copiar(a).
//
//  Es como el borrow checker de Rust, pero en español y con
//  las reglas de Eclipse. Revisa:
//
//  - Usar una variable que se movió (texto o lista regalada).
//  - Usar una variable que no existe.
//  - Cambiar una variable fija o una constante.
//  - Llamar a una función que no existe, o con la cantidad
//    de datos equivocada.
//  - Prestar con `&mut` dos veces a la vez, o prestar una
//    variable fija para cambiarla.
//  - `romper`/`seguir` fuera de un bucle, `devolver` fuera
//    de una función.
//  - Decir que una variable es `numero` y darle un texto.
//
//  Lo que NO revisa (eso lo sigue revisando el programa al
//  correr, como siempre):
//  - Si una variable se mueve dentro de un `si` y se usa
//    después: el revisor no sabe si ese `si` se cumplió, así
//    que no te regaña; si de verdad se movió, el error sale
//    al correr, con el mismo mensaje de siempre.
//  - El tipo de lo que devuelve una función tuya (no lo sabe
//    hasta correrla): si te equivocas ahí, el error sale al
//    correr.
//  - Lo que pasa dentro de un archivo `importar`: solo revisa
//    los nombres de sus funciones, no su código.
// ============================================================

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use crate::herramientas::bibliotecas;
use crate::lenguaje::ast::{Declaracion, Expresion, OperadorBinario, OperadorUnario};
use crate::lenguaje::lexer::Lexer;
use crate::lenguaje::parser::Parser;

// Una foto de qué variables están movidas en cada ámbito:
// (a dónde se movió y en qué línea, o nada) y si quizá se movió.
type FotoMovidos = Vec<HashMap<String, (Option<(String, usize)>, bool)>>;

// Un error del verificador: en qué línea y qué pasó.
pub struct ErrorVerificacion {
    pub linea: usize,
    pub mensaje: String,
}

impl ErrorVerificacion {
    pub fn texto(&self) -> String {
        format!("Error en la línea {}: {}", self.linea, self.mensaje)
    }
}

fn error(linea: usize, mensaje: String) -> ErrorVerificacion {
    ErrorVerificacion { linea, mensaje }
}

// Cómo se usa una expresión: solo mirarla, o llevársela.
#[derive(Clone, Copy, PartialEq)]
enum Modo {
    Leer,
    Mover,
}

// Lo que sabemos de una variable.
#[derive(Clone)]
struct InfoVar {
    mutable: bool,
    constante: bool,
    // Si se movió, a dónde se fue y en qué línea.
    movido: Option<(String, usize)>,
    // Se movió dentro de un `si` o un bucle: quizá se movió,
    // quizá no. No es error usarla (el programa al correr
    // dirá si de verdad se movió).
    movido_quiza: bool,
    // Qué tipo tiene, si lo sabemos ("numero", "texto",
    // "lista", "logico"). Si no lo sabemos, es None.
    tipo: Option<String>,
    // Si es una lista escrita a mano, cuántos elementos tiene
    // (para avisar si pides un lugar que no existe).
    largo_lista: Option<usize>,
}

// Una función que trae Eclipse: cuántos datos pide como
// mínimo y como máximo, y qué devuelve.
struct InfoBuiltin {
    min: usize,
    max: usize,
    devuelve: &'static str,
}

fn builtins() -> HashMap<&'static str, InfoBuiltin> {
    let mut b = HashMap::new();
    let mut poner = |nombre: &'static str, min: usize, max: usize, devuelve: &'static str| {
        b.insert(nombre, InfoBuiltin { min, max, devuelve });
    };
    // imprimir acepta 1 o más datos.
    poner("imprimir", 1, usize::MAX, "nulo");
    poner("leer", 0, 0, "texto");
    poner("leer_numero", 0, 0, "numero");
    poner("a_numero", 1, 1, "numero");
    poner("a_texto", 1, 1, "texto");
    poner("leer_archivo", 1, 1, "texto");
    poner("escribir_archivo", 2, 2, "nulo");
    poner("largo", 1, 1, "numero");
    poner("tipo_de", 1, 1, "texto");
    poner("aleatorio", 0, 0, "numero");
    poner("copiar", 1, 1, "?");
    poner("min", 2, 2, "numero");
    poner("max", 2, 2, "numero");
    poner("ahora", 0, 0, "numero");
    poner("redondear", 1, 1, "numero");
    poner("absoluto", 1, 1, "numero");
    poner("mayusculas", 1, 1, "texto");
    poner("minusculas", 1, 1, "texto");
    poner("contiene", 2, 2, "logico");
    poner("ordenar", 1, 1, "lista");
    poner("invertir", 1, 1, "?");
    poner("unir", 2, 2, "texto");
    poner("separar", 2, 2, "lista");
    poner("raiz", 1, 1, "numero");
    poner("agregar", 2, 2, "lista");
    poner("elemento", 2, 2, "?");
    // Hablar con el hardware y el software (desde la 1.0).
    poner("sistema", 0, 0, "texto");
    poner("usuario", 0, 0, "texto");
    poner("carpeta", 0, 0, "texto");
    poner("variable_entorno", 1, 1, "texto");
    poner("comando", 1, 1, "texto");
    poner("argumentos", 0, 0, "lista");
    poner("procesadores", 0, 0, "numero");
    poner("memoria", 0, 0, "numero");
    poner("caracter", 1, 1, "texto");
    // Mapas (desde la 1.2).
    poner("claves", 1, 1, "lista");
    poner("valores", 1, 1, "lista");
    poner("tiene", 2, 2, "logico");
    // Fecha, hora y automatización (desde la 1.2).
    poner("fecha", 0, 0, "texto");
    poner("hora", 0, 0, "texto");
    poner("fecha_hora", 0, 0, "texto");
    poner("dormir", 1, 1, "nulo");
    poner("existe", 1, 1, "logico");
    poner("listar_carpeta", 1, 1, "lista");
    poner("crear_carpeta", 1, 1, "nulo");
    poner("mover_archivo", 2, 2, "nulo");
    poner("borrar_archivo", 1, 1, "nulo");
    poner("reemplazar", 3, 3, "texto");
    b
}

pub struct Verificador {
    // Los ámbitos: una pila de cajitas con variables.
    ambitos: Vec<HashMap<String, InfoVar>>,
    // Las funciones que ya conocemos: nombre -> datos que pide.
    funciones: HashMap<String, usize>,
    // Archivos importar/ya leídos (para no leer dos veces).
    importados: HashSet<PathBuf>,
    en_bucle: usize,
    en_funcion: bool,
    // Lo que devuelve la función actual (tipo y línea del
    // primer `devolver`): una función siempre devuelve el
    // mismo tipo de dato.
    devuelve: Option<(String, usize)>,
    base: PathBuf,
    builtins: HashMap<&'static str, InfoBuiltin>,
}

// Revisa un programa completo antes de correrlo.
pub fn verificar(
    declaraciones: &[Declaracion],
    base: &Path,
) -> Result<(), ErrorVerificacion> {
    let mut v = Verificador {
        ambitos: vec![HashMap::new()],
        funciones: HashMap::new(),
        importados: HashSet::new(),
        en_bucle: 0,
        en_funcion: false,
        devuelve: None,
        base: base.to_path_buf(),
        builtins: builtins(),
    };
    v.revisar_lista(declaraciones)
}

impl Verificador {
    // ---------- Ámbitos ----------

    fn buscar(&self, nombre: &str) -> Option<(usize, &InfoVar)> {
        for (i, ambito) in self.ambitos.iter().enumerate().rev() {
            if let Some(info) = ambito.get(nombre) {
                return Some((i, info));
            }
        }
        None
    }

    fn buscar_mut(&mut self, nombre: &str) -> Option<&mut InfoVar> {
        for ambito in self.ambitos.iter_mut().rev() {
            if let Some(info) = ambito.get_mut(nombre) {
                return Some(info);
            }
        }
        None
    }

    fn definir(&mut self, nombre: String, mutable: bool, constante: bool, tipo: Option<String>) {
        let info = InfoVar {
            mutable,
            constante,
            movido: None,
            movido_quiza: false,
            tipo,
            largo_lista: None,
        };
        if let Some(ambito) = self.ambitos.last_mut() {
            ambito.insert(nombre, info);
        }
    }

    // Una foto de qué variables están movidas (para ramas).
    fn foto(&self) -> FotoMovidos {
        self.ambitos
            .iter()
            .map(|ambito| {
                ambito
                    .iter()
                    .map(|(n, info)| (n.clone(), (info.movido.clone(), info.movido_quiza)))
                    .collect()
            })
            .collect()
    }

    // Qué variables se movieron de más desde la foto.
    fn movidos_de_mas(
        &self,
        foto: &FotoMovidos,
    ) -> HashMap<String, (String, usize)> {
        let mut mas = HashMap::new();
        for (i, ambito) in self.ambitos.iter().enumerate() {
            for (nombre, info) in ambito.iter() {
                let antes = foto
                    .get(i)
                    .and_then(|f| f.get(nombre))
                    .map(|(m, _)| m.clone())
                    .unwrap_or(None);
                if antes.is_none() {
                    if let Some(m) = &info.movido {
                        mas.insert(nombre.clone(), m.clone());
                    }
                }
            }
        }
        mas
    }

    // Restaura la foto, pero marca como "quizá" lo que se movió.
    fn restaurar_con_quiza(&mut self, foto: &FotoMovidos) {
        for (i, ambito) in self.ambitos.iter_mut().enumerate() {
            for (nombre, info) in ambito.iter_mut() {
                if let Some((m, q)) = foto.get(i).and_then(|f| f.get(nombre)) {
                    let estaba_movido = m.is_some();
                    let ahora_movido = info.movido.is_some();
                    info.movido = m.clone();
                    info.movido_quiza = *q || (!estaba_movido && ahora_movido);
                }
            }
        }
    }

    // ---------- Variables ----------

    // Usar una variable: revisa que exista y que no se haya
    // movido. Si el modo es Mover y es texto o lista, se la
    // lleva (con el destino que le diga quien llama).
    fn usar_variable(
        &mut self,
        nombre: &str,
        linea: usize,
        modo: Modo,
        destino: Option<&str>,
    ) -> Result<Option<String>, ErrorVerificacion> {
        let info = match self.buscar(nombre) {
            Some((_, info)) => info.clone(),
            None => {
                return Err(error(
                    linea,
                    format!(
                        "Oye, no conozco la variable '{}'. ¿La creaste con 'sea' antes de usarla?",
                        nombre
                    ),
                ));
            }
        };
        if let Some((a_donde, linea_movio)) = &info.movido {
            return Err(error(
                linea,
                format!(
                    "Oye, no puedes usar '{}' aquí: se la regalaste {} en la línea {}. Si quieres quedarte con una copia, usa copiar({})",
                    nombre, a_donde, linea_movio, nombre
                ),
            ));
        }
        if modo == Modo::Mover {
            let se_mueve = matches!(info.tipo.as_deref(), Some("texto") | Some("lista") | Some("mapa"));
            if se_mueve {
                let destino = destino.unwrap_or("a otro lado").to_string();
                if let Some(info_mut) = self.buscar_mut(nombre) {
                    info_mut.movido = Some((destino, linea));
                }
            }
        }
        Ok(info.tipo)
    }

    // Prestar una variable para mirar (`&x`): solo revisa que
    // exista y que no se haya movido.
    fn prestar_variable(&mut self, nombre: &str, linea: usize) -> Result<(), ErrorVerificacion> {
        match self.buscar(nombre) {
            Some(_) => {
                self.usar_variable(nombre, linea, Modo::Leer, None)?;
                Ok(())
            }
            None => Err(error(
                linea,
                format!(
                    "Oye, no conozco la variable '{}'. ¿La creaste con 'sea' antes de prestarla?",
                    nombre
                ),
            )),
        }
    }

    // Prestar una variable para cambiar (`&mut x`).
    fn prestar_mut_variable(
        &mut self,
        nombre: &str,
        linea: usize,
    ) -> Result<(), ErrorVerificacion> {
        let info = match self.buscar(nombre) {
            Some((_, info)) => info.clone(),
            None => {
                return Err(error(
                    linea,
                    format!(
                        "Oye, no conozco la variable '{}'. ¿La creaste con 'sea' antes de prestarla?",
                        nombre
                    ),
                ));
            }
        };
        if info.movido.is_some() {
            let (a_donde, linea_movio) = info.movido.clone().unwrap();
            return Err(error(
                linea,
                format!(
                    "Oye, no puedes prestar '{}' para cambiar: se la regalaste {} en la línea {}.",
                    nombre, a_donde, linea_movio
                ),
            ));
        }
        if info.constante {
            return Err(error(
                linea,
                format!(
                    "Oye, no puedes prestar '{}' para cambiar porque es una constante y nunca cambia.",
                    nombre
                ),
            ));
        }
        if !info.mutable {
            return Err(error(
                linea,
                format!(
                    "Oye, no puedes prestar '{}' para cambiar porque es fija. Si quieres cambiarla, créala con 'sea mut'.",
                    nombre
                ),
            ));
        }
        Ok(())
    }

    // Revisar que una variable se pueda cambiar (asignar, ++).
    fn revisar_cambio(
        &mut self,
        nombre: &str,
        linea: usize,
    ) -> Result<(), ErrorVerificacion> {
        let info = match self.buscar(nombre) {
            Some((_, info)) => info.clone(),
            None => {
                return Err(error(
                    linea,
                    format!("Oye, no conozco la variable '{}'.", nombre),
                ));
            }
        };
        if info.constante {
            return Err(error(
                linea,
                format!(
                    "Oye, no puedes cambiar '{}' porque es una constante y nunca cambia.",
                    nombre
                ),
            ));
        }
        if !info.mutable {
            return Err(error(
                linea,
                format!(
                    "Oye, la variable '{}' es fija, no se puede cambiar. Si quieres cambiarla, créala con 'sea mut'.",
                    nombre
                ),
            ));
        }
        Ok(())
    }

    // ---------- Tipos ----------

    // Qué tipo tiene una expresión, si lo sabemos.
    fn tipo_de(&mut self, e: &Expresion) -> Option<String> {
        match e {
            Expresion::Numero(_) => Some("numero".to_string()),
            Expresion::Cadena(_) => Some("texto".to_string()),
            Expresion::Logico(_) => Some("logico".to_string()),
            Expresion::Nulo => Some("nulo".to_string()),
            Expresion::Variable(nombre) => {
                self.buscar(nombre).and_then(|(_, info)| info.tipo.clone())
            }
            Expresion::Agrupada(dentro) => self.tipo_de(dentro),
            Expresion::Unaria { operador, .. } => match operador {
                OperadorUnario::Negativo => Some("numero".to_string()),
                OperadorUnario::No => Some("logico".to_string()),
            },
            Expresion::Binaria { izquierda, operador, derecha } => match operador {
                OperadorBinario::Y | OperadorBinario::O => Some("logico".to_string()),
                OperadorBinario::Igual
                | OperadorBinario::Diferente
                | OperadorBinario::Menor
                | OperadorBinario::MenorIgual
                | OperadorBinario::Mayor
                | OperadorBinario::MayorIgual => Some("logico".to_string()),
                OperadorBinario::Sumar => {
                    let ti = self.tipo_de(izquierda);
                    let td = self.tipo_de(derecha);
                    if ti.as_deref() == Some("texto") || td.as_deref() == Some("texto") {
                        Some("texto".to_string())
                    } else if ti.as_deref() == Some("lista") || td.as_deref() == Some("lista") {
                        Some("lista".to_string())
                    } else {
                        Some("numero".to_string())
                    }
                }
                _ => Some("numero".to_string()),
            },
            Expresion::Llamada { nombre, .. } => {
                if let Some(info) = self.builtins.get(nombre.as_str()) {
                    if info.devuelve == "?" {
                        None
                    } else {
                        Some(info.devuelve.to_string())
                    }
                } else {
                    None
                }
            }
            Expresion::Interpolada(_) => Some("texto".to_string()),
            Expresion::Lista(_) => Some("lista".to_string()),
            Expresion::Mapa(_) => Some("mapa".to_string()),
            Expresion::Indice { .. } => None,
            Expresion::Prestar(dentro) | Expresion::PrestarMut(dentro) => self.tipo_de(dentro),
        }
    }

    // ---------- Expresiones ----------

    fn revisar_expresion(&mut self, e: &Expresion, modo: Modo, linea: usize) -> Result<(), ErrorVerificacion> {
        match e {
            Expresion::Numero(_) | Expresion::Cadena(_) | Expresion::Logico(_) | Expresion::Nulo => Ok(()),
            Expresion::Variable(nombre) => {
                self.usar_variable(nombre, linea, modo, None)?;
                Ok(())
            }
            Expresion::Agrupada(dentro) => {
                // (a) no mueve `a`: el paréntesis hace una copia.
                self.revisar_expresion(dentro, Modo::Leer, linea)
            }
            Expresion::Unaria { derecha, .. } => self.revisar_expresion(derecha, Modo::Leer, linea),
            Expresion::Binaria { izquierda, operador, derecha } => {
                self.revisar_expresion(izquierda, Modo::Leer, linea)?;
                self.revisar_expresion(derecha, Modo::Leer, linea)?;
                // Dividir entre un 0 escrito a mano no se puede
                // (ni el resto entre cero): te aviso antes de
                // correr, no cuando el programa se caiga.
                if es_cero_literal(derecha) {
                    match operador {
                        OperadorBinario::Dividir => {
                            return Err(error(
                                linea,
                                "Oye, estás dividiendo entre cero. Eso no se puede: el número de la derecha es 0."
                                    .to_string(),
                            ));
                        }
                        OperadorBinario::Residuo => {
                            return Err(error(
                                linea,
                                "Oye, estás calculando el resto entre cero. Eso no se puede: el número de la derecha es 0."
                                    .to_string(),
                            ));
                        }
                        _ => {}
                    }
                }
                Ok(())
            }
            Expresion::Llamada { nombre, argumentos } => {
                self.revisar_llamada(nombre, argumentos, linea)
            }
            Expresion::Interpolada(partes) => {
                for p in partes {
                    if let crate::lenguaje::ast::ParteCadena::Expresion(dentro) = p {
                        self.revisar_expresion(dentro, Modo::Leer, linea)?;
                    }
                }
                Ok(())
            }
            Expresion::Lista(elementos) => {
                for el in elementos {
                    self.revisar_expresion(el, Modo::Leer, linea)?;
                }
                Ok(())
            }
            Expresion::Mapa(pares) => {
                for (clave, valor) in pares {
                    // La clave tiene que ser un número, un texto
                    // o verdadero/falso (no una lista ni un mapa).
                    if let Some(tipo_clave) = self.tipo_de(clave) {
                        if tipo_clave != "numero"
                            && tipo_clave != "texto"
                            && tipo_clave != "logico"
                        {
                            return Err(error(
                                linea,
                                format!(
                                    "Oye, la clave de un mapa tiene que ser un número, un texto o verdadero/falso, no {}.",
                                    tipo_clave
                                ),
                            ));
                        }
                    }
                    self.revisar_expresion(clave, Modo::Leer, linea)?;
                    self.revisar_expresion(valor, Modo::Leer, linea)?;
                }
                Ok(())
            }
            Expresion::Indice { lista, indice } => {
                self.revisar_expresion(lista, Modo::Leer, linea)?;
                self.revisar_expresion(indice, Modo::Leer, linea)?;
                // El largo se sabe si la lista está escrita a mano
                // o si es una variable con una lista escrita a mano.
                let largo = match &**lista {
                    Expresion::Lista(elementos) => Some(elementos.len() as i64),
                    Expresion::Variable(nombre) => self
                        .buscar(nombre)
                        .and_then(|(_, info)| info.largo_lista.map(|l| l as i64)),
                    _ => None,
                };
                if let (Some(total), Some(n)) = (largo, numero_literal(indice)) {
                    if n.fract() == 0.0 {
                        let lugar = n as i64;
                        if lugar < 0 || lugar >= total {
                            return Err(error(
                                linea,
                                format!(
                                    "Oye, esa lista tiene {} {} (del lugar 0 al {}), pero pediste el lugar {}. Ese lugar no existe.",
                                    total,
                                    if total == 1 { "elemento" } else { "elementos" },
                                    total - 1,
                                    lugar
                                ),
                            ));
                        }
                    }
                }
                Ok(())
            }
            Expresion::Prestar(dentro) => {
                if let Expresion::Variable(nombre) = &**dentro {
                    self.prestar_variable(nombre, linea)
                } else {
                    self.revisar_expresion(dentro, Modo::Leer, linea)
                }
            }
            Expresion::PrestarMut(dentro) => {
                if let Expresion::Variable(nombre) = &**dentro {
                    self.prestar_mut_variable(nombre, linea)
                } else {
                    self.revisar_expresion(dentro, Modo::Leer, linea)
                }
            }
        }
    }

    fn revisar_llamada(
        &mut self,
        nombre: &str,
        argumentos: &[Expresion],
        linea: usize,
    ) -> Result<(), ErrorVerificacion> {
        // Las funciones que trae Eclipse (incluidas las de
        // hardware y software): piden lo que piden y solo
        // miran los datos, nunca se los llevan.
        if let Some(info) = self.builtins.get(nombre) {
            if argumentos.len() < info.min || argumentos.len() > info.max {
                let cuantos = if info.min == info.max {
                    format!("{}", info.min)
                } else if info.max == usize::MAX {
                    format!("{} o más", info.min)
                } else {
                    format!("entre {} y {}", info.min, info.max)
                };
                let palabra = if argumentos.len() == 1 { "dato" } else { "datos" };
                return Err(error(
                    linea,
                    format!(
                        "Oye, la función '{}' necesita {} datos, pero le diste {} {}.",
                        nombre,
                        cuantos,
                        argumentos.len(),
                        palabra
                    ),
                ));
            }
            for arg in argumentos {
                self.revisar_expresion(arg, Modo::Leer, linea)?;
            }
            return Ok(());
        }
        // Las funciones tuyas (y las de importar/usar): si les
        // pasas un texto o una lista, se la llevan (se mueve).
        let arity = match self.funciones.get(nombre) {
            Some(n) => *n,
            None => {
                return Err(error(
                    linea,
                    format!(
                        "Oye, no conozco la función '{}'. ¿La creaste con 'funcion' antes de llamarla, o te falta un 'usar' arriba?",
                        nombre
                    ),
                ));
            }
        };
        if argumentos.len() != arity {
            let palabra = if argumentos.len() == 1 { "dato" } else { "datos" };
            return Err(error(
                linea,
                format!(
                    "Oye, la función '{}' necesita {} datos, pero le diste {} {}.",
                    nombre, arity, argumentos.len(), palabra
                ),
            ));
        }
        // Primero, los préstamos: hay que revisarlos juntos,
        // porque prestar lo mismo dos veces para cambiar no vale.
        let mut prestados_mut: Vec<String> = Vec::new();
        for arg in argumentos {
            if let Expresion::PrestarMut(dentro) = arg {
                if let Expresion::Variable(n) = &**dentro {
                    if prestados_mut.contains(n) {
                        return Err(error(
                            linea,
                            format!(
                                "Oye, no puedes prestar '{}' dos veces para cambiar a la vez. Presta una copia con copiar({}) para la segunda.",
                                n, n
                            ),
                        ));
                    }
                    prestados_mut.push(n.clone());
                }
            }
        }
        for arg in argumentos {
            match arg {
                Expresion::Variable(nombre_var) => {
                    let destino = format!("a la función '{}'", nombre);
                    self.usar_variable(nombre_var, linea, Modo::Mover, Some(&destino))?;
                }
                _ => {
                    self.revisar_expresion(arg, Modo::Leer, linea)?;
                }
            }
        }
        Ok(())
    }

    // ---------- Declaraciones ----------

    fn revisar_lista(&mut self, lista: &[Declaracion]) -> Result<(), ErrorVerificacion> {
        for d in lista {
            self.revisar_declaracion(d)?;
        }
        Ok(())
    }

    // Revisa una rama (un bloque dentro de un `si`, un bucle...).
    // Devuelve qué variables se movieron de más dentro.
    fn revisar_rama(
        &mut self,
        cuerpo: &[Declaracion],
    ) -> Result<HashMap<String, (String, usize)>, ErrorVerificacion> {
        let foto = self.foto();
        self.ambitos.push(HashMap::new());
        let resultado = self.revisar_lista(cuerpo);
        self.ambitos.pop();
        let mas = self.movidos_de_mas(&foto);
        if resultado.is_err() {
            return resultado.map(|_| mas);
        }
        Ok(mas)
    }

    fn revisar_declaracion(&mut self, d: &Declaracion) -> Result<(), ErrorVerificacion> {
        let linea = d.linea();
        match d {
            Declaracion::Sea { nombre, mutable, tipo, valor, .. } => {
                let tipo_valor = match valor {
                    Some(e) => {
                        // `sea b = a;` se lleva `a` si es texto o lista.
                        match e {
                            Expresion::Variable(fuente) if fuente != nombre => {
                                let destino = format!("a la variable '{}'", nombre);
                                self.usar_variable(fuente, linea, Modo::Mover, Some(&destino))?;
                                self.tipo_de(e)
                            }
                            _ => {
                                self.revisar_expresion(e, Modo::Leer, linea)?;
                                self.tipo_de(e)
                            }
                        }
                    }
                    None => None,
                };
                // Si dijiste el tipo y el valor es otro, te aviso.
                if let (Some(t), Some(tv)) = (tipo, &tipo_valor) {
                    if t != tv && tv != "nulo" {
                        return Err(error(
                            linea,
                            format!(
                                "Oye, dijiste que '{}' es {}, pero le diste un {}.",
                                nombre, t, tv
                            ),
                        ));
                    }
                }
                let tipo_final = tipo.clone().or(tipo_valor);
                self.definir(nombre.clone(), *mutable, false, tipo_final);
                if let Some(e) = valor {
                    if let Some(largo) = largo_de(e) {
                        if let Some(info) = self.buscar_mut(nombre) {
                            info.largo_lista = Some(largo);
                        }
                    }
                }
                Ok(())
            }
            Declaracion::Constante { nombre, tipo, valor, .. } => {
                self.revisar_expresion(valor, Modo::Leer, linea)?;
                let tipo_final = tipo.clone().or_else(|| self.tipo_de(valor));
                self.definir(nombre.clone(), false, true, tipo_final);
                if let Some(largo) = largo_de(valor) {
                    if let Some(info) = self.buscar_mut(nombre) {
                        info.largo_lista = Some(largo);
                    }
                }
                Ok(())
            }
            Declaracion::Asignacion { nombre, valor, .. } => {
                // Primero el valor (como al correr), luego la caja.
                match valor {
                    // `x = agregar(x, v)` no se lleva `x`: lo
                    // mete en su sitio, sin copiar la lista.
                    Expresion::Llamada { nombre: f, argumentos }
                        if f == "agregar"
                            && argumentos.len() == 2
                            && matches!(&argumentos[0], Expresion::Variable(v) if v == nombre) =>
                    {
                        self.revisar_expresion(&argumentos[1], Modo::Leer, linea)?;
                    }
                    Expresion::Variable(fuente) if fuente != nombre => {
                        let destino = format!("a la variable '{}'", nombre);
                        self.usar_variable(fuente, linea, Modo::Mover, Some(&destino))?;
                    }
                    _ => {
                        self.revisar_expresion(valor, Modo::Leer, linea)?;
                    }
                }
                self.revisar_cambio(nombre, linea)?;
                // Al darle un valor nuevo, la variable revive.
                let tipo_nuevo = self.tipo_de(valor);
                let largo_nuevo = largo_de(valor);
                if let Some(info) = self.buscar_mut(nombre) {
                    info.movido = None;
                    info.movido_quiza = false;
                    info.tipo = tipo_nuevo;
                    info.largo_lista = largo_nuevo;
                }
                Ok(())
            }
            Declaracion::AsignacionCompuesta { nombre, valor, .. } => {
                self.revisar_expresion(valor, Modo::Leer, linea)?;
                self.revisar_cambio(nombre, linea)?;
                Ok(())
            }
            Declaracion::AsignarIndice { objetivo, indice, valor, .. } => {
                // El objetivo es una variable con `sea mut`
                // (el parser solo acepta eso); si no lo es,
                // es un error interno del compilador.
                let nombre = match objetivo {
                    Expresion::Variable(n) => n.clone(),
                    _ => {
                        return Err(error(
                            linea,
                            "Oye, solo se puede cambiar un lugar de una lista o un mapa guardado en una variable.".to_string(),
                        ));
                    }
                };
                // Debe existir, ser mutable y no haberse movido.
                self.revisar_cambio(&nombre, linea)?;
                if let Some((_, info)) = self.buscar(&nombre) {
                    if let Some((a_donde, linea_movio)) = &info.movido {
                        return Err(error(
                            linea,
                            format!(
                                "Oye, no puedes usar '{}' aquí: se la regalaste {} en la línea {}. Si quieres quedarte con una copia, usa copiar({})",
                                nombre, a_donde, linea_movio, nombre
                            ),
                        ));
                    }
                    // Si ya sabemos qué es y no es lista ni mapa,
                    // te aviso antes de correr.
                    if let Some(tipo) = &info.tipo {
                        if tipo != "lista" && tipo != "mapa" {
                            return Err(error(
                                linea,
                                format!(
                                    "Oye, solo se puede usar [] con una lista o un mapa, no con {}.",
                                    tipo
                                ),
                            ));
                        }
                    }
                    // Si es una lista de largo conocido y el
                    // lugar es un número escrito a mano fuera de
                    // rango, te aviso (igual que al leer).
                    if let (Some(total), Some(n)) =
                        (info.largo_lista, numero_literal(indice))
                    {
                        if n.fract() == 0.0 {
                            let lugar = n as i64;
                            if lugar < 0 || lugar as usize >= total {
                                return Err(error(
                                    linea,
                                    format!(
                                        "Oye, esa lista tiene {} {} (del lugar 0 al {}), pero pediste el lugar {}. Ese lugar no existe.",
                                        total,
                                        if total == 1 { "elemento" } else { "elementos" },
                                        total - 1,
                                        lugar
                                    ),
                                ));
                            }
                        }
                    }
                }
                self.revisar_expresion(indice, Modo::Leer, linea)?;
                self.revisar_expresion(valor, Modo::Leer, linea)?;
                Ok(())
            }
            Declaracion::Intentar { cuerpo, falla, al_final, .. } => {
                // El cuerpo corre al menos una vez, pero puede
                // fallar a la mitad: lo que se mueve dentro,
                // quizá se movió.
                let foto = self.foto();
                self.revisar_rama(cuerpo)?;
                self.restaurar_con_quiza(&foto);
                // El si_falla solo corre si hubo error: lo que
                // se mueve ahí dentro, quizá se movió. La
                // variable del error (si la hay) es un texto.
                if let Some((variable, cuerpo_falla)) = falla {
                    let foto_falla = self.foto();
                    self.ambitos.push(HashMap::new());
                    if let Some(nombre) = variable {
                        self.definir(nombre.clone(), false, false, Some("texto".to_string()));
                    }
                    let resultado = self.revisar_lista(cuerpo_falla);
                    self.ambitos.pop();
                    resultado?;
                    self.restaurar_con_quiza(&foto_falla);
                }
                // El al_final corre SIEMPRE: lo que se mueve
                // ahí dentro, seguro se movió.
                if let Some(cuerpo_final) = al_final {
                    self.revisar_lista(cuerpo_final)?;
                }
                Ok(())
            }
            Declaracion::Incrementar { nombre, .. } => {
                if let Some((_, info)) = self.buscar(nombre) {
                    if let Some(t) = &info.tipo {
                        if t != "numero" {
                            return Err(error(
                                linea,
                                format!(
                                    "Oye, solo se puede usar '++' o '--' con un numero, no con {}.",
                                    t
                                ),
                            ));
                        }
                    }
                }
                self.revisar_cambio(nombre, linea)?;
                Ok(())
            }
            Declaracion::Expresion { expresion: e, .. } => {
                self.revisar_expresion(e, Modo::Leer, linea)
            }
            Declaracion::Si { condicion, entonces, sino, .. } => {
                self.revisar_expresion(condicion, Modo::Leer, linea)?;
                let foto = self.foto();
                let movidos_si = self.revisar_rama(entonces)?;
                // Restaurar: el `sino` parte del mismo estado.
                self.restaurar_con_quiza(&foto);
                // Si no hay `sino`, lo que se movió quizá se movió.
                let movidos = match sino {
                    Some(otro) => {
                        let movidos_sino = self.revisar_rama(otro)?;
                        self.restaurar_con_quiza(&foto);
                        // Lo que se movió en LOS DOS brazos, seguro
                        // se movió (da igual cuál se cumplió).
                        let mut ambos: HashMap<String, (String, usize)> = HashMap::new();
                        for (nombre, destino) in movidos_si.iter() {
                            if movidos_sino.contains_key(nombre) {
                                ambos.insert(nombre.clone(), destino.clone());
                            }
                        }
                        for (nombre, destino) in ambos {
                            if let Some(info) = self.buscar_mut(&nombre) {
                                info.movido = Some(destino);
                                info.movido_quiza = false;
                            }
                        }
                        // Lo que se movió en un solo brazo, quizá.
                        for nombre in movidos_si.keys().chain(movidos_sino.keys()) {
                            if let Some(info) = self.buscar_mut(nombre) {
                                if info.movido.is_none() {
                                    info.movido_quiza = true;
                                }
                            }
                        }
                        return Ok(());
                    }
                    None => movidos_si,
                };
                for nombre in movidos.keys() {
                    if let Some(info) = self.buscar_mut(nombre) {
                        if info.movido.is_none() {
                            info.movido_quiza = true;
                        }
                    }
                }
                Ok(())
            }
            Declaracion::Mientras { condicion, cuerpo, .. } => {
                self.revisar_expresion(condicion, Modo::Leer, linea)?;
                let foto = self.foto();
                // Un `mientras` ES un bucle: `romper` y `seguir`
                // valen dentro de él (antes se me olvidó
                // contarlo y el verificador los rechazaba).
                self.en_bucle += 1;
                let resultado = self.revisar_rama(cuerpo);
                self.en_bucle -= 1;
                resultado?;
                self.restaurar_con_quiza(&foto);
                Ok(())
            }
            Declaracion::Repetir { cuerpo, condicion, .. } => {
                // `repetir` corre el cuerpo al menos una vez:
                // lo que se mueve dentro, seguro se movió.
                self.en_bucle += 1;
                self.revisar_lista(cuerpo)?;
                self.en_bucle -= 1;
                self.revisar_expresion(condicion, Modo::Leer, linea)?;
                Ok(())
            }
            Declaracion::Para { variable, inicio, fin, paso, cuerpo, .. } => {
                self.revisar_expresion(inicio, Modo::Leer, linea)?;
                self.revisar_expresion(fin, Modo::Leer, linea)?;
                if let Some(p) = paso {
                    self.revisar_expresion(p, Modo::Leer, linea)?;
                }
                let foto = self.foto();
                self.ambitos.push(HashMap::new());
                self.definir(variable.clone(), false, false, Some("numero".to_string()));
                self.en_bucle += 1;
                let resultado = self.revisar_lista(cuerpo);
                self.en_bucle -= 1;
                self.ambitos.pop();
                self.restaurar_con_quiza(&foto);
                resultado
            }
            Declaracion::ParaCada { variable, lista, cuerpo, .. } => {
                self.revisar_expresion(lista, Modo::Leer, linea)?;
                let foto = self.foto();
                self.ambitos.push(HashMap::new());
                self.definir(variable.clone(), false, false, None);
                self.en_bucle += 1;
                let resultado = self.revisar_lista(cuerpo);
                self.en_bucle -= 1;
                self.ambitos.pop();
                self.restaurar_con_quiza(&foto);
                resultado
            }
            Declaracion::Funcion { nombre, parametros, cuerpo, .. } => {
                // La función se conoce a sí misma (recursión).
                self.funciones.insert(nombre.clone(), parametros.len());
                let foto = self.foto();
                self.ambitos.push(HashMap::new());
                for p in parametros {
                    self.definir(p.clone(), false, false, None);
                }
                let en_funcion_viejo = self.en_funcion;
                self.en_funcion = true;
                // Cada función empieza sin saber qué devuelve.
                let devuelve_viejo = self.devuelve.take();
                let resultado = self.revisar_lista(cuerpo);
                self.devuelve = devuelve_viejo;
                self.en_funcion = en_funcion_viejo;
                self.ambitos.pop();
                // Lo que se mueve dentro de una función quizá se
                // mueve (solo si alguien la llama).
                self.restaurar_con_quiza(&foto);
                resultado
            }
            Declaracion::Devolver { valor, .. } => {
                if !self.en_funcion {
                    return Err(error(
                        linea,
                        "Oye, no puedes usar 'devolver' fuera de una función.".to_string(),
                    ));
                }
                // El tipo que devuelve: si es un `devolver;` sin
                // nada, devuelve nulo. Si esta función ya devolvió
                // otro tipo antes, te aviso (una función siempre
                // devuelve el mismo tipo de dato).
                let tipo = match valor {
                    Some(e) => {
                        self.revisar_expresion(e, Modo::Leer, linea)?;
                        self.tipo_de(e)
                    }
                    None => Some("nulo".to_string()),
                };
                if let Some(t) = tipo {
                    match &self.devuelve {
                        None => self.devuelve = Some((t, linea)),
                        Some((antes, linea_antes)) => {
                            if *antes != t {
                                return Err(error(
                                    linea,
                                    format!(
                                        "Oye, esta función ya devolvió un {} en la línea {}, pero aquí devuelve un {}. Una función siempre devuelve el mismo tipo de dato.",
                                        antes, linea_antes, t
                                    ),
                                ));
                            }
                        }
                    }
                }
                Ok(())
            }
            Declaracion::Romper { .. } => {
                if self.en_bucle == 0 {
                    return Err(error(
                        linea,
                        "Oye, no puedes usar 'romper' fuera de un bucle.".to_string(),
                    ));
                }
                Ok(())
            }
            Declaracion::Seguir { .. } => {
                if self.en_bucle == 0 {
                    return Err(error(
                        linea,
                        "Oye, no puedes usar 'seguir' fuera de un bucle.".to_string(),
                    ));
                }
                Ok(())
            }
            Declaracion::Segun { valor, brazos, .. } => {
                self.revisar_expresion(valor, Modo::Leer, linea)?;
                let foto = self.foto();
                let mut movidos_todos: Option<HashMap<String, (String, usize)>> = None;
                for brazo in brazos {
                    if let Some(p) = &brazo.patron {
                        self.revisar_expresion(p, Modo::Leer, linea)?;
                    }
                    let movidos = self.revisar_rama(&brazo.cuerpo)?;
                    self.restaurar_con_quiza(&foto);
                    movidos_todos = match movidos_todos {
                        None => Some(movidos),
                        Some(antes) => Some(
                            antes
                                .into_iter()
                                .filter(|(n, _)| movidos.contains_key(n))
                                .collect(),
                        ),
                    };
                }
                // Lo que se movió en TODOS los brazos, seguro se movió.
                if let Some(todos) = movidos_todos {
                    for (nombre, destino) in todos {
                        if let Some(info) = self.buscar_mut(&nombre) {
                            info.movido = Some(destino);
                            info.movido_quiza = false;
                        }
                    }
                }
                Ok(())
            }
            Declaracion::Importar { ruta, .. } => {
                self.cargar_firmas_importar(ruta, linea)
            }
            Declaracion::Usar { nombre, .. } => {
                match bibliotecas::firmas_biblioteca(nombre) {
                    Ok(firmas) => {
                        for (f, n) in firmas {
                            self.funciones.insert(f, n);
                        }
                        Ok(())
                    }
                    Err(e) => Err(error(linea, format!("Oye, {}", e))),
                }
            }
            Declaracion::Leer { nombre, .. } => {
                self.revisar_cambio(nombre, linea)?;
                Ok(())
            }
            Declaracion::Bloque { declaraciones: cuerpo, .. } => {
                // Un bloque solo siempre corre: lo que se mueve
                // dentro, seguro se movió (sin fusionar).
                self.ambitos.push(HashMap::new());
                let resultado = self.revisar_lista(cuerpo);
                self.ambitos.pop();
                resultado
            }
        }
    }

    // Lee un archivo `importar` y apunta los nombres de sus
    // funciones (para que llamarlas no sea un error).
    fn cargar_firmas_importar(
        &mut self,
        ruta: &str,
        linea: usize,
    ) -> Result<(), ErrorVerificacion> {
        let completa = if Path::new(ruta).is_absolute() {
            PathBuf::from(ruta)
        } else {
            self.base.join(ruta)
        };
        let canonica = match fs::canonicalize(&completa) {
            Ok(c) => c,
            Err(_) => {
                return Err(error(
                    linea,
                    format!("Oye, no encontré el archivo '{}' para importar.", ruta),
                ));
            }
        };
        if self.importados.contains(&canonica) {
            return Ok(());
        }
        self.importados.insert(canonica.clone());
        let fuente = match fs::read_to_string(&canonica) {
            Ok(f) => f,
            Err(_) => {
                return Err(error(
                    linea,
                    format!("Oye, no pude leer el archivo '{}' para importar.", ruta),
                ));
            }
        };
        let tokens = match Lexer::nuevo(&fuente).tokenizar() {
            Ok(t) => t,
            Err(e) => return Err(error(linea, format!("Oye, {}", e))),
        };
        let declaraciones = match Parser::nuevo(tokens).parsear() {
            Ok(d) => d,
            Err(e) => return Err(error(linea, format!("Oye, {}", e))),
        };
        for d in &declaraciones {
            if let Declaracion::Funcion { nombre: f, parametros, .. } = d {
                self.funciones.insert(f.clone(), parametros.len());
            }
        }
        Ok(())
    }
}

// ¿Es un 0 escrito a mano? Quita los paréntesis y el signo
// menos: `0`, `(0)` y `-0` son cero.
fn es_cero_literal(e: &Expresion) -> bool {
    numero_literal(e).map(|n| n == 0.0).unwrap_or(false)
}

// El número escrito a mano, si es que lo es: quita los
// paréntesis y aplica el signo menos.
fn numero_literal(e: &Expresion) -> Option<f64> {
    match e {
        Expresion::Numero(n) => Some(*n),
        Expresion::Agrupada(dentro) => numero_literal(dentro),
        Expresion::Unaria { operador, derecha } => match operador {
            OperadorUnario::Negativo => numero_literal(derecha).map(|n| -n),
            OperadorUnario::No => None,
        },
        _ => None,
    }
}

// Cuántos elementos tiene una lista escrita a mano (o nada,
// si no es una lista escrita a mano).
fn largo_de(e: &Expresion) -> Option<usize> {
    match e {
        Expresion::Lista(elementos) => Some(elementos.len()),
        Expresion::Agrupada(dentro) => largo_de(dentro),
        _ => None,
    }
}
