// ============================================================
//  ECLIPSE 1.2 - El parser
//  Archivo: src/parser.rs
//
//  Qué hace:
//  El parser es el segundo trabajador de la fábrica.
//  Toma los tokens del lexer y revisa si están en el orden
//  correcto, como armar una frase que tenga sentido.
//  Con ellos construye el arbolito (AST).
//
//  Cómo decide el orden (la precedencia, de menos a más
//  fuerte, como en matemáticas):
//      o  ->  y  ->  igualdad (==, !=)  ->  comparación (<, >)
//      ->  suma (+, -)  ->  producto (*, /, %)  ->  unaria (-, no)
//      ->  llamada y valores
//
//  Si algo está mal ordenado, el parser te dice en qué
//  línea está el problema, en español.
// ============================================================

use crate::lenguaje::ast::{BrazoSegun, Declaracion, Expresion, OperadorBinario, OperadorUnario, ParteCadena};
use crate::lenguaje::lexer::{Lexer, TipoToken, Token};

pub struct Parser {
    tokens: Vec<Token>,
    actual: usize,
    // Cuántos paréntesis y signos unarios llevamos anidados:
    // si son demasiados, el programa es demasiado profundo
    // y lo paramos con un error en vez de romper la pila.
    anidado: usize,
    // ¿Ya se escribió algo que no sea `usar`? Las librerías
    // (`usar`) solo valen arriba del archivo, antes de
    // cualquier otra línea.
    ya_empezo: bool,
    // ¿Cuántos bloques `{ ... }` llevamos anidados? Dentro de
    // un bloque tampoco vale `usar`.
    en_bloque: usize,
}

impl Parser {
    pub fn nuevo(tokens: Vec<Token>) -> Self {
        Parser {
            tokens,
            actual: 0,
            anidado: 0,
            ya_empezo: false,
            en_bloque: 0,
        }
    }

    // Lee todo el programa y devuelve la lista de órdenes (el arbolito).
    pub fn parsear(&mut self) -> Result<Vec<Declaracion>, String> {
        let mut declaraciones = Vec::new();
        while !self.al_final() {
            let d = self.declaracion()?;
            if !matches!(d, Declaracion::Usar { .. }) {
                self.ya_empezo = true;
            }
            declaraciones.push(d);
        }
        Ok(declaraciones)
    }

    // ---------- Ayudantes ----------

    fn mirar(&self) -> &Token {
        &self.tokens[self.actual]
    }

    fn al_final(&self) -> bool {
        matches!(self.mirar().tipo, TipoToken::Fin)
    }

    // Avanza un token y devuelve el que acabamos de pasar.
    fn avanzar(&mut self) -> Token {
        let t = self.tokens[self.actual].clone();
        if !self.al_final() {
            self.actual += 1;
        }
        t
    }

    // ¿El token actual es de este tipo? (sin avanzar)
    fn es(&self, tipo: &TipoToken) -> bool {
        std::mem::discriminant(&self.mirar().tipo) == std::mem::discriminant(tipo)
    }

    // Si el token actual es uno de estos, lo come y dice que sí.
    fn coincide(&mut self, tipos: &[TipoToken]) -> bool {
        for tipo in tipos {
            if self.es(tipo) {
                self.avanzar();
                return true;
            }
        }
        false
    }

    // Come el token que esperamos, o da un error en español.
    fn consumir(&mut self, tipo: TipoToken, mensaje: &str) -> Result<Token, String> {
        if self.es(&tipo) {
            Ok(self.avanzar())
        } else {
            Err(self.error(mensaje))
        }
    }

    // Come un nombre (identificador), o da un error en español.
    // Las palabras nuevas (lista, para_cada, constante, leer)
    // también se pueden usar como nombres de variable: son
    // palabras "contextuales". Así un programa viejo que tenga
    // `sea lista = 5;` sigue corriendo igual en la 0.6.
    fn consumir_identificador(&mut self, mensaje: &str) -> Result<String, String> {
        let t = self.avanzar();
        match t.tipo {
            TipoToken::Identificador(nombre) => Ok(nombre),
            TipoToken::Lista => Ok("lista".to_string()),
            TipoToken::ParaCada => Ok("para_cada".to_string()),
            TipoToken::Constante => Ok("constante".to_string()),
            TipoToken::Leer => Ok("leer".to_string()),
            TipoToken::Mapa => Ok("mapa".to_string()),
            TipoToken::Intentar => Ok("intentar".to_string()),
            TipoToken::SiFalla => Ok("si_falla".to_string()),
            TipoToken::AlFinal => Ok("al_final".to_string()),
            _ => Err(self.error_en(t.linea, mensaje)),
        }
    }

    fn error(&self, mensaje: &str) -> String {
        format!("Error en la línea {}: {}", self.mirar().linea, mensaje)
    }

    fn error_en(&self, linea: usize, mensaje: &str) -> String {
        format!("Error en la línea {}: {}", linea, mensaje)
    }

    // ¿El token que sigue es un `=` o un `+=` (y familia)?
    // Sirve para saber si esto es una asignación:
    // nombre = valor;   nombre += valor;
    fn siguiente_es_asignacion(&self) -> bool {
        match self.tokens.get(self.actual + 1) {
            Some(t) => matches!(
                t.tipo,
                TipoToken::Igual
                    | TipoToken::MasIgual
                    | TipoToken::MenosIgual
                    | TipoToken::PorIgual
                    | TipoToken::EntreIgual
                    | TipoToken::ResiduoIgual
                    | TipoToken::PotenciaIgual
            ),
            None => false,
        }
    }

    // ¿El token que sigue es `++` o `--`?
    // Sirve para saber si esto es un incremento:
    // puntos++;   puntos--;
    fn siguiente_es_incremento(&self) -> bool {
        match self.tokens.get(self.actual + 1) {
            Some(t) => matches!(t.tipo, TipoToken::MasMas | TipoToken::MenosMenos),
            None => false,
        }
    }

    // ---------- Declaraciones (órdenes) ----------

    fn declaracion(&mut self) -> Result<Declaracion, String> {
        // La línea donde empieza esta orden (para
        // que el verificador diga dónde está el error).
        let linea = self.mirar().linea;
        match self.mirar().tipo.clone() {
            TipoToken::Sea => self.declaracion_sea(),
            TipoToken::Si => self.declaracion_si(),
            TipoToken::Mientras => self.declaracion_mientras(),
            TipoToken::Repetir => self.declaracion_repetir(),
            TipoToken::Para => self.declaracion_para(),
            TipoToken::Funcion => self.declaracion_funcion(),
            TipoToken::Devolver => self.declaracion_devolver(),
            TipoToken::Romper => {
                self.avanzar();
                self.consumir(TipoToken::PuntoYComa, "después de 'romper' va ';'")?;
                Ok(Declaracion::Romper { linea })
            }
            TipoToken::Seguir => {
                self.avanzar();
                self.consumir(TipoToken::PuntoYComa, "después de 'seguir' va ';'")?;
                Ok(Declaracion::Seguir { linea })
            }
            TipoToken::Usar => self.declaracion_usar(),
            TipoToken::Intentar => self.declaracion_intentar(),
            TipoToken::Segun => self.declaracion_segun(),
            TipoToken::Constante => self.declaracion_constante(),
            TipoToken::ParaCada => self.declaracion_para_cada(),
            TipoToken::Leer => {
                // `leer nombre;` es la orden Input (guarda lo que
                // escribe el usuario en la variable). Pero `leer()`
                // sigue siendo la función de siempre: si después
                // de `leer` viene un `(`, es una expresión.
                // (El nombre también puede ser una palabra nueva
                // como `lista`, por compatibilidad.)
                let es_input = matches!(
                    self.tokens.get(self.actual + 1).map(|t| &t.tipo),
                    Some(TipoToken::Identificador(_))
                        | Some(TipoToken::Lista)
                        | Some(TipoToken::ParaCada)
                        | Some(TipoToken::Constante)
                        | Some(TipoToken::Leer)
                        | Some(TipoToken::Mapa)
                        | Some(TipoToken::Intentar)
                        | Some(TipoToken::SiFalla)
                        | Some(TipoToken::AlFinal)
                        | Some(TipoToken::Cadena(_))
                );
                if es_input {
                    self.declaracion_leer()
                } else {
                    let e = self.expresion()?;
                    self.consumir(TipoToken::PuntoYComa, "después de la expresión va ';'")?;
                    Ok(Declaracion::Expresion { linea, expresion: e })
                }
            }
            TipoToken::Importar => {
                self.avanzar();
                let t = self.avanzar();
                match t.tipo {
                    TipoToken::Cadena(ruta) => {
                        self.consumir(
                            TipoToken::PuntoYComa,
                            "después de 'importar \"archivo.ec\"' va ';'",
                        )?;
                        Ok(Declaracion::Importar { linea, ruta })
                    }
                    _ => Err(self.error_en(
                        t.linea,
                        "después de 'importar' va el nombre del archivo entre comillas",
                    )),
                }
            }
            TipoToken::LlaveIzq => Ok(Declaracion::Bloque { linea, declaraciones: self.bloque()? }),
            // Un nombre al inicio: puede ser un incremento
            // (puntos++), una asignación (x = 5) o una
            // expresión sola (imprimir("Hola");). `lista` se
            // trata igual, por si es el nombre de tu variable.
            TipoToken::Identificador(_) | TipoToken::Lista => {
                self.declaracion_nombre()
            }
            _ => {
                let e = self.expresion()?;
                self.consumir(TipoToken::PuntoYComa, "después de la expresión va ';'")?;
                Ok(Declaracion::Expresion { linea, expresion: e })
            }
        }
    }

    // Una orden que empieza con un nombre: incremento,
    // asignación o expresión sola.
    fn declaracion_nombre(&mut self) -> Result<Declaracion, String> {
        // La línea donde empieza esta orden (para
        // que el verificador diga dónde está el error).
        let linea = self.mirar().linea;
        if self.siguiente_es_incremento() {
            // Incremento:  puntos++;  puntos--;
            let nombre = self.consumir_identificador("aquí va el nombre de la variable")?;
            let subir = matches!(self.avanzar().tipo, TipoToken::MasMas);
            self.consumir(TipoToken::PuntoYComa, "después de '++' o '--' va ';'")?;
            return Ok(Declaracion::Incrementar { linea, nombre, subir });
        }
        if self.siguiente_es_asignacion() {
            // Asignación:  nombre = valor;  nombre += valor;
            let nombre = self.consumir_identificador("aquí va el nombre de la variable")?;
            let operador = match self.avanzar().tipo {
                TipoToken::Igual => None,
                TipoToken::MasIgual => Some(OperadorBinario::Sumar),
                TipoToken::MenosIgual => Some(OperadorBinario::Restar),
                TipoToken::PorIgual => Some(OperadorBinario::Multiplicar),
                TipoToken::EntreIgual => Some(OperadorBinario::Dividir),
                TipoToken::ResiduoIgual => Some(OperadorBinario::Residuo),
                TipoToken::PotenciaIgual => Some(OperadorBinario::Potencia),
                _ => None,
            };
            let valor = self.expresion()?;
            self.consumir(TipoToken::PuntoYComa, "después de la asignación va ';'")?;
            match operador {
                None => Ok(Declaracion::Asignacion { linea, nombre, valor }),
                Some(operador) => Ok(Declaracion::AsignacionCompuesta {
                    linea,
                    nombre,
                    operador,
                    valor,
                }),
            }
        } else {
            let e = self.expresion()?;
            // Asignación a un lugar: lista[0] = 5;  mapa[21] = "FTP";
            // (solo con `=`: para += y familia, calcula el valor
            // nuevo y guárdalo con `=`, como siempre.)
            if matches!(e, Expresion::Indice { .. }) && self.es(&TipoToken::Igual) {
                let (objetivo, indice) = match e {
                    Expresion::Indice { lista, indice } => (*lista, *indice),
                    _ => unreachable!(),
                };
                self.avanzar(); // =
                let valor = self.expresion()?;
                self.consumir(
                    TipoToken::PuntoYComa,
                    "después de la asignación va ';'",
                )?;
                return Ok(Declaracion::AsignarIndice {
                    linea,
                    objetivo,
                    indice,
                    valor,
                });
            }
            if matches!(e, Expresion::Indice { .. })
                && matches!(
                    self.mirar().tipo,
                    TipoToken::MasIgual
                        | TipoToken::MenosIgual
                        | TipoToken::PorIgual
                        | TipoToken::EntreIgual
                        | TipoToken::ResiduoIgual
                        | TipoToken::PotenciaIgual
                )
            {
                return Err(self.error(
                    "para cambiar un lugar de una lista o un mapa usa '=' con el valor nuevo, por ejemplo: lista[0] = 5;",
                ));
            }
            self.consumir(TipoToken::PuntoYComa, "después de la expresión va ';'")?;
            Ok(Declaracion::Expresion { linea, expresion: e })
        }
    }

    // usar matematicas;   usar juegos;
    // Carga una librería instalada. Solo vale arriba del
    // archivo, antes de cualquier otra línea: si ya empezó
    // el programa (o estamos dentro de un bloque), da error.
    fn declaracion_usar(&mut self) -> Result<Declaracion, String> {
        // La línea donde empieza esta orden (para
        // que el verificador diga dónde está el error).
        let linea = self.mirar().linea;
        if self.ya_empezo || self.en_bloque > 0 {
            return Err(self.error_en(
                linea,
                "'usar' solo vale arriba del archivo, antes de cualquier otra línea. Si quieres usar una librería, ponla al principio.",
            ));
        }
        self.avanzar(); // usar
        let nombre = match self.avanzar().tipo {
            TipoToken::Identificador(n) => n,
            TipoToken::Cadena(n) => n,
            _ => {
                return Err(self.error_en(
                    self.mirar().linea,
                    "después de 'usar' va el nombre de la librería, por ejemplo: usar matematicas;",
                ))
            }
        };
        self.consumir(TipoToken::PuntoYComa, "después de 'usar nombre' va ';'")?;
        Ok(Declaracion::Usar { linea, nombre })
    }

    // intentar { ... } si_falla e { ... } al_final { ... }
    // Corre el cuerpo; si algo falla, corre si_falla (con el
    // error en `e`, si le pones nombre) en vez de parar el
    // programa; al_final corre siempre. Al menos uno de los
    // dos tiene que estar.
    fn declaracion_intentar(&mut self) -> Result<Declaracion, String> {
        // La línea donde empieza esta orden (para
        // que el verificador diga dónde está el error).
        let linea = self.mirar().linea;
        self.avanzar(); // intentar
        let cuerpo = self.bloque()?;
        let mut falla = None;
        if self.coincide(&[TipoToken::SiFalla]) {
            // El nombre del error es opcional: si_falla { ... }
            // o si_falla e { ... } (e es el mensaje del error).
            let nombre_error = match self.mirar().tipo.clone() {
                TipoToken::Identificador(n) => {
                    self.avanzar();
                    Some(n)
                }
                TipoToken::Mapa => {
                    self.avanzar();
                    Some("mapa".to_string())
                }
                TipoToken::Intentar => {
                    self.avanzar();
                    Some("intentar".to_string())
                }
                TipoToken::SiFalla => {
                    self.avanzar();
                    Some("si_falla".to_string())
                }
                TipoToken::AlFinal => {
                    self.avanzar();
                    Some("al_final".to_string())
                }
                _ => None,
            };
            let cuerpo_falla = self.bloque()?;
            falla = Some((nombre_error, cuerpo_falla));
        }
        let mut al_final = None;
        if self.coincide(&[TipoToken::AlFinal]) {
            al_final = Some(self.bloque()?);
        }
        if falla.is_none() && al_final.is_none() {
            return Err(self.error_en(
                linea,
                "un 'intentar' necesita 'si_falla' o 'al_final': si no, no sirve para nada",
            ));
        }
        Ok(Declaracion::Intentar {
            linea,
            cuerpo,
            falla,
            al_final,
        })
    }

    // sea x = 5;   sea mut x: numero = 5;   sea x;
    fn declaracion_sea(&mut self) -> Result<Declaracion, String> {
        // La línea donde empieza esta orden (para
        // que el verificador diga dónde está el error).
        let linea = self.mirar().linea;
        self.avanzar(); // sea
        let mutable = self.coincide(&[TipoToken::Mut]);
        let nombre =
            self.consumir_identificador("después de 'sea' va el nombre de la variable")?;
        let mut tipo = None;
        if self.coincide(&[TipoToken::DosPuntos]) {
            let t = self.avanzar();
            tipo = Some(match t.tipo {
                TipoToken::Numero => "numero".to_string(),
                TipoToken::Texto => "texto".to_string(),
                TipoToken::Logico => "logico".to_string(),
                TipoToken::Lista => "lista".to_string(),
                TipoToken::Mapa => "mapa".to_string(),
                _ => {
                    return Err(self.error_en(
                        t.linea,
                        "el tipo debe ser numero, texto, logico, lista o mapa",
                    ))
                }
            });
        }
        let mut valor = None;
        if self.coincide(&[TipoToken::Igual]) {
            valor = Some(self.expresion()?);
        }
        // Sin valor, la variable vale nulo. Si le pusiste tipo,
        // el tipo se revisa cuando le des un valor.
        // (Antes pedía valor obligatorio con tipo; ahora no.)
        self.consumir(
            TipoToken::PuntoYComa,
            "después de crear la variable va ';'",
        )?;
        Ok(Declaracion::Sea {
            linea,
            nombre,
            mutable,
            tipo,
            valor,
        })
    }

    // constante PI = 3.14;   constante NOMBRE: texto = "Kevin";
    // Como `sea`, pero el valor nunca se puede cambiar ni se
    // mueve: siempre lo puedes usar. El valor es obligatorio.
    fn declaracion_constante(&mut self) -> Result<Declaracion, String> {
        // La línea donde empieza esta orden (para
        // que el verificador diga dónde está el error).
        let linea = self.mirar().linea;
        self.avanzar(); // constante
        let nombre =
            self.consumir_identificador("después de 'constante' va el nombre de la constante")?;
        let mut tipo = None;
        if self.coincide(&[TipoToken::DosPuntos]) {
            let t = self.avanzar();
            tipo = Some(match t.tipo {
                TipoToken::Numero => "numero".to_string(),
                TipoToken::Texto => "texto".to_string(),
                TipoToken::Logico => "logico".to_string(),
                TipoToken::Lista => "lista".to_string(),
                TipoToken::Mapa => "mapa".to_string(),
                _ => {
                    return Err(self.error_en(
                        t.linea,
                        "el tipo debe ser numero, texto, logico, lista o mapa",
                    ))
                }
            });
        }
        self.consumir(
            TipoToken::Igual,
            "después del nombre de la constante va '=' con su valor",
        )?;
        let valor = self.expresion()?;
        self.consumir(
            TipoToken::PuntoYComa,
            "después de crear la constante va ';'",
        )?;
        Ok(Declaracion::Constante {
            linea,
            nombre,
            tipo,
            valor,
        })
    }

    // para_cada numero en mi_lista { ... }
    // Recorre una lista, un elemento por vuelta.
    fn declaracion_para_cada(&mut self) -> Result<Declaracion, String> {
        // La línea donde empieza esta orden (para
        // que el verificador diga dónde está el error).
        let linea = self.mirar().linea;
        self.avanzar(); // para_cada
        let variable =
            self.consumir_identificador("después de 'para_cada' va el nombre de la variable")?;
        self.consumir(TipoToken::En, "después de la variable va 'en'")?;
        let lista = self.expresion()?;
        let cuerpo = self.bloque()?;
        Ok(Declaracion::ParaCada {
            linea,
            variable,
            lista,
            cuerpo,
        })
    }

    // leer nombre;   (Input)
    // Lee lo que escribe el usuario y lo guarda en la variable.
    // Con mensaje: leer "¿Cómo te llamas?" nombre; muestra el
    // mensaje antes de leer, sin salto de línea.
    fn declaracion_leer(&mut self) -> Result<Declaracion, String> {
        // La línea donde empieza esta orden (para
        // que el verificador diga dónde está el error).
        let linea = self.mirar().linea;
        self.avanzar(); // leer
        let mensaje = match self.mirar().tipo.clone() {
            TipoToken::Cadena(m) => {
                self.avanzar();
                Some(m)
            }
            _ => None,
        };
        let nombre =
            self.consumir_identificador("después de 'leer' va el nombre de la variable")?;
        self.consumir(
            TipoToken::PuntoYComa,
            "después de 'leer nombre' va ';'",
        )?;
        Ok(Declaracion::Leer { linea, nombre, mensaje })
    }

    // si condicion { ... } sino { ... }   (el sino es opcional,
    // y puede ser otro `si`:  sino si ... )
    fn declaracion_si(&mut self) -> Result<Declaracion, String> {
        // La línea donde empieza esta orden (para
        // que el verificador diga dónde está el error).
        let linea = self.mirar().linea;
        self.avanzar(); // si
        let condicion = self.expresion()?;
        let entonces = self.bloque()?;
        let mut sino = None;
        if self.coincide(&[TipoToken::Sino]) {
            if self.es(&TipoToken::Si) {
                let anidado = self.declaracion_si()?;
                sino = Some(vec![anidado]);
            } else {
                sino = Some(self.bloque()?);
            }
        }
        Ok(Declaracion::Si {
            linea,
            condicion,
            entonces,
            sino,
        })
    }

    // segun valor { 10 => { ... } "hola" => { ... } _ => { ... } }
    // Cada brazo es un patrón y un bloque. El patrón puede ser
    // un número, un texto, verdadero/falso, o `_` que significa
    // "cualquier otro valor". Corre el primer brazo que coincida.
    fn declaracion_segun(&mut self) -> Result<Declaracion, String> {
        // La línea donde empieza esta orden (para
        // que el verificador diga dónde está el error).
        let linea = self.mirar().linea;
        self.avanzar(); // segun
        let valor = self.expresion()?;
        self.consumir(TipoToken::LlaveIzq, "después del valor va '{'")?;
        let mut brazos = Vec::new();
        while !self.es(&TipoToken::LlaveDer) && !self.al_final() {
            // Una coma entre brazos es opcional: se puede poner o no.
            if self.coincide(&[TipoToken::Coma]) {
                continue;
            }
            let patron = if matches!(self.mirar().tipo, TipoToken::Identificador(ref n) if n == "_")
            {
                self.avanzar(); // _
                None
            } else {
                let t = self.avanzar();
                let patron = match t.tipo {
                    TipoToken::NumeroLit(n) => Expresion::Numero(n),
                    TipoToken::Cadena(s) => Expresion::Cadena(s),
                    TipoToken::Verdadero => Expresion::Logico(true),
                    TipoToken::Falso => Expresion::Logico(false),
                    TipoToken::Nulo => Expresion::Nulo,
                    _ => {
                        return Err(self.error_en(
                            t.linea,
                            "en un segun solo puedes poner números, textos, verdadero/falso, nulo o _",
                        ))
                    }
                };
                Some(patron)
            };
            self.consumir(TipoToken::Flecha, "después del patrón va '=>'")?;
            let cuerpo = self.bloque()?;
            brazos.push(BrazoSegun { patron, cuerpo });
        }
        self.consumir(TipoToken::LlaveDer, "aquí va '}' para cerrar el segun")?;
        Ok(Declaracion::Segun { linea, valor, brazos })
    }

    // mientras condicion { ... }
    fn declaracion_mientras(&mut self) -> Result<Declaracion, String> {
        // La línea donde empieza esta orden (para
        // que el verificador diga dónde está el error).
        let linea = self.mirar().linea;
        self.avanzar(); // mientras
        let condicion = self.expresion()?;
        let cuerpo = self.bloque()?;
        Ok(Declaracion::Mientras { linea, condicion, cuerpo })
    }

    // repetir { ... } mientras condicion;
    // Corre el cuerpo una vez, y luego repite mientras la
    // condición sea verdadera.
    fn declaracion_repetir(&mut self) -> Result<Declaracion, String> {
        // La línea donde empieza esta orden (para
        // que el verificador diga dónde está el error).
        let linea = self.mirar().linea;
        self.avanzar(); // repetir
        let cuerpo = self.bloque()?;
        self.consumir(
            TipoToken::Mientras,
            "después del cuerpo de 'repetir' va 'mientras' con la condición",
        )?;
        let condicion = self.expresion()?;
        self.consumir(
            TipoToken::PuntoYComa,
            "después de la condición de 'repetir' va ';'",
        )?;
        Ok(Declaracion::Repetir { linea, cuerpo, condicion })
    }

    // para i en 1..5 { ... }   para i en 1..=5 { ... }
    // para i en 1..10 paso 2 { ... }
    fn declaracion_para(&mut self) -> Result<Declaracion, String> {
        // La línea donde empieza esta orden (para
        // que el verificador diga dónde está el error).
        let linea = self.mirar().linea;
        self.avanzar(); // para
        let variable =
            self.consumir_identificador("después de 'para' va el nombre de la variable")?;
        self.consumir(TipoToken::En, "después de la variable va 'en'")?;
        let inicio = self.expresion()?;
        let inclusivo = if self.coincide(&[TipoToken::PuntoPuntoIgual]) {
            true
        } else if self.coincide(&[TipoToken::PuntoPunto]) {
            false
        } else {
            return Err(self.error("después del inicio va '..' o '..=' para el rango"));
        };
        let fin = self.expresion()?;
        let mut paso = None;
        if self.coincide(&[TipoToken::Paso]) {
            paso = Some(self.expresion()?);
        }
        let cuerpo = self.bloque()?;
        Ok(Declaracion::Para {
            linea,
            variable,
            inicio,
            fin,
            inclusivo,
            paso,
            cuerpo,
        })
    }

    // funcion sumar(a, b) { devolver a + b; }
    fn declaracion_funcion(&mut self) -> Result<Declaracion, String> {
        // La línea donde empieza esta orden (para
        // que el verificador diga dónde está el error).
        let linea = self.mirar().linea;
        self.avanzar(); // funcion
        let nombre =
            self.consumir_identificador("después de 'funcion' va el nombre de la función")?;
        self.consumir(TipoToken::ParenIzq, "después del nombre va '('")?;
        let mut parametros = Vec::new();
        if !self.es(&TipoToken::ParenDer) {
            loop {
                parametros.push(
                    self.consumir_identificador("aquí va el nombre de un parámetro")?,
                );
                if !self.coincide(&[TipoToken::Coma]) {
                    break;
                }
                // Una coma al final está permitida: funcion f(a, b,) {.
                if self.es(&TipoToken::ParenDer) {
                    break;
                }
            }
        }
        self.consumir(TipoToken::ParenDer, "después de los parámetros va ')'")?;
        let cuerpo = self.bloque()?;
        Ok(Declaracion::Funcion {
            linea,
            nombre,
            parametros,
            cuerpo,
        })
    }

    // devolver 42;   devolver;
    fn declaracion_devolver(&mut self) -> Result<Declaracion, String> {
        // La línea donde empieza esta orden (para
        // que el verificador diga dónde está el error).
        let linea = self.mirar().linea;
        self.avanzar(); // devolver
        let valor = if self.es(&TipoToken::PuntoYComa) {
            None
        } else {
            Some(self.expresion()?)
        };
        self.consumir(TipoToken::PuntoYComa, "después de 'devolver' va ';'")?;
        Ok(Declaracion::Devolver { linea, valor })
    }

    // Un bloque: { ... } con órdenes adentro.
    fn bloque(&mut self) -> Result<Vec<Declaracion>, String> {
        self.consumir(TipoToken::LlaveIzq, "aquí va '{' para abrir el bloque")?;
        self.en_bloque += 1;
        let mut declaraciones = Vec::new();
        while !self.es(&TipoToken::LlaveDer) && !self.al_final() {
            declaraciones.push(self.declaracion()?);
        }
        let cierre = self.consumir(TipoToken::LlaveDer, "aquí va '}' para cerrar el bloque");
        self.en_bloque -= 1;
        cierre?;
        Ok(declaraciones)
    }

    // ---------- Expresiones (con precedencia) ----------

    fn expresion(&mut self) -> Result<Expresion, String> {
        self.o()
    }

    // o  (lo menos fuerte)
    fn o(&mut self) -> Result<Expresion, String> {
        let mut expr = self.y()?;
        while self.es(&TipoToken::O) {
            self.avanzar();
            let derecha = self.y()?;
            expr = Expresion::Binaria {
                izquierda: Box::new(expr),
                operador: OperadorBinario::O,
                derecha: Box::new(derecha),
            };
        }
        Ok(expr)
    }

    // y
    fn y(&mut self) -> Result<Expresion, String> {
        let mut expr = self.igualdad()?;
        while self.es(&TipoToken::Y) {
            self.avanzar();
            let derecha = self.igualdad()?;
            expr = Expresion::Binaria {
                izquierda: Box::new(expr),
                operador: OperadorBinario::Y,
                derecha: Box::new(derecha),
            };
        }
        Ok(expr)
    }

    // ==  !=
    fn igualdad(&mut self) -> Result<Expresion, String> {
        let mut expr = self.comparacion()?;
        loop {
            let operador = if self.es(&TipoToken::IgualIgual) {
                OperadorBinario::Igual
            } else if self.es(&TipoToken::Diferente) {
                OperadorBinario::Diferente
            } else {
                break;
            };
            self.avanzar();
            let derecha = self.comparacion()?;
            expr = Expresion::Binaria {
                izquierda: Box::new(expr),
                operador,
                derecha: Box::new(derecha),
            };
        }
        Ok(expr)
    }

    // <  <=  >  >=
    fn comparacion(&mut self) -> Result<Expresion, String> {
        let mut expr = self.suma()?;
        loop {
            let operador = if self.es(&TipoToken::Menor) {
                OperadorBinario::Menor
            } else if self.es(&TipoToken::MenorIgual) {
                OperadorBinario::MenorIgual
            } else if self.es(&TipoToken::Mayor) {
                OperadorBinario::Mayor
            } else if self.es(&TipoToken::MayorIgual) {
                OperadorBinario::MayorIgual
            } else {
                break;
            };
            self.avanzar();
            let derecha = self.suma()?;
            expr = Expresion::Binaria {
                izquierda: Box::new(expr),
                operador,
                derecha: Box::new(derecha),
            };
        }
        Ok(expr)
    }

    // +  -
    fn suma(&mut self) -> Result<Expresion, String> {
        let mut expr = self.producto()?;
        loop {
            let operador = if self.es(&TipoToken::Mas) {
                OperadorBinario::Sumar
            } else if self.es(&TipoToken::Menos) {
                OperadorBinario::Restar
            } else {
                break;
            };
            self.avanzar();
            let derecha = self.producto()?;
            expr = Expresion::Binaria {
                izquierda: Box::new(expr),
                operador,
                derecha: Box::new(derecha),
            };
        }
        Ok(expr)
    }

    // *  /  %
    fn producto(&mut self) -> Result<Expresion, String> {
        let mut expr = self.potencia()?;
        loop {
            let operador = if self.es(&TipoToken::Por) {
                OperadorBinario::Multiplicar
            } else if self.es(&TipoToken::Entre) {
                OperadorBinario::Dividir
            } else if self.es(&TipoToken::Residuo) {
                OperadorBinario::Residuo
            } else {
                break;
            };
            self.avanzar();
            let derecha = self.potencia()?;
            expr = Expresion::Binaria {
                izquierda: Box::new(expr),
                operador,
                derecha: Box::new(derecha),
            };
        }
        Ok(expr)
    }

    // **  (la potencia, 2 ** 3 es 8)
    // Se asocia a la derecha: 2 ** 3 ** 2 es 2 ** (3 ** 2).
    fn potencia(&mut self) -> Result<Expresion, String> {
        let izquierda = self.unaria()?;
        if self.es(&TipoToken::Potencia) {
            self.avanzar();
            let derecha = self.potencia()?;
            return Ok(Expresion::Binaria {
                izquierda: Box::new(izquierda),
                operador: OperadorBinario::Potencia,
                derecha: Box::new(derecha),
            });
        }
        Ok(izquierda)
    }

    // -5   no verdadero
    fn unaria(&mut self) -> Result<Expresion, String> {
        if self.es(&TipoToken::Menos) {
            self.anidado += 1;
            if self.anidado > 100 {
                return Err(self.error("demasiados signos juntos, el programa es demasiado profundo"));
            }
            self.avanzar();
            let derecha = self.unaria()?;
            self.anidado -= 1;
            return Ok(Expresion::Unaria {
                operador: OperadorUnario::Negativo,
                derecha: Box::new(derecha),
            });
        }
        if self.es(&TipoToken::No) {
            self.anidado += 1;
            if self.anidado > 100 {
                return Err(self.error("demasiados signos juntos, el programa es demasiado profundo"));
            }
            self.avanzar();
            let derecha = self.unaria()?;
            self.anidado -= 1;
            return Ok(Expresion::Unaria {
                operador: OperadorUnario::No,
                derecha: Box::new(derecha),
            });
        }
        self.llamada()
    }

    // Una llamada:  sumar(1, 2)   imprimir("Hola")   saludar(&nombre)
    // Y un índice:  mi_lista[0]   (el elemento en esa posición)
    fn llamada(&mut self) -> Result<Expresion, String> {
        let mut expr = self.primaria()?;
        loop {
            if self.es(&TipoToken::ParenIzq) {
                self.avanzar();
                let mut argumentos = Vec::new();
                if !self.es(&TipoToken::ParenDer) {
                    loop {
                        if self.coincide(&[TipoToken::Ampersand]) {
                            // &nombre: prestar, no mover.
                            // &mut nombre: prestar para cambiar.
                            let mutable = self.coincide(&[TipoToken::Mut]);
                            let t = self.avanzar();
                            let nombre = match t.tipo {
                                TipoToken::Identificador(n) => n,
                                _ => {
                                    return Err(self.error_en(
                                        t.linea,
                                        "después de & va el nombre de una variable",
                                    ))
                                }
                            };
                            if mutable {
                                argumentos.push(Expresion::PrestarMut(Box::new(
                                    Expresion::Variable(nombre),
                                )));
                            } else {
                                argumentos.push(Expresion::Prestar(Box::new(
                                    Expresion::Variable(nombre),
                                )));
                            }
                        } else {
                            argumentos.push(self.expresion()?);
                        }
                        if !self.coincide(&[TipoToken::Coma]) {
                            break;
                        }
                        // Una coma al final está permitida: f(1, 2,).
                        if self.es(&TipoToken::ParenDer) {
                            break;
                        }
                    }
                }
                self.consumir(TipoToken::ParenDer, "después de los argumentos va ')'")?;
                let nombre = match expr {
                    Expresion::Variable(n) => n,
                    _ => {
                        return Err(self.error(
                            "solo se puede llamar a una función por su nombre",
                        ))
                    }
                };
                expr = Expresion::Llamada { nombre, argumentos };
            } else if self.es(&TipoToken::CorcheteIzq) {
                // Índice: mi_lista[0], o [1, 2, 3][0].
                self.avanzar(); // [
                let indice = self.expresion()?;
                self.consumir(TipoToken::CorcheteDer, "después del índice va ']'")?;
                expr = Expresion::Indice {
                    lista: Box::new(expr),
                    indice: Box::new(indice),
                };
            } else {
                break;
            }
        }
        Ok(expr)
    }

    // Los valores más simples: números, textos, nombres, ( ... )
    fn primaria(&mut self) -> Result<Expresion, String> {        let t = self.avanzar();
        match t.tipo {
            TipoToken::NumeroLit(n) => Ok(Expresion::Numero(n)),
            TipoToken::Cadena(s) => {
                // Si el texto tiene {expresiones}, se interpola:
                // "Hola, {nombre}" pega el valor de nombre.
                // {{ es una llave { literal, y }} es una }.
                if s.contains('{') || s.contains('}') {
                    let partes = interpolar(&s, t.linea)?;
                    let hay_expresion = partes
                        .iter()
                        .any(|p| matches!(p, ParteCadena::Expresion(_)));
                    if hay_expresion {
                        Ok(Expresion::Interpolada(partes))
                    } else {
                        let mut texto = String::new();
                        for p in partes {
                            if let ParteCadena::Texto(t) = p {
                                texto.push_str(&t);
                            }
                        }
                        Ok(Expresion::Cadena(texto))
                    }
                } else {
                    Ok(Expresion::Cadena(s))
                }
            }
            TipoToken::Verdadero => Ok(Expresion::Logico(true)),
            TipoToken::Falso => Ok(Expresion::Logico(false)),
            TipoToken::Nulo => Ok(Expresion::Nulo),
            TipoToken::Identificador(nombre) => Ok(Expresion::Variable(nombre)),
            // imprimir y leer son palabras clave, pero también
            // se pueden llamar como funciones: imprimir("Hola")
            TipoToken::Imprimir => Ok(Expresion::Variable("imprimir".to_string())),
            TipoToken::Leer => Ok(Expresion::Variable("leer".to_string())),
            // Las palabras nuevas también se pueden usar como
            // nombres de variable, por compatibilidad con los
            // programas viejos: sea lista = 5; imprimir(lista);
            TipoToken::Lista => Ok(Expresion::Variable("lista".to_string())),
            TipoToken::ParaCada => Ok(Expresion::Variable("para_cada".to_string())),
            TipoToken::Constante => Ok(Expresion::Variable("constante".to_string())),
            TipoToken::Mapa => Ok(Expresion::Variable("mapa".to_string())),
            TipoToken::Intentar => Ok(Expresion::Variable("intentar".to_string())),
            TipoToken::SiFalla => Ok(Expresion::Variable("si_falla".to_string())),
            TipoToken::AlFinal => Ok(Expresion::Variable("al_final".to_string())),
            // Un mapa: {21: "FTP", 22: "SSH"}  o  {}  (vacío).
            // La clave va antes de los dos puntos y el valor
            // después. Una coma al final está permitida.
            TipoToken::LlaveIzq => {
                let mut pares = Vec::new();
                if !self.es(&TipoToken::LlaveDer) {
                    loop {
                        let clave = self.expresion()?;
                        self.consumir(
                            TipoToken::DosPuntos,
                            "en un mapa, después de la clave va ':' con su valor",
                        )?;
                        let valor = self.expresion()?;
                        pares.push((clave, valor));
                        if !self.coincide(&[TipoToken::Coma]) {
                            break;
                        }
                        // Una coma al final está permitida.
                        if self.es(&TipoToken::LlaveDer) {
                            break;
                        }
                    }
                }
                self.consumir(TipoToken::LlaveDer, "después del mapa va '}'")?;
                Ok(Expresion::Mapa(pares))
            }
            // Una lista: [1, 2, 3]  o  []  (vacía).
            TipoToken::CorcheteIzq => {
                let mut elementos = Vec::new();
                if !self.es(&TipoToken::CorcheteDer) {
                    loop {
                        elementos.push(self.expresion()?);
                        if !self.coincide(&[TipoToken::Coma]) {
                            break;
                        }
                        // Una coma al final está permitida: [1, 2, 3,].
                        if self.es(&TipoToken::CorcheteDer) {
                            break;
                        }
                    }
                }
                self.consumir(TipoToken::CorcheteDer, "después de la lista va ']'")?;
                Ok(Expresion::Lista(elementos))
            }
            TipoToken::ParenIzq => {
                self.anidado += 1;
                if self.anidado > 100 {
                    return Err(self.error_en(
                        t.linea,
                        "demasiados paréntesis anidados, el programa es demasiado profundo",
                    ));
                }
                let e = self.expresion()?;
                self.consumir(TipoToken::ParenDer, "después de la expresión va ')'")?;
                self.anidado -= 1;
                Ok(Expresion::Agrupada(Box::new(e)))
            }
            _ => {
                // Si es una palabra clave, decirlo claro: las palabras
                // clave no se pueden usar como valor o como nombre.
                if let Some(palabra) = nombre_palabra(&t.tipo) {
                    return Err(self.error_en(
                        t.linea,
                        &format!(
                            "la palabra '{}' es una palabra clave de Eclipse, no se puede usar como valor",
                            palabra
                        ),
                    ));
                }
                Err(self.error_en(
                    t.linea,
                    "esperaba un valor, un nombre o '('",
                ))
            }
        }
    }
}

// La palabra en español de cada palabra clave, para poder
// decirte en el error cuál palabra usaste mal.
fn nombre_palabra(tipo: &TipoToken) -> Option<&'static str> {
    let palabra = match tipo {
        TipoToken::Sea => "sea",
        TipoToken::Mut => "mut",
        TipoToken::Si => "si",
        TipoToken::Sino => "sino",
        TipoToken::Mientras => "mientras",
        TipoToken::Para => "para",
        TipoToken::En => "en",
        TipoToken::Funcion => "funcion",
        TipoToken::Devolver => "devolver",
        TipoToken::Imprimir => "imprimir",
        TipoToken::Leer => "leer",
        TipoToken::Verdadero => "verdadero",
        TipoToken::Falso => "falso",
        TipoToken::Y => "y",
        TipoToken::O => "o",
        TipoToken::No => "no",
        TipoToken::Romper => "romper",
        TipoToken::Seguir => "seguir",
        TipoToken::Nulo => "nulo",
        TipoToken::Importar => "importar",
        TipoToken::Numero => "numero",
        TipoToken::Texto => "texto",
        TipoToken::Logico => "logico",
        TipoToken::Segun => "segun",
        TipoToken::Paso => "paso",
        TipoToken::Repetir => "repetir",
        TipoToken::Lista => "lista",
        TipoToken::ParaCada => "para_cada",
        TipoToken::Constante => "constante",
        TipoToken::Usar => "usar",
        TipoToken::Mapa => "mapa",
        TipoToken::Intentar => "intentar",
        TipoToken::SiFalla => "si_falla",
        TipoToken::AlFinal => "al_final",
        _ => return None,
    };
    Some(palabra)
}

// Parsea una sola expresión desde un texto, para la
// interpolación: {nombre + 1} dentro de un texto.
pub fn parsear_expresion(fuente: &str) -> Result<Expresion, String> {
    let tokens = Lexer::nuevo(fuente).tokenizar()?;
    let mut parser = Parser::nuevo(tokens);
    let expr = parser.expresion()?;
    if !parser.al_final() {
        return Err("no entendí lo que hay dentro de { }".to_string());
    }
    Ok(expr)
}

// Corta un texto en partes: trozos fijos y {expresiones}.
// `{{` es una llave `{` literal, y `}}` es una `}` literal.
fn interpolar(texto: &str, linea: usize) -> Result<Vec<ParteCadena>, String> {
    let mut partes = Vec::new();
    let mut actual = String::new();
    let chars: Vec<char> = texto.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c == '{' {
            if i + 1 < chars.len() && chars[i + 1] == '{' {
                actual.push('{');
                i += 2;
                continue;
            }
            // Buscar la } que cierra, contando llaves anidadas
            // y saltando los textos entre comillas.
            let mut profundidad = 1;
            let mut j = i + 1;
            let mut dentro_texto = false;
            while j < chars.len() && profundidad > 0 {
                let d = chars[j];
                if dentro_texto {
                    if d == '\\' {
                        j += 2;
                        continue;
                    }
                    if d == '"' {
                        dentro_texto = false;
                    }
                    j += 1;
                    continue;
                }
                match d {
                    '"' => dentro_texto = true,
                    '{' => profundidad += 1,
                    '}' => profundidad -= 1,
                    _ => {}
                }
                j += 1;
            }
            if profundidad > 0 {
                return Err(format!(
                    "Error en la línea {}: en el texto falta }} para cerrar {{",
                    linea
                ));
            }
            let interior: String = chars[i + 1..j - 1].iter().collect();
            // `{}` vacío (o solo espacios) es texto literal, no
            // una expresión: "Vacío: {}" se imprime tal cual.
            if interior.trim().is_empty() {
                actual.push('{');
                actual.push_str(&interior);
                actual.push('}');
                i = j;
                continue;
            }
            let expr = parsear_expresion(&interior)
                .map_err(|e| format!("Error en la línea {}: {}", linea, e))?;
            if !actual.is_empty() {
                partes.push(ParteCadena::Texto(std::mem::take(&mut actual)));
            }
            partes.push(ParteCadena::Expresion(Box::new(expr)));
            i = j;
            continue;
        }
        if c == '}' && i + 1 < chars.len() && chars[i + 1] == '}' {
            actual.push('}');
            i += 2;
            continue;
        }
        actual.push(c);
        i += 1;
    }
    if !actual.is_empty() {
        partes.push(ParteCadena::Texto(actual));
    }
    Ok(partes)
}
