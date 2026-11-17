// ============================================================
//  ECLIPSE 1.2 - La librería de ejecución (runtime)
//  Archivo: rt/eclipse_rt.rs
//
//  Qué es esto:
//  Cuando Eclipse compila tu programa a un .exe, primero lo
//  traduce a Rust (un archivo main.rs) y luego llama a `rustc`.
//  Este archivo es la librería que ese programa usa: los
//  valores (números, textos, lógicos), el entorno (las
//  variables), las funciones que trae Eclipse y las reglas
//  de ownership. Es la misma lógica que el intérprete, pero
//  escrita para que el programa compilado la use.
//
//  No hay Garbage Collector: como en Rust, cada valor tiene
//  UN solo dueño y se libera cuando deja de usarse. No hay
//  un recolector corriendo en segundo plano ni pausas.
//
//  No edites este archivo a mano: Eclipse lo copia solo
//  cuando compila.
// ============================================================

use std::collections::HashMap;
use std::io::{self, Write};
use std::rc::Rc;
use std::time::{SystemTime, UNIX_EPOCH};

// Un valor de Eclipse: un número, un texto, un lógico, nulo o una lista.
#[derive(Debug, Clone, PartialEq)]
pub enum Valor {
    Numero(f64),
    Texto(String),
    Logico(bool),
    Nulo,
    // Una lista: varios valores en orden, como el `Vec` de Rust.
    // Vive dentro de un Rc: copiar una lista solo sube un
    // contador, no copia los elementos. Solo se copia de verdad
    // cuando alguien la cambia y hay más de un dueño (como
    // hace `agregar` con Rc::make_mut).
    Lista(Rc<Vec<Valor>>),
    // Un mapa: pares (clave, valor), como un diccionario.
    // Las claves son números, textos o verdadero/falso.
    // Igual que la lista, vive en un Rc (copiar solo sube
    // un contador; al cambiarlo se copia si hay más de un
    // dueño, con Rc::make_mut).
    Mapa(Rc<Vec<(Valor, Valor)>>),
}

impl Valor {
    // Crea una lista con sus elementos (los envuelve en Rc).
    pub fn lista(elementos: Vec<Valor>) -> Valor {
        Valor::Lista(Rc::new(elementos))
    }

    // Crea un mapa con sus pares (clave, valor). Si una clave
    // se repite, gana el último valor (y se queda en el
    // primer lugar donde apareció).
    pub fn mapa(pares: Vec<(Valor, Valor)>) -> Valor {
        let mut unicos: Vec<(Valor, Valor)> = Vec::new();
        for (clave, valor) in pares {
            match Self::buscar_clave(&unicos, &clave) {
                Some(pos) => unicos[pos].1 = valor,
                None => unicos.push((clave, valor)),
            }
        }
        Valor::Mapa(Rc::new(unicos))
    }

    // ¿En qué lugar está esta clave dentro del mapa?
    // (None si el mapa no la tiene.)
    pub fn buscar_clave(pares: &[(Valor, Valor)], clave: &Valor) -> Option<usize> {
        pares.iter().position(|(k, _)| k == clave)
    }

    // El tipo, en español: "numero", "texto", "logico", "nulo" o "lista".
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

    // Cómo se ve al imprimirlo. Los números enteros se ven
    // sin ".0" (10, no 10.0). Una lista se ve entre
    // corchetes: [1, 2, 3].
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

    // ¿Es verdadero? Solo falso, nulo, una lista vacía y un
    // mapa vacío son falsos.
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

// Cómo se ve la clave de un mapa al imprimirlo: los textos
// van entre comillas ("nombre"), los números y los lógicos
// se ven como son.
fn mostrar_clave(clave: &Valor) -> String {
    match clave {
        Valor::Texto(s) => format!("\"{}\"", s),
        otro => otro.a_texto(),
    }
}

// Una función tuya, compilada a Rust: recibe el entorno,
// sus datos, los nombres de los datos que se prestaron
// para cambiar (&mut) y los nombres de los datos que se
// prestaron solo para mirar (&), y devuelve un valor
// (o un error).
pub type FuncionEclipse =
    fn(&mut Entorno, Vec<Valor>, Vec<String>, Vec<String>) -> Result<Valor, String>;

// Lo que Eclipse guarda de una función tuya: su código y
// los nombres de sus datos (en orden).
struct FuncionInfo {
    codigo: FuncionEclipse,
    parametros: Vec<String>,
}

// Una variable: su valor, si se puede cambiar, si es una
// constante (nunca cambia ni se mueve), si está prestada a
// una función (no se puede mover mientras dure el préstamo)
// y a dónde se movió (si es que se movió).
struct Variable {
    valor: Valor,
    mutable: bool,
    constante: bool,
    prestado: bool,
    movido: Option<String>,
}

// Un préstamo para cambiar (&mut): la variable que se
// prestó, a qué dato de la función va (su posición) y qué
// tipo tenía.
struct Prestamo {
    fuente: String,
    param: usize,
    tipo_original: String,
}

// Los préstamos de una función que está corriendo: su
// nombre (para los errores) y la lista de préstamos.
struct MarcoPrestamo {
    funcion: String,
    prestamos: Vec<(String, String, String)>,
}

// El entorno: una pila de ámbitos con variables, las
// funciones que ya se definieron y los préstamos.
pub struct Entorno {
    ambitos: Vec<HashMap<String, Variable>>,
    funciones: HashMap<String, FuncionInfo>,
    // Préstamos anotados, esperando a la próxima llamada.
    prestamos_pendientes: Vec<Prestamo>,
    // Préstamos solo para mirar (&), esperando a la próxima
    // llamada: la posición del dato en la llamada.
    prestamos_lectura_pendientes: Vec<usize>,
    // Un marco de préstamos por cada función que está corriendo.
    marcos_prestamo: Vec<MarcoPrestamo>,
    // Cuántas funciones están corriendo una dentro de otra.
    profundidad: usize,
    semilla: u64,
}

impl Entorno {
    pub fn nuevo() -> Self {
        Entorno {
            ambitos: vec![HashMap::new()],
            funciones: HashMap::new(),
            prestamos_pendientes: Vec::new(),
            prestamos_lectura_pendientes: Vec::new(),
            marcos_prestamo: Vec::new(),
            profundidad: 0,
            semilla: semilla_inicial(),
        }
    }

    // Abre un ámbito nuevo (al entrar a un bloque { }).
    pub fn entrar(&mut self) {
        self.ambitos.push(HashMap::new());
    }

    // Cierra el ámbito actual (al salir del bloque).
    pub fn salir(&mut self) {
        self.ambitos.pop();
    }

    // Crea una variable en el ámbito actual.
    pub fn definir(&mut self, nombre: String, valor: Valor, mutable: bool) {
        if let Some(ambito) = self.ambitos.last_mut() {
            ambito.insert(
                nombre,
                Variable {
                    valor,
                    mutable,
                    constante: false,
                    prestado: false,
                    movido: None,
                },
            );
        }
    }

    // Crea una constante: nunca se puede cambiar ni se mueve.
    pub fn definir_constante(&mut self, nombre: String, valor: Valor) {
        if let Some(ambito) = self.ambitos.last_mut() {
            ambito.insert(
                nombre,
                Variable {
                    valor,
                    mutable: false,
                    constante: true,
                    prestado: false,
                    movido: None,
                },
            );
        }
    }

    // Crea un dato prestado (& o &mut): se puede mirar (y
    // cambiar, si es &mut), pero no se puede mover a otro
    // dueño mientras dure el préstamo.
    pub fn definir_prestado(&mut self, nombre: String, valor: Valor, mutable: bool) {
        if let Some(ambito) = self.ambitos.last_mut() {
            ambito.insert(
                nombre,
                Variable {
                    valor,
                    mutable,
                    constante: false,
                    prestado: true,
                    movido: None,
                },
            );
        }
    }

    // Cambia el valor de una variable que ya existe.
    // Asignarle un valor nuevo la revive: ya no está movida.
    // Una constante no se puede cambiar nunca.
    pub fn asignar(&mut self, nombre: &str, valor: Valor) -> Result<(), String> {
        for ambito in self.ambitos.iter_mut().rev() {
            if let Some(var) = ambito.get_mut(nombre) {
                if var.constante {
                    return Err(format!(
                        "la constante '{}' no se puede cambiar, nunca. Si quieres una variable que cambie, créala con 'sea mut'",
                        nombre
                    ));
                }
                if var.mutable {
                    var.valor = valor;
                    var.movido = None;
                    return Ok(());
                }
                return Err(format!(
                    "la variable '{}' es fija, no se puede cambiar. Si quieres cambiarla, créala con 'sea mut'",
                    nombre
                ));
            }
        }
        Err(format!("no conozco la variable '{}'", nombre))
    }

    // Mete un valor al final de una lista, en su sitio:
    // es el atajo de `x = agregar(x, v)`. Si la lista solo
    // la tiene esta variable, no se clona nada; si otro
    // también la tiene, se clona (la otra no cambia).
    pub fn agregar_en_sitio(&mut self, nombre: &str, valor: Valor) -> Result<(), String> {
        for ambito in self.ambitos.iter_mut().rev() {
            if let Some(var) = ambito.get_mut(nombre) {
                if let Some(destino) = &var.movido {
                    return Err(format!(
                        "la variable '{}' se movió {}, ya no puedes usarla. Para quedarte con una copia, usa copiar({})",
                        nombre, destino, nombre
                    ));
                }
                if !matches!(var.valor, Valor::Lista(_)) {
                    return Err(format!(
                        "agregar necesita una lista, no {}",
                        var.valor.tipo()
                    ));
                }
                if var.constante {
                    return Err(format!(
                        "la constante '{}' no se puede cambiar, nunca. Si quieres una variable que cambie, créala con 'sea mut'",
                        nombre
                    ));
                }
                if !var.mutable {
                    return Err(format!(
                        "la variable '{}' es fija, no se puede cambiar. Si quieres cambiarla, créala con 'sea mut'",
                        nombre
                    ));
                }
                if let Valor::Lista(elementos) = &mut var.valor {
                    Rc::make_mut(elementos).push(valor);
                }
                return Ok(());
            }
        }
        Err(format!("no conozco la variable '{}'", nombre))
    }

    // Cambia un lugar dentro de una lista o un mapa:
    // lista[0] = 5; o mapa[21] = "FTP";. La variable tiene
    // que ser `sea mut` (no fija ni constante) y no haberse
    // movido. Si la clave no existe en el mapa, la crea.
    // El orden de los problemas es el mismo que en la
    // máquina virtual: constante, fija, movida.
    pub fn asignar_indice(
        &mut self,
        nombre: &str,
        indice: Valor,
        valor: Valor,
    ) -> Result<(), String> {
        for ambito in self.ambitos.iter_mut().rev() {
            if let Some(var) = ambito.get_mut(nombre) {
                if var.constante {
                    return Err(format!(
                        "la constante '{}' no se puede cambiar, nunca. Si quieres una variable que cambie, créala con 'sea mut'",
                        nombre
                    ));
                }
                if !var.mutable {
                    return Err(format!(
                        "la variable '{}' es fija, no se puede cambiar. Si quieres cambiarla, créala con 'sea mut'",
                        nombre
                    ));
                }
                if let Some(destino) = &var.movido {
                    return Err(format!(
                        "la variable '{}' se movió {}, ya no puedes usarla. Para quedarte con una copia, usa copiar({})",
                        nombre, destino, nombre
                    ));
                }
                return cambiar_lugar(&mut var.valor, indice, valor);
            }
        }
        Err(format!("no conozco la variable '{}'", nombre))
    }

    // Busca el valor de una variable. Si se movió, te dice
    // a dónde se fue.
    pub fn obtener(&self, nombre: &str) -> Result<Valor, String> {
        for ambito in self.ambitos.iter().rev() {
            if let Some(var) = ambito.get(nombre) {
                if let Some(destino) = &var.movido {
                    return Err(format!(
                        "la variable '{}' se movió {}, ya no puedes usarla. Para quedarte con una copia, usa copiar({})",
                        nombre, destino, nombre
                    ));
                }
                return Ok(var.valor.clone());
            }
        }
        Err(format!("no conozco la variable '{}'", nombre))
    }

    // ¿Se puede cambiar esta variable? (¿se creó con `sea mut`?)
    pub fn es_mutable(&self, nombre: &str) -> Option<bool> {
        for ambito in self.ambitos.iter().rev() {
            if let Some(var) = ambito.get(nombre) {
                return Some(var.mutable);
            }
        }
        None
    }

    // Mueve una variable a otro dueño: si su valor es texto o
    // lista, la marca como movida y anota a dónde se fue (los
    // números y los lógicos son fotocopias, así que no se
    // mueven nunca). Las constantes nunca se mueven. Y algo
    // prestado no se puede mover mientras dure el préstamo.
    pub fn mover(&mut self, nombre: &str, destino: &str) -> Result<(), String> {
        for ambito in self.ambitos.iter_mut().rev() {
            if let Some(var) = ambito.get_mut(nombre) {
                if var.constante {
                    return Ok(());
                }
                if var.prestado {
                    return Err(format!(
                        "no se puede mover '{}' porque está prestado a una función (se lo pasaste con & o &mut) y el préstamo sigue activo",
                        nombre
                    ));
                }
                if matches!(var.valor, Valor::Texto(_) | Valor::Lista(_) | Valor::Mapa(_)) {
                    var.movido = Some(destino.to_string());
                }
                return Ok(());
            }
        }
        Ok(())
    }

    // Registra una función tuya, para poder llamarla a
    // partir de aquí (no antes de definirla).
    pub fn definir_funcion(
        &mut self,
        nombre: &str,
        codigo: FuncionEclipse,
        parametros: Vec<String>,
    ) {
        self.funciones.insert(
            nombre.to_string(),
            FuncionInfo {
                codigo,
                parametros,
            },
        );
    }

    // Input: lee una línea del teclado y la guarda en una
    // variable que ya existe. La variable debe ser `sea mut`
    // (si es fija o constante, da error). El texto se convierte
    // según lo que la variable ya es: número → parse, lógico
    // → "verdadero"/"falso", texto o lista → el texto tal cual.
    pub fn leer_entrada(&mut self, nombre: &str) -> Result<(), String> {
        let valor = self.obtener(nombre)?;
        if !self.es_mutable(nombre).unwrap_or(false) {
            return Err(format!(
                "no se puede leer en '{}' porque es fija. Si quieres que el usuario la escriba, créala con 'sea mut'",
                nombre
            ));
        }
        let mut linea = String::new();
        io::stdin()
            .read_line(&mut linea)
            .map_err(|_| "no pude leer del teclado".to_string())?;
        let linea = linea.trim().to_string();
        let nuevo = match valor {
            Valor::Numero(_) => match linea.parse::<f64>() {
                Ok(n) => Valor::Numero(n),
                Err(_) => {
                    return Err(format!("eso no era un número: '{}'", linea));
                }
            },
            Valor::Logico(_) => match linea.as_str() {
                "verdadero" => Valor::Logico(true),
                "falso" => Valor::Logico(false),
                _ => {
                    return Err(format!(
                        "eso no era un lógico: '{}'. Escribe 'verdadero' o 'falso'",
                        linea
                    ));
                }
            },
            // Una lista: separa lo escrito por comas (igual que
            // en el intérprete y la VM).
            Valor::Lista(items) => {
                let partes: Vec<&str> = linea.split(',').map(|p| p.trim()).collect();
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
            _ => Valor::Texto(linea),
        };
        self.asignar(nombre, nuevo)
    }

    // puntos++ o puntos--: suma o resta 1 a un número.
    // Solo sirve para números.
    pub fn incrementar(&mut self, nombre: &str, subir: bool) -> Result<(), String> {
        let valor = self.obtener(nombre)?;
        match valor {
            Valor::Numero(n) => {
                let nuevo = if subir { n + 1.0 } else { n - 1.0 };
                self.asignar(nombre, Valor::Numero(nuevo))
            }
            otro => Err(format!(
                "solo se puede usar '{}' con un numero, no con {}",
                if subir { "++" } else { "--" },
                otro.tipo()
            )),
        }
    }

    // Anota un préstamo para cambiar (&mut): revisa que la
    // variable se pueda cambiar y que sea un número, un
    // lógico o una lista. `param` es la posición del dato
    // en la llamada. La escritura de vuelta se hace al
    // terminar la función.
    pub fn anotar_prestamo(&mut self, fuente: &str, param: usize) -> Result<(), String> {
        let valor = self.obtener(fuente)?;
        match valor {
            Valor::Numero(_) | Valor::Logico(_) | Valor::Lista(_) => {}
            otro => {
                return Err(format!(
                    "solo se puede prestar para cambiar un número, un lógico o una lista, no {}",
                    otro.tipo()
                ));
            }
        }
        match self.es_mutable(fuente) {
            Some(true) => {}
            Some(false) => {
                return Err(format!(
                    "no se puede prestar para cambiar '{}' porque es fija. Si quieres que una función la cambie, créala con 'sea mut'",
                    fuente
                ));
            }
            None => {
                return Err(format!("no conozco la variable '{}'", fuente));
            }
        }
        self.prestamos_pendientes.push(Prestamo {
            fuente: fuente.to_string(),
            param,
            tipo_original: valor.tipo().to_string(),
        });
        Ok(())
    }

    // Anota un préstamo solo para mirar (&): no revisa nada
    // (mirar sirve para cualquier tipo, fijo o no) y solo
    // apunta qué dato de la próxima llamada es. La función
    // lo definirá como prestado, para que no se pueda mover
    // mientras corre.
    pub fn anotar_prestamo_lectura(&mut self, _fuente: &str, param: usize) {
        self.prestamos_lectura_pendientes.push(param);
    }

    // Al terminar una función: lee los valores finales de
    // los préstamos, cierra el ámbito de la función y los
    // escribe de vuelta a tus variables. Si la función
    // cambió el tipo, da error con el nombre de la función.
    pub fn terminar_prestamos(&mut self) -> Result<(), String> {
        let marco = match self.marcos_prestamo.pop() {
            Some(marco) => marco,
            None => return Ok(()),
        };
        let mut devoluciones: Vec<(String, Valor, String)> = Vec::new();
        for (fuente, param, tipo) in &marco.prestamos {
            if let Ok(valor) = self.obtener(param) {
                devoluciones.push((fuente.clone(), valor, tipo.clone()));
            }
        }
        // Cierra el ámbito de la función.
        self.salir();
        for (fuente, valor, tipo) in devoluciones {
            if valor.tipo() != tipo {
                return Err(format!(
                    "la función '{}' cambió el tipo de '{}': era {} y ahora es {}",
                    marco.funcion, fuente, tipo, valor.tipo()
                ));
            }
            self.asignar(&fuente, valor)?;
        }
        Ok(())
    }

    // Llama a una función: primero las que trae Eclipse,
    // y si no es una de esas, una que definiste tú.
    pub fn llamar(&mut self, nombre: &str, argumentos: Vec<Valor>) -> Result<Valor, String> {
        match nombre {
            "imprimir" => {
                let textos: Vec<String> = argumentos.iter().map(|v| v.a_texto()).collect();
                println!("{}", textos.join(" "));
                Ok(Valor::Nulo)
            }
            "leer" => {
                if !argumentos.is_empty() {
                    return Err("leer no necesita datos entre paréntesis".to_string());
                }
                let mut linea = String::new();
                io::stdin()
                    .read_line(&mut linea)
                    .map_err(|_| "no pude leer lo que escribiste".to_string())?;
                Ok(Valor::Texto(
                    linea.trim_end_matches(['\n', '\r']).to_string(),
                ))
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
                    Ok(n) => Ok(Valor::Numero(n)),
                    Err(_) => Err(format!("eso no era un número: '{}'", texto)),
                }
            }
            "a_numero" => {
                cantidad(nombre, &argumentos, 1)?;
                match &argumentos[0] {
                    Valor::Texto(s) => s
                        .trim()
                        .parse::<f64>()
                        .map(Valor::Numero)
                        .map_err(|_| format!("eso no era un número: '{}'", s)),
                    Valor::Numero(n) => Ok(Valor::Numero(*n)),
                    otro => Err(format!("a_numero necesita texto, no {}", otro.tipo())),
                }
            }
            "a_texto" => {
                cantidad(nombre, &argumentos, 1)?;
                Ok(Valor::Texto(argumentos[0].a_texto()))
            }
            "leer_archivo" => {
                cantidad(nombre, &argumentos, 1)?;
                let ruta = match &argumentos[0] {
                    Valor::Texto(s) => s.clone(),
                    _ => {
                        return Err(
                            "leer_archivo necesita el nombre del archivo entre comillas"
                                .to_string(),
                        )
                    }
                };
                std::fs::read_to_string(&ruta)
                    .map(Valor::Texto)
                    .map_err(|_| format!("no encontré el archivo '{}'", ruta))
            }
            "escribir_archivo" => {
                cantidad(nombre, &argumentos, 2)?;
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
                std::fs::write(&ruta, &contenido)
                    .map_err(|_| format!("no pude escribir el archivo '{}'", ruta))?;
                Ok(Valor::Nulo)
            }
            "largo" => {
                cantidad(nombre, &argumentos, 1)?;
                match &argumentos[0] {
                    Valor::Texto(s) => Ok(Valor::Numero(s.chars().count() as f64)),
                    Valor::Lista(elementos) => Ok(Valor::Numero(elementos.len() as f64)),
                    Valor::Mapa(pares) => Ok(Valor::Numero(pares.len() as f64)),
                    otro => Err(format!(
                        "largo necesita texto, lista o mapa, no {}",
                        otro.tipo()
                    )),
                }
            }
            "tipo_de" => {
                cantidad(nombre, &argumentos, 1)?;
                Ok(Valor::Texto(argumentos[0].tipo().to_string()))
            }
            "aleatorio" => {
                if !argumentos.is_empty() {
                    return Err("aleatorio no necesita datos entre paréntesis".to_string());
                }
                Ok(Valor::Numero(self.siguiente_aleatorio() as f64))
            }
            "copiar" => {
                cantidad(nombre, &argumentos, 1)?;
                Ok(argumentos[0].clone())
            }
            "min" => {
                cantidad(nombre, &argumentos, 2)?;
                match (&argumentos[0], &argumentos[1]) {
                    (Valor::Numero(a), Valor::Numero(b)) => {
                        Ok(Valor::Numero(if a < b { *a } else { *b }))
                    }
                    (a, b) => Err(format!(
                        "min necesita dos numeros, no {} y {}",
                        a.tipo(),
                        b.tipo()
                    )),
                }
            }
            "max" => {
                cantidad(nombre, &argumentos, 2)?;
                match (&argumentos[0], &argumentos[1]) {
                    (Valor::Numero(a), Valor::Numero(b)) => {
                        Ok(Valor::Numero(if a > b { *a } else { *b }))
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
                Ok(Valor::Numero(segundos))
            }
            // caracter(65) -> "A" (el carácter de ese número).
            "caracter" => {
                cantidad(nombre, &argumentos, 1)?;
                match &argumentos[0] {
                    Valor::Numero(n) => match char::from_u32(*n as u32) {
                        Some(c) => Ok(Valor::Texto(c.to_string())),
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
                Ok(Valor::Texto(sistema::sistema()))
            }
            // usuario() -> tu nombre de usuario del sistema.
            "usuario" => {
                if !argumentos.is_empty() {
                    return Err("usuario no necesita datos entre paréntesis".to_string());
                }
                Ok(Valor::Texto(sistema::usuario()))
            }
            // carpeta() -> la carpeta donde estás parado.
            "carpeta" => {
                if !argumentos.is_empty() {
                    return Err("carpeta no necesita datos entre paréntesis".to_string());
                }
                Ok(Valor::Texto(sistema::carpeta()?))
            }
            // variable_entorno("PATH") -> el valor de esa variable.
            "variable_entorno" => {
                cantidad(nombre, &argumentos, 1)?;
                match &argumentos[0] {
                    Valor::Texto(s) => Ok(Valor::Texto(sistema::variable_entorno(s)?)),
                    otro => Err(format!("variable_entorno necesita un texto, no {}", otro.tipo())),
                }
            }
            // comando("dir") -> lo que imprimió ese programa.
            "comando" => {
                cantidad(nombre, &argumentos, 1)?;
                match &argumentos[0] {
                    Valor::Texto(s) => Ok(Valor::Texto(sistema::comando(s)?)),
                    otro => Err(format!("comando necesita un texto, no {}", otro.tipo())),
                }
            }
            // argumentos() -> la lista de datos que le diste al programa.
            "argumentos" => {
                if !argumentos.is_empty() {
                    return Err("argumentos no necesita datos entre paréntesis".to_string());
                }
                Ok(Valor::lista(
                    sistema::argumentos()
                        .into_iter()
                        .map(Valor::Texto)
                        .collect(),
                ))
            }
            // procesadores() -> cuántos núcleos tiene la computadora.
            "procesadores" => {
                if !argumentos.is_empty() {
                    return Err("procesadores no necesita datos entre paréntesis".to_string());
                }
                Ok(Valor::Numero(sistema::procesadores()))
            }
            // memoria() -> cuánta memoria total tiene, en MB.
            "memoria" => {
                if !argumentos.is_empty() {
                    return Err("memoria no necesita datos entre paréntesis".to_string());
                }
                Ok(Valor::Numero(sistema::memoria_mb()?))
            }
            // redondear(3.7) -> 4 ; absoluto(-5) -> 5
            "redondear" => {
                cantidad(nombre, &argumentos, 1)?;
                match &argumentos[0] {
                    Valor::Numero(n) => Ok(Valor::Numero(n.round())),
                    otro => Err(format!(
                        "redondear necesita un numero, no {}",
                        otro.tipo()
                    )),
                }
            }
            "absoluto" => {
                cantidad(nombre, &argumentos, 1)?;
                match &argumentos[0] {
                    Valor::Numero(n) => Ok(Valor::Numero(n.abs())),
                    otro => Err(format!(
                        "absoluto necesita un numero, no {}",
                        otro.tipo()
                    )),
                }
            }
            // mayusculas("hola") -> "HOLA" ; minusculas("HOLA") -> "hola"
            "mayusculas" => {
                cantidad(nombre, &argumentos, 1)?;
                match &argumentos[0] {
                    Valor::Texto(s) => Ok(Valor::Texto(s.to_uppercase())),
                    otro => Err(format!(
                        "mayusculas necesita un texto, no {}",
                        otro.tipo()
                    )),
                }
            }
            "minusculas" => {
                cantidad(nombre, &argumentos, 1)?;
                match &argumentos[0] {
                    Valor::Texto(s) => Ok(Valor::Texto(s.to_lowercase())),
                    otro => Err(format!(
                        "minusculas necesita un texto, no {}",
                        otro.tipo()
                    )),
                }
            }
            // contiene("hola kevin", "kevin") -> verdadero
            "contiene" => {
                cantidad(nombre, &argumentos, 2)?;
                match (&argumentos[0], &argumentos[1]) {
                    (Valor::Texto(t), Valor::Texto(p)) => {
                        Ok(Valor::Logico(t.contains(p.as_str())))
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
                cantidad(nombre, &argumentos, 1)?;
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
                        Ok(Valor::lista(copia))
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
                cantidad(nombre, &argumentos, 1)?;
                match &argumentos[0] {
                    Valor::Lista(elementos) => {
                        let mut copia = (**elementos).clone();
                        copia.reverse();
                        Ok(Valor::lista(copia))
                    }
                    Valor::Texto(s) => {
                        Ok(Valor::Texto(s.chars().rev().collect()))
                    }
                    otro => Err(format!(
                        "invertir necesita una lista o un texto, no {}",
                        otro.tipo()
                    )),
                }
            }
            // unir(["a", "b"], ", ") es "a, b".
            "unir" => {
                cantidad(nombre, &argumentos, 2)?;
                match (&argumentos[0], &argumentos[1]) {
                    (Valor::Lista(elementos), Valor::Texto(s)) => {
                        let textos: Vec<String> =
                            elementos.iter().map(|v| v.a_texto()).collect();
                        Ok(Valor::Texto(textos.join(s.as_str())))
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
                cantidad(nombre, &argumentos, 2)?;
                match (&argumentos[0], &argumentos[1]) {
                    (Valor::Texto(t), Valor::Texto(s)) => {
                        if s.is_empty() {
                            return Err(
                                "el separador no puede estar vacío".to_string()
                            );
                        }
                        Ok(Valor::lista(
                            t.split(s.as_str())
                                .map(|p| Valor::Texto(p.to_string()))
                                .collect(),
                        ))
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
                cantidad(nombre, &argumentos, 1)?;
                match &argumentos[0] {
                    Valor::Numero(n) => {
                        if *n < 0.0 {
                            return Err(format!(
                                "no se puede calcular la raíz de {} porque es negativo",
                                n
                            ));
                        }
                        Ok(Valor::Numero(n.sqrt()))
                    }
                    otro => Err(format!(
                        "raiz necesita un numero, no {}",
                        otro.tipo()
                    )),
                }
            }
            // agregar(mi_lista, 3) devuelve una lista NUEVA con
            // el 3 al final. La lista vieja no cambia, así que
            // hay que guardar el resultado: mi_lista =
            // agregar(mi_lista, 3);
            "agregar" => {
                cantidad(nombre, &argumentos, 2)?;
                // La lista se mueve (no se clona): el intérprete
                // hace lo mismo, así que agregar es O(1) y no O(n).
                let mut it = argumentos.into_iter();
                let lista = it.next().unwrap();
                let valor = it.next().unwrap();
                match lista {
                    Valor::Lista(mut elementos) => {
                        Rc::make_mut(&mut elementos).push(valor);
                        Ok(Valor::Lista(elementos))
                    }
                    otro => Err(format!(
                        "agregar necesita una lista, no {}",
                        otro.tipo()
                    )),
                }
            }
            // elemento(mi_lista, 0) es lo mismo que mi_lista[0].
            "elemento" => {
                cantidad(nombre, &argumentos, 2)?;
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
                        Ok(elementos[i as usize].clone())
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
                            Some(pos) => Ok(pares[pos].1.clone()),
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
                cantidad(nombre, &argumentos, 1)?;
                match &argumentos[0] {
                    Valor::Mapa(pares) => Ok(Valor::lista(
                        pares.iter().map(|(k, _)| k.clone()).collect(),
                    )),
                    otro => Err(format!("claves necesita un mapa, no {}", otro.tipo())),
                }
            }
            // valores({21: "FTP", 22: "SSH"}) es ["FTP", "SSH"].
            "valores" => {
                cantidad(nombre, &argumentos, 1)?;
                match &argumentos[0] {
                    Valor::Mapa(pares) => Ok(Valor::lista(
                        pares.iter().map(|(_, v)| v.clone()).collect(),
                    )),
                    otro => Err(format!("valores necesita un mapa, no {}", otro.tipo())),
                }
            }
            // tiene(mapa, 21) dice si el mapa tiene esa clave.
            "tiene" => {
                cantidad(nombre, &argumentos, 2)?;
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
                        Ok(Valor::Logico(
                            Valor::buscar_clave(pares, clave).is_some(),
                        ))
                    }
                    otro => Err(format!("tiene necesita un mapa, no {}", otro.tipo())),
                }
            }
            // fecha() es "2026-11-17", la fecha de hoy (UTC:
            // Rust solo no sabe tu zona horaria).
            "fecha" => {
                if !argumentos.is_empty() {
                    return Err("fecha no necesita datos entre paréntesis".to_string());
                }
                let (ano, mes, dia, _, _, _) = fecha_hora_partes();
                Ok(Valor::Texto(format!("{:04}-{:02}-{:02}", ano, mes, dia)))
            }
            // hora() es "14:32:05", la hora de ahora (UTC).
            "hora" => {
                if !argumentos.is_empty() {
                    return Err("hora no necesita datos entre paréntesis".to_string());
                }
                let (_, _, _, h, m, s) = fecha_hora_partes();
                Ok(Valor::Texto(format!("{:02}:{:02}:{:02}", h, m, s)))
            }
            // fecha_hora() es "2026-11-17 14:32:05" (UTC).
            "fecha_hora" => {
                if !argumentos.is_empty() {
                    return Err("fecha_hora no necesita datos entre paréntesis".to_string());
                }
                let (ano, mes, dia, h, m, s) = fecha_hora_partes();
                Ok(Valor::Texto(format!(
                    "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
                    ano, mes, dia, h, m, s
                )))
            }
            // dormir(2) espera 2 segundos sin hacer nada.
            // Sirve para automatizar: esperar a que algo pase.
            "dormir" => {
                cantidad(nombre, &argumentos, 1)?;
                match &argumentos[0] {
                    Valor::Numero(n) => {
                        if !n.is_finite() || *n < 0.0 {
                            return Err(
                                "dormir necesita un número de segundos, no puede ser negativo"
                                    .to_string(),
                            );
                        }
                        std::thread::sleep(std::time::Duration::from_secs_f64(*n));
                        Ok(Valor::Nulo)
                    }
                    otro => Err(format!("dormir necesita un numero, no {}", otro.tipo())),
                }
            }
            // existe("notas.txt") dice si ese archivo o esa
            // carpeta está ahí (verdadero o falso).
            "existe" => {
                cantidad(nombre, &argumentos, 1)?;
                match &argumentos[0] {
                    Valor::Texto(ruta) => {
                        Ok(Valor::Logico(std::path::Path::new(ruta).exists()))
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
                cantidad(nombre, &argumentos, 1)?;
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
                        Ok(Valor::lista(
                            nombres.into_iter().map(Valor::Texto).collect(),
                        ))
                    }
                    Err(_) => Err(format!("no encontré la carpeta '{}'", ruta)),
                }
            }
            // crear_carpeta("respaldo") crea esa carpeta (y las
            // que falten en el camino, si hace falta).
            "crear_carpeta" => {
                cantidad(nombre, &argumentos, 1)?;
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
                    Ok(()) => Ok(Valor::Nulo),
                    Err(_) => Err(format!("no pude crear la carpeta '{}'", ruta)),
                }
            }
            // mover_archivo("viejo.txt", "nuevo.txt") cambia el
            // nombre o mueve el archivo a otra carpeta.
            "mover_archivo" => {
                cantidad(nombre, &argumentos, 2)?;
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
                    Ok(()) => Ok(Valor::Nulo),
                    Err(_) => Err(format!("no pude mover '{}' a '{}'", viejo, nuevo)),
                }
            }
            // borrar_archivo("viejo.txt") borra ese archivo.
            "borrar_archivo" => {
                cantidad(nombre, &argumentos, 1)?;
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
                    Ok(()) => Ok(Valor::Nulo),
                    Err(_) => Err(format!("no pude borrar el archivo '{}'", ruta)),
                }
            }
            // reemplazar("hola mundo", "mundo", "Kevin") es
            // "hola Kevin": cambia todas las veces que aparece.
            "reemplazar" => {
                cantidad(nombre, &argumentos, 3)?;
                match (&argumentos[0], &argumentos[1], &argumentos[2]) {
                    (Valor::Texto(t), Valor::Texto(viejo), Valor::Texto(nuevo)) => {
                        if viejo.is_empty() {
                            return Err(
                                "el texto a reemplazar no puede estar vacío".to_string()
                            );
                        }
                        Ok(Valor::Texto(t.replace(viejo.as_str(), nuevo)))
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
                // Una función tuya: tiene que estar definida
                // (no se puede llamar antes de definirla).
                let info = self
                    .funciones
                    .get(nombre)
                    .ok_or_else(|| format!("no conozco la función '{}'", nombre))?;
                if info.parametros.len() != argumentos.len() {
                    return Err(format!(
                        "la función '{}' necesita {} {}, pero le diste {}",
                        nombre,
                        info.parametros.len(),
                        cuantos(info.parametros.len()),
                        argumentos.len()
                    ));
                }
                // Qué datos se prestaron para cambiar (&mut):
                // sus nombres, para que la función los defina
                // como cambiables y pueda escribir en ellos.
                let prestamos = std::mem::take(&mut self.prestamos_pendientes);
                let mut marco: Vec<(String, String, String)> = Vec::new();
                let mut mutables: Vec<String> = Vec::new();
                for p in prestamos {
                    if let Some(nombre_param) = info.parametros.get(p.param) {
                        mutables.push(nombre_param.clone());
                        marco.push((
                            p.fuente,
                            nombre_param.clone(),
                            p.tipo_original,
                        ));
                    }
                }
                // Qué datos se prestaron solo para mirar (&):
                // sus nombres, para que la función los defina
                // como prestados (y no se puedan mover).
                let prestamos_lectura =
                    std::mem::take(&mut self.prestamos_lectura_pendientes);
                let mut prestados: Vec<String> = Vec::new();
                for param in prestamos_lectura {
                    if let Some(nombre_param) = info.parametros.get(param) {
                        prestados.push(nombre_param.clone());
                    }
                }
                self.marcos_prestamo.push(MarcoPrestamo {
                    funcion: nombre.to_string(),
                    prestamos: marco,
                });
                // Estabilidad: igual que el intérprete y la VM,
                // paramos en 1000 funciones anidadas con un error
                // en español, en vez de romper la pila de Rust.
                if self.profundidad >= 1000 {
                    return Err(
                        "demasiadas funciones anidadas, ¿tienes una función que se llama a sí misma sin parar?"
                            .to_string(),
                    );
                }
                self.profundidad += 1;
                let codigo = info.codigo;
                let resultado = codigo(self, argumentos, mutables, prestados);
                self.profundidad -= 1;
                resultado
            }
        }
    }

    // Un número al azar entre 0 y 99.
    fn siguiente_aleatorio(&mut self) -> u64 {
        let mut x = self.semilla;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.semilla = x;
        x % 100
    }
}

// La semilla inicial para `aleatorio()`: la hora actual en
// nanosegundos (nunca 0, porque el generador se quedaría en 0).
fn semilla_inicial() -> u64 {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0x9E3779B97F4A7C15);
    if nanos == 0 {
        0x9E3779B97F4A7C15
    } else {
        nanos
    }
}

// "1 dato" o "2 datos", en español.
pub fn cuantos(n: usize) -> String {
    if n == 1 {
        "dato".to_string()
    } else {
        "datos".to_string()
    }
}

// Revisa que una función reciba la cantidad de datos que pide.
// Lo que puede pasar cuando corre un `intentar`: todo bien,
// salió con un `devolver`, salió con `romper` o salió con
// `seguir`. El .exe lo usa para decidir qué hacer después
// del `al_final`.
#[derive(Debug, Clone)]
pub enum SalidaIntento {
    Bien,
    Salio(Valor),
    Rompio,
    Siguio,
}

// Cambia un lugar dentro de una lista o un mapa. En una
// lista el lugar es un número (si no existe, error); en un
// mapa la clave es un número, un texto o verdadero/falso
// (si no existe, la crea). Es la misma lógica que la
// máquina virtual, con los mismos mensajes.
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
            Rc::make_mut(elementos)[i as usize] = valor;
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
            let lista = Rc::make_mut(pares);
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

// La fecha y la hora de ahora, en UTC (Rust solo no sabe
// tu zona horaria, así que la hora sale en tiempo
// universal). Devuelve (año, mes, día, hora, minuto,
// segundo). El cálculo de días a fecha civil es el
// algoritmo de Howard Hinnant.
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

fn cantidad(nombre: &str, argumentos: &[Valor], esperado: usize) -> Result<(), String> {
    if argumentos.len() != esperado {
        return Err(format!(
            "'{}' necesita {} {}, pero le diste {}",
            nombre,
            esperado,
            cuantos(esperado),
            argumentos.len()
        ));
    }
    Ok(())
}

// Una operación con dos valores ya calculados.
// `y` y `o` no pasan por aquí porque miran el segundo
// valor solo si hace falta.
pub fn aplicar_binario(izq: Valor, op: &str, der: Valor) -> Result<Valor, String> {
    match op {
        "==" => Ok(Valor::Logico(izq == der)),
        "!=" => Ok(Valor::Logico(izq != der)),
        "+" => match (&izq, &der) {
            (Valor::Numero(a), Valor::Numero(b)) => Ok(Valor::Numero(a + b)),
            // Pegar dos listas: [1, 2] + [3] es [1, 2, 3].
            (Valor::Lista(a), Valor::Lista(b)) => {
                let mut resultado = (**a).clone();
                resultado.extend(b.iter().cloned());
                Ok(Valor::lista(resultado))
            }
            _ => Ok(Valor::Texto(format!(
                "{}{}",
                izq.a_texto(),
                der.a_texto()
            ))),
        },
        "-" => numeros(izq, der, |a, b| a - b, "-"),
        "*" => numeros(izq, der, |a, b| a * b, "*"),
        "/" => {
            if matches!(der, Valor::Numero(n) if n == 0.0) {
                return Err("No se puede dividir entre cero".to_string());
            }
            numeros(izq, der, |a, b| a / b, "/")
        }
        "%" => {
            if matches!(der, Valor::Numero(n) if n == 0.0) {
                return Err("No se puede calcular el resto entre cero".to_string());
            }
            numeros(izq, der, |a, b| a % b, "%")
        }
        "**" => numeros(izq, der, |a, b| a.powf(b), "**"),
        "<" => comparar(izq, der, "<", |a, b| a < b),
        "<=" => comparar(izq, der, "<=", |a, b| a <= b),
        ">" => comparar(izq, der, ">", |a, b| a > b),
        ">=" => comparar(izq, der, ">=", |a, b| a >= b),
        _ => Err(format!("no conozco el operador '{}'", op)),
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

// Cambiar el signo de un número (-5). Solo sirve para números.
pub fn negar(valor: Valor) -> Result<Valor, String> {
    match valor {
        Valor::Numero(n) => Ok(Valor::Numero(-n)),
        _ => Err("solo se puede poner '-' delante de un numero".to_string()),
    }
}

// Negar un valor (no verdadero es falso).
pub fn no_logico(valor: Valor) -> Valor {
    Valor::Logico(!valor.es_verdadero())
}

// Descarga la salida estándar (para que `imprimir` no se
// quede guardado sin salir).
#[allow(dead_code)]
pub fn vaciar_salida() {
    let _ = io::stdout().flush();
}

// ---- Hablar con el hardware y el software (1.0) ----
// Las mismas funciones que src/motor/sistema.rs, copiadas
// aquí porque el programa compilado no ve ese archivo.
mod sistema {
    use std::env;
    use std::fs;
    use std::process::Command;

    pub fn sistema() -> String {
        match env::consts::OS {
            "windows" => "Windows".to_string(),
            "linux" => "Linux".to_string(),
            "macos" => "macOS".to_string(),
            otro => otro.to_string(),
        }
    }

    pub fn usuario() -> String {
        env::var("USER")
            .or_else(|_| env::var("USERNAME"))
            .unwrap_or_else(|_| "?".to_string())
    }

    pub fn carpeta() -> Result<String, String> {
        env::current_dir()
            .map(|p| p.to_string_lossy().to_string())
            .map_err(|_| "no pude saber en qué carpeta estoy".to_string())
    }

    pub fn variable_entorno(nombre: &str) -> Result<String, String> {
        env::var(nombre).map_err(|_| format!("no existe la variable de entorno '{}'", nombre))
    }

    pub fn comando(texto: &str) -> Result<String, String> {
        let salida = if cfg!(windows) {
            Command::new("cmd").args(["/C", texto]).output()
        } else {
            Command::new("sh").args(["-c", texto]).output()
        };
        let salida = salida.map_err(|_| format!("no pude correr el comando '{}'", texto))?;
        if !salida.status.success() {
            let codigo = salida.status.code().unwrap_or(-1);
            let error = String::from_utf8_lossy(&salida.stderr);
            let error = error.trim();
            if error.is_empty() {
                return Err(format!("el comando terminó con error (código {})", codigo));
            }
            return Err(format!("el comando terminó con error (código {}): {}", codigo, error));
        }
        Ok(String::from_utf8_lossy(&salida.stdout).trim_end().to_string())
    }

    pub fn argumentos() -> Vec<String> {
        env::args().collect()
    }

    pub fn procesadores() -> f64 {
        std::thread::available_parallelism()
            .map(|n| n.get() as f64)
            .unwrap_or(1.0)
    }

    pub fn memoria_mb() -> Result<f64, String> {
        if cfg!(target_os = "linux") {
            let texto = fs::read_to_string("/proc/meminfo")
                .map_err(|_| "no pude leer la memoria en este sistema".to_string())?;
            for linea in texto.lines() {
                if let Some(resto) = linea.strip_prefix("MemTotal:") {
                    let numero: String = resto.chars().filter(|c| c.is_ascii_digit()).collect();
                    if let Ok(kb) = numero.parse::<f64>() {
                        return Ok(kb / 1024.0);
                    }
                }
            }
            return Err("no encontré la memoria total en /proc/meminfo".to_string());
        }
        if cfg!(windows) {
            let salida = Command::new("wmic")
                .args(["computersystem", "get", "TotalPhysicalMemory", "/value"])
                .output()
                .map_err(|_| "no pude leer la memoria en este sistema".to_string())?;
            let texto = String::from_utf8_lossy(&salida.stdout);
            for linea in texto.lines() {
                if let Some(resto) = linea.trim().strip_prefix("TotalPhysicalMemory=") {
                    if let Ok(bytes) = resto.trim().parse::<f64>() {
                        return Ok(bytes / 1024.0 / 1024.0);
                    }
                }
            }
            return Err("no encontré la memoria total (wmic)".to_string());
        }
        if cfg!(target_os = "macos") {
            let salida = Command::new("sysctl")
                .args(["-n", "hw.memsize"])
                .output()
                .map_err(|_| "no pude leer la memoria en este sistema".to_string())?;
            let texto = String::from_utf8_lossy(&salida.stdout);
            if let Ok(bytes) = texto.trim().parse::<f64>() {
                return Ok(bytes / 1024.0 / 1024.0);
            }
            return Err("no encontré la memoria total (sysctl)".to_string());
        }
        Err("no puedo leer la memoria total en este sistema".to_string())
    }
}
