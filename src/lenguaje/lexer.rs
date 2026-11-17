// ============================================================
//  ECLIPSE 1.2 - El lexer
//  Archivo: src/lexer.rs
//
//  Qué hace:
//  El lexer es el primer trabajador de la fábrica.
//  Toma el texto del archivo .ec y lo corta en pedacitos
//  llamados tokens: palabras clave, nombres, números,
//  textos y símbolos.
//
//  Ejemplo:
//      sea nombre = "Kevin";
//  se convierte en:
//      [sea] [nombre] [=] ["Kevin"] [;]
//
//  Si el texto tiene algo raro, el lexer te dice en qué
//  línea está el problema, en español.
// ============================================================

#[derive(Debug, Clone, PartialEq)]
pub enum TipoToken {
    // ---- Las 34 palabras clave de Eclipse ----
    Sea, Mut, Si, Sino, Mientras, Para, En, Funcion, Devolver,
    Imprimir, Leer, Verdadero, Falso, Y, O, No, Romper, Seguir,
    Nulo, Importar, Numero, Texto, Logico, Segun, Paso, Repetir,
    Lista, ParaCada, Constante, Usar, Mapa, Intentar, SiFalla, AlFinal,
    // ---- Otros pedacitos ----
    Identificador(String),
    NumeroLit(f64),
    Cadena(String),
    // ---- Los 9 símbolos ----
    Mas, Menos, Por, Entre, Residuo, Igual, IgualIgual, Menor, Mayor,
    // ---- Combinaciones hechas con esos símbolos ----
    Diferente, MenorIgual, MayorIgual, Flecha, Potencia,
    MasIgual, MenosIgual, PorIgual, EntreIgual, ResiduoIgual, PotenciaIgual,
    // ---- El símbolo nuevo: ++ y -- (subir o bajar 1) ----
    MasMas, MenosMenos,
    // ---- El préstamo: &nombre ----
    Ampersand,
    // ---- Signos de estructura ----
    PuntoYComa, Coma, ParenIzq, ParenDer, LlaveIzq, LlaveDer, DosPuntos,
    PuntoPunto, PuntoPuntoIgual,
    // ---- Los corchetes de las listas: [1, 2, 3] y lista[0] ----
    CorcheteIzq, CorcheteDer,
    Fin,
}

// Un token es un pedacito, con la línea y la columna donde
// apareció para poder decirte dónde está el error (y para
// que el autocompletado sepa qué estás escribiendo).
#[derive(Debug, Clone)]
pub struct Token {
    pub tipo: TipoToken,
    pub linea: usize,
    pub columna: usize,
}

// Mira si una palabra es una de las 34 palabras clave.
// Si no lo es, es un nombre normal (un identificador).
fn palabra_clave(texto: &str) -> Option<TipoToken> {
    let token = match texto {
        "sea" => TipoToken::Sea,
        "mut" => TipoToken::Mut,
        "si" => TipoToken::Si,
        "sino" => TipoToken::Sino,
        "mientras" => TipoToken::Mientras,
        "para" => TipoToken::Para,
        "en" => TipoToken::En,
        "funcion" => TipoToken::Funcion,
        "devolver" => TipoToken::Devolver,
        "imprimir" => TipoToken::Imprimir,
        "leer" => TipoToken::Leer,
        "verdadero" => TipoToken::Verdadero,
        "falso" => TipoToken::Falso,
        "y" => TipoToken::Y,
        "o" => TipoToken::O,
        "no" => TipoToken::No,
        "romper" => TipoToken::Romper,
        "seguir" => TipoToken::Seguir,
        "nulo" => TipoToken::Nulo,
        "importar" => TipoToken::Importar,
        "numero" => TipoToken::Numero,
        "texto" => TipoToken::Texto,
        "logico" => TipoToken::Logico,
        "segun" => TipoToken::Segun,
        "paso" => TipoToken::Paso,
        "repetir" => TipoToken::Repetir,
        "lista" => TipoToken::Lista,
        "para_cada" => TipoToken::ParaCada,
        "constante" => TipoToken::Constante,
        "usar" => TipoToken::Usar,
        "mapa" => TipoToken::Mapa,
        "intentar" => TipoToken::Intentar,
        "si_falla" => TipoToken::SiFalla,
        "al_final" => TipoToken::AlFinal,
        _ => return None,
    };
    Some(token)
}

pub struct Lexer {
    caracteres: Vec<char>,
    posicion: usize,
    linea: usize,
    columna: usize,
}

impl Lexer {
    pub fn nuevo(fuente: &str) -> Self {
        Lexer {
            caracteres: fuente.chars().collect(),
            posicion: 0,
            linea: 1,
            columna: 1,
        }
    }

    fn al_final(&self) -> bool {
        self.posicion >= self.caracteres.len()
    }

    fn mirar(&self) -> char {
        if self.al_final() {
            '\0'
        } else {
            self.caracteres[self.posicion]
        }
    }

    fn mirar_siguiente(&self) -> char {
        if self.posicion + 1 >= self.caracteres.len() {
            '\0'
        } else {
            self.caracteres[self.posicion + 1]
        }
    }

    fn avanzar(&mut self) -> char {
        let c = self.mirar();
        self.posicion += 1;
        if c == '\n' {
            self.linea += 1;
            self.columna = 1;
        } else {
            self.columna += 1;
        }
        c
    }

    fn error(&self, mensaje: &str) -> String {
        format!("Error en la línea {}: {}", self.linea, mensaje)
    }

    fn token(&self, tipo: TipoToken) -> Token {
        Token {
            tipo,
            linea: self.linea,
            columna: self.columna,
        }
    }

    // Corta todo el texto en tokens, uno por uno.
    pub fn tokenizar(&mut self) -> Result<Vec<Token>, String> {
        let mut tokens = Vec::new();
        while !self.al_final() {
            let c = self.mirar();
            match c {
                ' ' | '\t' | '\r' => {
                    self.avanzar();
                }
                '\n' => {
                    self.avanzar();
                }
                '/' => {
                    if self.mirar_siguiente() == '/' {
                        // Comentario: // hasta el final de la línea
                        while !self.al_final() && self.mirar() != '\n' {
                            self.avanzar();
                        }
                    } else if self.mirar_siguiente() == '*' {
                        // Comentario de bloque: /* ... */ puede
                        // ocupar varias líneas.
                        self.avanzar(); // /
                        self.avanzar(); // *
                        let mut cerrado = false;
                        while !self.al_final() {
                            if self.mirar() == '*' && self.mirar_siguiente() == '/' {
                                self.avanzar(); // *
                                self.avanzar(); // /
                                cerrado = true;
                                break;
                            }
                            self.avanzar();
                        }
                        if !cerrado {
                            return Err(self.error("el comentario /* no se cerró: falta */ al final"));
                        }
                    } else if self.mirar_siguiente() == '=' {
                        self.avanzar(); // /
                        self.avanzar(); // =
                        tokens.push(self.token(TipoToken::EntreIgual));
                    } else {
                        tokens.push(self.token(TipoToken::Entre));
                        self.avanzar();
                    }
                }
                '"' => {
                    tokens.push(self.cadena()?);
                }
                '+' => {
                    self.avanzar();
                    if self.mirar() == '+' {
                        // ++ es subir 1: puntos++ es puntos + 1.
                        self.avanzar();
                        tokens.push(self.token(TipoToken::MasMas));
                    } else if self.mirar() == '=' {
                        self.avanzar();
                        tokens.push(self.token(TipoToken::MasIgual));
                    } else {
                        tokens.push(self.token(TipoToken::Mas));
                    }
                }
                '-' => {
                    self.avanzar();
                    if self.mirar() == '-' {
                        // -- es bajar 1: puntos-- es puntos - 1.
                        // (Un `-` seguido de un espacio no es `--`:
                        // 5 - -3 sigue siendo 5 menos -3.)
                        self.avanzar();
                        tokens.push(self.token(TipoToken::MenosMenos));
                    } else if self.mirar() == '=' {
                        self.avanzar();
                        tokens.push(self.token(TipoToken::MenosIgual));
                    } else {
                        tokens.push(self.token(TipoToken::Menos));
                    }
                }
                '*' => {
                    self.avanzar();
                    if self.mirar() == '*' {
                        // ** es la potencia: 2 ** 3 es 8.
                        // Y **= es la potencia con asignación:
                        // x **= 2 es x = x ** 2.
                        self.avanzar();
                        if self.mirar() == '=' {
                            self.avanzar();
                            tokens.push(self.token(TipoToken::PotenciaIgual));
                        } else {
                            tokens.push(self.token(TipoToken::Potencia));
                        }
                    } else if self.mirar() == '=' {
                        self.avanzar();
                        tokens.push(self.token(TipoToken::PorIgual));
                    } else {
                        tokens.push(self.token(TipoToken::Por));
                    }
                }
                '%' => {
                    self.avanzar();
                    if self.mirar() == '=' {
                        self.avanzar();
                        tokens.push(self.token(TipoToken::ResiduoIgual));
                    } else {
                        tokens.push(self.token(TipoToken::Residuo));
                    }
                }
                ';' => {
                    tokens.push(self.token(TipoToken::PuntoYComa));
                    self.avanzar();
                }
                ',' => {
                    tokens.push(self.token(TipoToken::Coma));
                    self.avanzar();
                }
                '(' => {
                    tokens.push(self.token(TipoToken::ParenIzq));
                    self.avanzar();
                }
                ')' => {
                    tokens.push(self.token(TipoToken::ParenDer));
                    self.avanzar();
                }
                '{' => {
                    tokens.push(self.token(TipoToken::LlaveIzq));
                    self.avanzar();
                }
                '}' => {
                    tokens.push(self.token(TipoToken::LlaveDer));
                    self.avanzar();
                }
                '[' => {
                    tokens.push(self.token(TipoToken::CorcheteIzq));
                    self.avanzar();
                }
                ']' => {
                    tokens.push(self.token(TipoToken::CorcheteDer));
                    self.avanzar();
                }
                ':' => {
                    tokens.push(self.token(TipoToken::DosPuntos));
                    self.avanzar();
                }
                '=' => {
                    self.avanzar();
                    if self.mirar() == '=' {
                        self.avanzar();
                        tokens.push(self.token(TipoToken::IgualIgual));
                    } else if self.mirar() == '>' {
                        self.avanzar();
                        tokens.push(self.token(TipoToken::Flecha));
                    } else {
                        tokens.push(self.token(TipoToken::Igual));
                    }
                }
                '!' => {
                    self.avanzar();
                    if self.mirar() == '=' {
                        self.avanzar();
                        tokens.push(self.token(TipoToken::Diferente));
                    } else {
                        // `!` solo es lo mismo que `no`: !verdadero
                        tokens.push(self.token(TipoToken::No));
                    }
                }
                '&' => {
                    // `&nombre` es prestar, no mover.
                    tokens.push(self.token(TipoToken::Ampersand));
                    self.avanzar();
                }
                '<' => {
                    self.avanzar();
                    if self.mirar() == '=' {
                        self.avanzar();
                        tokens.push(self.token(TipoToken::MenorIgual));
                    } else {
                        tokens.push(self.token(TipoToken::Menor));
                    }
                }
                '>' => {
                    self.avanzar();
                    if self.mirar() == '=' {
                        self.avanzar();
                        tokens.push(self.token(TipoToken::MayorIgual));
                    } else {
                        tokens.push(self.token(TipoToken::Mayor));
                    }
                }
                '.' => {
                    self.avanzar();
                    if self.mirar() == '.' {
                        self.avanzar();
                        if self.mirar() == '=' {
                            self.avanzar();
                            tokens.push(self.token(TipoToken::PuntoPuntoIgual));
                        } else {
                            tokens.push(self.token(TipoToken::PuntoPunto));
                        }
                    } else {
                        return Err(self.error("el punto solo se usa en rangos, como 1..5"));
                    }
                }
                _ => {
                    if c.is_alphabetic() || c == '_' {
                        tokens.push(self.identificador());
                    } else if c.is_ascii_digit() {
                        tokens.push(self.numero()?);
                    } else {
                        return Err(self.error(&format!("no conozco el símbolo '{}'", c)));
                    }
                }
            }
        }
        tokens.push(Token {
            tipo: TipoToken::Fin,
            linea: self.linea,
            columna: self.columna,
        });
        Ok(tokens)
    }

    // Lee un nombre o una palabra clave, letra por letra.
    fn identificador(&mut self) -> Token {
        let mut texto = String::new();
        while !self.al_final() && (self.mirar().is_alphanumeric() || self.mirar() == '_') {
            texto.push(self.avanzar());
        }
        let tipo = palabra_clave(&texto).unwrap_or(TipoToken::Identificador(texto));
        Token {
            tipo,
            linea: self.linea,
            columna: self.columna,
        }
    }

    // Lee un número. Acepta decimales como 3.14 y guiones
    // bajos para leer mejor: 1_000 es lo mismo que 1000.
    // El punto solo es decimal si después viene un dígito:
    // así 1..5 no se confunde con 1.5.
    fn numero(&mut self) -> Result<Token, String> {
        let mut texto = String::new();
        while !self.al_final() && (self.mirar().is_ascii_digit() || self.mirar() == '_') {
            let c = self.mirar();
            if c != '_' {
                texto.push(c);
            }
            self.avanzar();
        }
        if self.mirar() == '.' && self.mirar_siguiente().is_ascii_digit() {
            texto.push(self.avanzar());
            while !self.al_final() && (self.mirar().is_ascii_digit() || self.mirar() == '_') {
                let c = self.mirar();
                if c != '_' {
                    texto.push(c);
                }
                self.avanzar();
            }
        }
        match texto.parse::<f64>() {
            Ok(n) => Ok(Token {
                tipo: TipoToken::NumeroLit(n),
                linea: self.linea,
                columna: self.columna,
            }),
            Err(_) => Err(self.error(&format!("no entendí el número '{}'", texto))),
        }
    }

    // Lee un texto entre comillas. Acepta \n (nueva línea),
    // \" (comilla) y \\ (barra invertida).
    fn cadena(&mut self) -> Result<Token, String> {
        self.avanzar(); // comilla de apertura
        let mut texto = String::new();
        while !self.al_final() && self.mirar() != '"' {
            let c = self.mirar();
            if c == '\n' {
                return Err(self.error("la cadena no se cerró: falta la comilla \" al final"));
            }
            if c == '\\' {
                self.avanzar();
                let siguiente = self.mirar();
                match siguiente {
                    'n' => texto.push('\n'),
                    't' => texto.push('\t'),
                    '"' => texto.push('"'),
                    '\\' => texto.push('\\'),
                    _ => texto.push(siguiente),
                }
                self.avanzar();
            } else {
                texto.push(self.avanzar());
            }
        }
        if self.al_final() {
            return Err(self.error("la cadena no se cerró: falta la comilla \" al final"));
        }
        self.avanzar(); // comilla de cierre
        Ok(Token {
            tipo: TipoToken::Cadena(texto),
            linea: self.linea,
            columna: self.columna,
        })
    }
}
