// ============================================================
//  ECLIPSE 1.1 - JSON chiquito
//  Archivo: src/json.rs
//
//  Qué es esto:
//  Un JSON chiquito, solo con la librería estándar, para que
//  el servidor LSP pueda leer y escribir los mensajes del
//  protocolo (que son JSON). No descarga nada: es solo este
//  archivo.
//
//  Un Json puede ser: nulo, verdadero/falso, un número, un
//  texto, una lista o un objeto (pares nombre: valor).
// ============================================================

#[derive(Debug, Clone, PartialEq)]
pub enum Json {
    Nulo,
    Booleano(bool),
    Numero(f64),
    Texto(String),
    Arreglo(Vec<Json>),
    Objeto(Vec<(String, Json)>),
}

impl Json {
    // Lee un JSON de un texto. Devuelve un error en español
    // si el texto no es JSON válido.
    pub fn parsear(texto: &str) -> Result<Json, String> {
        let mut p = ParserJson {
            chars: texto.chars().collect(),
            pos: 0,
        };
        p.espacios();
        let valor = p.valor()?;
        p.espacios();
        if p.pos != p.chars.len() {
            return Err("hay texto de más después del JSON".to_string());
        }
        Ok(valor)
    }

    // Escribe el JSON como texto, sin espacios de más.
    pub fn texto(&self) -> String {
        let mut s = String::new();
        self.escribir(&mut s);
        s
    }

    fn escribir(&self, s: &mut String) {
        match self {
            Json::Nulo => s.push_str("null"),
            Json::Booleano(b) => s.push_str(if *b { "true" } else { "false" }),
            Json::Numero(n) => {
                if n.fract() == 0.0 && n.is_finite() && n.abs() < 1e15 {
                    s.push_str(&format!("{}", *n as i64));
                } else {
                    s.push_str(&format!("{}", n));
                }
            }
            Json::Texto(t) => {
                s.push('"');
                for c in t.chars() {
                    match c {
                        '"' => s.push_str("\\\""),
                        '\\' => s.push_str("\\\\"),
                        '\n' => s.push_str("\\n"),
                        '\r' => s.push_str("\\r"),
                        '\t' => s.push_str("\\t"),
                        c if (c as u32) < 0x20 => {
                            s.push_str(&format!("\\u{:04x}", c as u32));
                        }
                        c => s.push(c),
                    }
                }
                s.push('"');
            }
            Json::Arreglo(v) => {
                s.push('[');
                for (i, e) in v.iter().enumerate() {
                    if i > 0 {
                        s.push(',');
                    }
                    e.escribir(s);
                }
                s.push(']');
            }
            Json::Objeto(v) => {
                s.push('{');
                for (i, (k, val)) in v.iter().enumerate() {
                    if i > 0 {
                        s.push(',');
                    }
                    Json::Texto(k.clone()).escribir(s);
                    s.push(':');
                    val.escribir(s);
                }
                s.push('}');
            }
        }
    }

    // Busca un campo de un objeto, por su nombre.
    pub fn get(&self, clave: &str) -> Option<&Json> {
        match self {
            Json::Objeto(v) => v
                .iter()
                .find(|(k, _)| k == clave)
                .map(|(_, val)| val),
            _ => None,
        }
    }

    // El valor como texto, si es un texto.
    pub fn como_texto(&self) -> Option<&str> {
        match self {
            Json::Texto(s) => Some(s),
            _ => None,
        }
    }

    // El valor como entero, si es un número entero.
    pub fn como_entero(&self) -> Option<i64> {
        match self {
            Json::Numero(n) if n.fract() == 0.0 => Some(*n as i64),
            _ => None,
        }
    }

    // El valor como verdadero/falso, si es un lógico.
    // (Lo usa el probador del LSP, no el servidor.)
    #[allow(dead_code)]
    pub fn como_bool(&self) -> Option<bool> {
        match self {
            Json::Booleano(b) => Some(*b),
            _ => None,
        }
    }
}

// Crea un objeto JSON de pares (nombre, valor).
pub fn objeto(pares: Vec<(&str, Json)>) -> Json {
    Json::Objeto(
        pares
            .into_iter()
            .map(|(k, v)| (k.to_string(), v))
            .collect(),
    )
}

pub fn texto(s: &str) -> Json {
    Json::Texto(s.to_string())
}

pub fn entero(n: i64) -> Json {
    Json::Numero(n as f64)
}

pub fn booleano(b: bool) -> Json {
    Json::Booleano(b)
}

pub fn arreglo(v: Vec<Json>) -> Json {
    Json::Arreglo(v)
}

struct ParserJson {
    chars: Vec<char>,
    pos: usize,
}

impl ParserJson {
    fn espacios(&mut self) {
        while self.pos < self.chars.len() && self.chars[self.pos].is_whitespace() {
            self.pos += 1;
        }
    }

    fn mirar(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn valor(&mut self) -> Result<Json, String> {
        match self.mirar() {
            Some('{') => self.objeto(),
            Some('[') => self.arreglo(),
            Some('"') => Ok(Json::Texto(self.texto()?)),
            Some('t') => {
                self.literal("true")?;
                Ok(Json::Booleano(true))
            }
            Some('f') => {
                self.literal("false")?;
                Ok(Json::Booleano(false))
            }
            Some('n') => {
                self.literal("null")?;
                Ok(Json::Nulo)
            }
            Some(c) if c == '-' || c.is_ascii_digit() => self.numero(),
            Some(c) => Err(format!("no esperaba '{}' en el JSON", c)),
            None => Err("el JSON se acabó de repente".to_string()),
        }
    }

    fn literal(&mut self, palabra: &str) -> Result<(), String> {
        for c in palabra.chars() {
            if self.mirar() != Some(c) {
                return Err(format!("no esperaba eso en el JSON, quería '{}'", palabra));
            }
            self.pos += 1;
        }
        Ok(())
    }

    fn numero(&mut self) -> Result<Json, String> {
        let inicio = self.pos;
        if self.mirar() == Some('-') {
            self.pos += 1;
        }
        while matches!(self.mirar(), Some(c) if c.is_ascii_digit()) {
            self.pos += 1;
        }
        if self.mirar() == Some('.') {
            self.pos += 1;
            while matches!(self.mirar(), Some(c) if c.is_ascii_digit()) {
                self.pos += 1;
            }
        }
        if matches!(self.mirar(), Some('e') | Some('E')) {
            self.pos += 1;
            if matches!(self.mirar(), Some('+') | Some('-')) {
                self.pos += 1;
            }
            while matches!(self.mirar(), Some(c) if c.is_ascii_digit()) {
                self.pos += 1;
            }
        }
        let texto: String = self.chars[inicio..self.pos].iter().collect();
        texto
            .parse::<f64>()
            .map(Json::Numero)
            .map_err(|_| format!("el número '{}' no es válido", texto))
    }

    fn texto(&mut self) -> Result<String, String> {
        // Se asume que el carácter actual es ".
        self.pos += 1;
        let mut s = String::new();
        loop {
            match self.mirar() {
                None => return Err("el texto se acabó sin cerrar las comillas".to_string()),
                Some('"') => {
                    self.pos += 1;
                    return Ok(s);
                }
                Some('\\') => {
                    self.pos += 1;
                    match self.mirar() {
                        Some('"') => s.push('"'),
                        Some('\\') => s.push('\\'),
                        Some('/') => s.push('/'),
                        Some('n') => s.push('\n'),
                        Some('r') => s.push('\r'),
                        Some('t') => s.push('\t'),
                        Some('b') => s.push('\u{0008}'),
                        Some('f') => s.push('\u{000C}'),
                        Some('u') => {
                            self.pos += 1;
                            let mut codigo = 0u32;
                            for _ in 0..4 {
                                let c = self.mirar().ok_or_else(|| {
                                    "el código \\u se acabó de repente".to_string()
                                })?;
                                let d = c.to_digit(16).ok_or_else(|| {
                                    format!("'{}' no es un dígito hexadecimal", c)
                                })?;
                                codigo = codigo * 16 + d;
                                self.pos += 1;
                            }
                            // Si es un par sustituto (surrogate),
                            // leer la segunda parte \uXXXX.
                            if (0xD800..0xDC00).contains(&codigo) {
                                if self.mirar() == Some('\\') {
                                    self.pos += 1;
                                    if self.mirar() == Some('u') {
                                        self.pos += 1;
                                        let mut codigo2 = 0u32;
                                        for _ in 0..4 {
                                            let c = self.mirar().ok_or_else(|| {
                                                "el código \\u se acabó de repente".to_string()
                                            })?;
                                            let d = c.to_digit(16).ok_or_else(|| {
                                                format!("'{}' no es un dígito hexadecimal", c)
                                            })?;
                                            codigo2 = codigo2 * 16 + d;
                                            self.pos += 1;
                                        }
                                        let combinado = 0x10000
                                            + ((codigo - 0xD800) << 10)
                                            + (codigo2 - 0xDC00);
                                        if let Some(c) = char::from_u32(combinado) {
                                            s.push(c);
                                            continue;
                                        }
                                    }
                                }
                                s.push('\u{FFFD}');
                                continue;
                            }
                            match char::from_u32(codigo) {
                                Some(c) => s.push(c),
                                None => s.push('\u{FFFD}'),
                            }
                            continue;
                        }
                        Some(c) => {
                            return Err(format!("no conozco el escape '\\{}'", c));
                        }
                        None => {
                            return Err("el texto se acabó sin cerrar las comillas".to_string())
                        }
                    }
                    self.pos += 1;
                }
                Some(c) => {
                    s.push(c);
                    self.pos += 1;
                }
            }
        }
    }

    fn arreglo(&mut self) -> Result<Json, String> {
        self.pos += 1; // [
        let mut valores = Vec::new();
        self.espacios();
        if self.mirar() == Some(']') {
            self.pos += 1;
            return Ok(Json::Arreglo(valores));
        }
        loop {
            self.espacios();
            valores.push(self.valor()?);
            self.espacios();
            match self.mirar() {
                Some(',') => {
                    self.pos += 1;
                }
                Some(']') => {
                    self.pos += 1;
                    return Ok(Json::Arreglo(valores));
                }
                _ => return Err("en la lista falta ',' o ']'".to_string()),
            }
        }
    }

    fn objeto(&mut self) -> Result<Json, String> {
        self.pos += 1; // {
        let mut pares = Vec::new();
        self.espacios();
        if self.mirar() == Some('}') {
            self.pos += 1;
            return Ok(Json::Objeto(pares));
        }
        loop {
            self.espacios();
            if self.mirar() != Some('"') {
                return Err("en el objeto falta el nombre entre comillas".to_string());
            }
            let clave = self.texto()?;
            self.espacios();
            if self.mirar() != Some(':') {
                return Err("en el objeto falta ':'".to_string());
            }
            self.pos += 1;
            self.espacios();
            let valor = self.valor()?;
            pares.push((clave, valor));
            self.espacios();
            match self.mirar() {
                Some(',') => {
                    self.pos += 1;
                }
                Some('}') => {
                    self.pos += 1;
                    return Ok(Json::Objeto(pares));
                }
                _ => return Err("en el objeto falta ',' o '}'".to_string()),
            }
        }
    }
}
