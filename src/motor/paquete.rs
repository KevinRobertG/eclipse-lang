// ============================================================
//  ECLIPSE 1.2 - El paquete standalone (sin Rust, sin Cargo)
//  Archivo: src/motor/paquete.rs
//
//  Qué hace:
//  Eclipse ya no necesita que tengas Rust ni Cargo instalados
//  para correr tus programas. Este archivo guarda un programa
//  ya compilado (su bytecode) en un formato binario chiquito
//  (ECBIN), y pega ese bytecode al final del propio programa
//  `eclipse`, para crear UN SOLO archivo que corre solo:
//
//      eclipse empaquetar juego.ec
//      -> juego (o juego.exe en Windows): el eclipse + tu juego
//
//  Ese archivo se puede copiar a otra computadora y correrlo
//  sin instalar nada: al arrancar, Eclipse mira si su propio
//  archivo trae un paquete pegado al final, y si lo trae,
//  corre ese programa en su máquina virtual.
//
//  Ojo, con honestidad: esto NO es código máquina como el
//  .exe de `rustc` (para ese sí hace falta Rust). Es el
//  Eclipse + tu programa ya compilado a bytecode, en un
//  solo archivo. Para el usuario es lo mismo: doble clic
//  y corre, sin instalar nada.
//
//  Formato ECBIN (todo en little-endian):
//      "ECBIN" (5 bytes) + versión (1 byte = 1)
//      pedazo principal + funciones (cuántas + nombre + pedazo)
//  Pedazo:
//      max_slots (u32), constantes, nombres, parámetros, código
//  Valor: etiqueta (u8) + datos
//      0 = nulo, 1 = numero (f64), 2 = texto, 3 = lógico,
//      4 = lista (cuántas + valores)
//  Texto: largo (u32) + bytes UTF-8.
//  Instrucción: código (u8) + sus datos (u32 o u8).
//
//  Solo usa la librería estándar de Rust.
// ============================================================

use crate::motor::bytecode::{InfoManejador, InfoParaAvanzar, Instruccion, Pedazo, Programa};
use crate::motor::valor::Valor;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

// La marca del principio de un paquete ECBIN.
const MARCA_ECBIN: &[u8; 5] = b"ECBIN";
const VERSION_ECBIN: u8 = 1;

// La marca del final de un ejecutable empaquetado:
// [largo del paquete (u64)] + "ECLIPSE1" (8 bytes).
const MARCA_PIE: &[u8; 8] = b"ECLIPSE1";

// Los códigos de cada instrucción en el formato binario.
// Van en el mismo orden que en `Instruccion`.
const OP_CONSTANTE: u8 = 0;
const OP_NULO: u8 = 1;
const OP_VERDADERO: u8 = 2;
const OP_FALSO: u8 = 3;
const OP_SUMAR: u8 = 4;
const OP_RESTAR: u8 = 5;
const OP_MULTIPLICAR: u8 = 6;
const OP_DIVIDIR: u8 = 7;
const OP_RESIDUO: u8 = 8;
const OP_POTENCIA: u8 = 9;
const OP_NEGATIVO: u8 = 10;
const OP_NO: u8 = 11;
const OP_CONVERTIR_LOGICO: u8 = 12;
const OP_IGUAL: u8 = 13;
const OP_DIFERENTE: u8 = 14;
const OP_MENOR: u8 = 15;
const OP_MENOR_IGUAL: u8 = 16;
const OP_MAYOR: u8 = 17;
const OP_MAYOR_IGUAL: u8 = 18;
const OP_SACAR: u8 = 19;
const OP_DEFINIR_VARIABLE: u8 = 20;
const OP_DEFINIR_CONSTANTE: u8 = 21;
const OP_OBTENER_VARIABLE: u8 = 22;
const OP_ASIGNAR_VARIABLE: u8 = 23;
const OP_MOVER: u8 = 24;
const OP_PRESTAR_MUT: u8 = 25;
const OP_PRESTAR: u8 = 26;
const OP_VERIFICAR_TIPO: u8 = 27;
const OP_CREAR_LISTA: u8 = 28;
const OP_INDICE: u8 = 29;
const OP_LEER_ENTRADA: u8 = 30;
const OP_INCREMENTAR: u8 = 31;
const OP_SALTAR: u8 = 32;
const OP_SALTAR_SI_FALSO: u8 = 33;
const OP_LLAMAR: u8 = 34;
const OP_REGISTRAR_FUNCION: u8 = 35;
const OP_RETORNAR: u8 = 36;
const OP_VERIFICAR_LISTA: u8 = 37;
const OP_PARA_AVANZAR: u8 = 38;
const OP_AGREGAR_EN_SITIO: u8 = 39;
const OP_CREAR_MAPA: u8 = 40;
const OP_ASIGNAR_INDICE: u8 = 41;
const OP_INTENTAR: u8 = 42;
const OP_QUITAR_MANEJADOR: u8 = 43;
const OP_RELANZAR: u8 = 44;

// =============== ESCRIBIR ===============

// Escribe bytes en un vector, poco a poco.
struct Escritor {
    datos: Vec<u8>,
}

impl Escritor {
    fn nuevo() -> Self {
        Escritor { datos: Vec::new() }
    }

    fn u8(&mut self, v: u8) {
        self.datos.push(v);
    }

    fn u32(&mut self, v: u32) {
        self.datos.extend_from_slice(&v.to_le_bytes());
    }

    fn u64(&mut self, v: u64) {
        self.datos.extend_from_slice(&v.to_le_bytes());
    }

    fn texto(&mut self, s: &str) {
        self.u32(s.len() as u32);
        self.datos.extend_from_slice(s.as_bytes());
    }

    fn valor(&mut self, v: &Valor) {
        match v {
            Valor::Nulo => self.u8(0),
            Valor::Numero(n) => {
                self.u8(1);
                self.u64(n.to_bits());
            }
            Valor::Texto(s) => {
                self.u8(2);
                self.texto(s);
            }
            Valor::Logico(b) => {
                self.u8(3);
                self.u8(if *b { 1 } else { 0 });
            }
            Valor::Lista(items) => {
                self.u8(4);
                self.u32(items.len() as u32);
                for item in items.iter() {
                    self.valor(item);
                }
            }
            Valor::Mapa(pares) => {
                self.u8(5);
                self.u32(pares.len() as u32);
                for (clave, valor) in pares.iter() {
                    self.valor(clave);
                    self.valor(valor);
                }
            }
        }
    }

    fn instruccion(&mut self, instr: &Instruccion) {
        match instr {
            Instruccion::Constante(i) => {
                self.u8(OP_CONSTANTE);
                self.u32(*i as u32);
            }
            Instruccion::Nulo => self.u8(OP_NULO),
            Instruccion::Verdadero => self.u8(OP_VERDADERO),
            Instruccion::Falso => self.u8(OP_FALSO),
            Instruccion::Sumar => self.u8(OP_SUMAR),
            Instruccion::Restar => self.u8(OP_RESTAR),
            Instruccion::Multiplicar => self.u8(OP_MULTIPLICAR),
            Instruccion::Dividir => self.u8(OP_DIVIDIR),
            Instruccion::Residuo => self.u8(OP_RESIDUO),
            Instruccion::Potencia => self.u8(OP_POTENCIA),
            Instruccion::Negativo => self.u8(OP_NEGATIVO),
            Instruccion::No => self.u8(OP_NO),
            Instruccion::ConvertirLogico => self.u8(OP_CONVERTIR_LOGICO),
            Instruccion::Igual => self.u8(OP_IGUAL),
            Instruccion::Diferente => self.u8(OP_DIFERENTE),
            Instruccion::Menor => self.u8(OP_MENOR),
            Instruccion::MenorIgual => self.u8(OP_MENOR_IGUAL),
            Instruccion::Mayor => self.u8(OP_MAYOR),
            Instruccion::MayorIgual => self.u8(OP_MAYOR_IGUAL),
            Instruccion::Sacar => self.u8(OP_SACAR),
            Instruccion::DefinirVariable(slot, m, g, n) => {
                self.u8(OP_DEFINIR_VARIABLE);
                self.u32(*slot as u32);
                self.u8(if *m { 1 } else { 0 });
                self.u8(if *g { 1 } else { 0 });
                self.u32(*n as u32);
            }
            Instruccion::DefinirConstante(slot, g, n) => {
                self.u8(OP_DEFINIR_CONSTANTE);
                self.u32(*slot as u32);
                self.u8(if *g { 1 } else { 0 });
                self.u32(*n as u32);
            }
            Instruccion::ObtenerVariable(slot, g, n) => {
                self.u8(OP_OBTENER_VARIABLE);
                self.u32(*slot as u32);
                self.u8(if *g { 1 } else { 0 });
                self.u32(*n as u32);
            }
            Instruccion::AsignarVariable(slot, g, n) => {
                self.u8(OP_ASIGNAR_VARIABLE);
                self.u32(*slot as u32);
                self.u8(if *g { 1 } else { 0 });
                self.u32(*n as u32);
            }
            Instruccion::Mover(slot, g, n, d) => {
                self.u8(OP_MOVER);
                self.u32(*slot as u32);
                self.u8(if *g { 1 } else { 0 });
                self.u32(*n as u32);
                self.u32(*d as u32);
            }
            Instruccion::PrestarMut(slot, g, n, p) => {
                self.u8(OP_PRESTAR_MUT);
                self.u32(*slot as u32);
                self.u8(if *g { 1 } else { 0 });
                self.u32(*n as u32);
                self.u32(*p as u32);
            }
            Instruccion::Prestar(slot, g, n, p) => {
                self.u8(OP_PRESTAR);
                self.u32(*slot as u32);
                self.u8(if *g { 1 } else { 0 });
                self.u32(*n as u32);
                self.u32(*p as u32);
            }
            Instruccion::VerificarTipo(t, slot, g, n) => {
                self.u8(OP_VERIFICAR_TIPO);
                self.u32(*t as u32);
                self.u32(*slot as u32);
                self.u8(if *g { 1 } else { 0 });
                self.u32(*n as u32);
            }
            Instruccion::VerificarLista(slot, g, n) => {
                self.u8(OP_VERIFICAR_LISTA);
                self.u32(*slot as u32);
                self.u8(if *g { 1 } else { 0 });
                self.u32(*n as u32);
            }
            Instruccion::ParaAvanzar(info) => {
                self.u8(OP_PARA_AVANZAR);
                self.u32(info.var_slot as u32);
                self.u8(if info.var_global { 1 } else { 0 });
                self.u32(info.fin_slot as u32);
                self.u8(if info.fin_global { 1 } else { 0 });
                self.u32(info.paso_slot as u32);
                self.u8(if info.paso_global { 1 } else { 0 });
                self.u32(info.salto_cuerpo as u32);
                self.u8(if info.inclusivo { 1 } else { 0 });
            }
            Instruccion::AgregarEnSitio(slot, g, n) => {
                self.u8(OP_AGREGAR_EN_SITIO);
                self.u32(*slot as u32);
                self.u8(if *g { 1 } else { 0 });
                self.u32(*n as u32);
            }
            Instruccion::CrearMapa(n) => {
                self.u8(OP_CREAR_MAPA);
                self.u32(*n as u32);
            }
            Instruccion::AsignarIndice(slot, g, n) => {
                self.u8(OP_ASIGNAR_INDICE);
                self.u32(*slot as u32);
                self.u8(if *g { 1 } else { 0 });
                self.u32(*n as u32);
            }
            Instruccion::Intentar(info) => {
                self.u8(OP_INTENTAR);
                // Cada destino: 1 si existe y luego su número.
                match info.salto_falla {
                    Some(d) => {
                        self.u8(1);
                        self.u32(d as u32);
                    }
                    None => self.u8(0),
                }
                match info.salto_final {
                    Some(d) => {
                        self.u8(1);
                        self.u32(d as u32);
                    }
                    None => self.u8(0),
                }
            }
            Instruccion::QuitarManejador => self.u8(OP_QUITAR_MANEJADOR),
            Instruccion::Relanzar => self.u8(OP_RELANZAR),
            Instruccion::CrearLista(n) => {
                self.u8(OP_CREAR_LISTA);
                self.u32(*n as u32);
            }
            Instruccion::Indice => self.u8(OP_INDICE),
            Instruccion::LeerEntrada(slot, g, n, mensaje) => {
                self.u8(OP_LEER_ENTRADA);
                self.u32(*slot as u32);
                self.u8(if *g { 1 } else { 0 });
                self.u32(*n as u32);
                match mensaje {
                    Some(m) => {
                        self.u8(1);
                        self.u32(*m as u32);
                    }
                    None => self.u8(0),
                }
            }
            Instruccion::Incrementar(slot, g, n, subir) => {
                self.u8(OP_INCREMENTAR);
                self.u32(*slot as u32);
                self.u8(if *g { 1 } else { 0 });
                self.u32(*n as u32);
                self.u8(if *subir { 1 } else { 0 });
            }
            Instruccion::Saltar(d) => {
                self.u8(OP_SALTAR);
                self.u32(*d as u32);
            }
            Instruccion::SaltarSiFalso(d) => {
                self.u8(OP_SALTAR_SI_FALSO);
                self.u32(*d as u32);
            }
            Instruccion::Llamar(n, a) => {
                self.u8(OP_LLAMAR);
                self.u32(*n as u32);
                self.u32(*a as u32);
            }
            Instruccion::RegistrarFuncion(n) => {
                self.u8(OP_REGISTRAR_FUNCION);
                self.u32(*n as u32);
            }
            Instruccion::Retornar => self.u8(OP_RETORNAR),
        }
    }

    fn pedazo(&mut self, p: &Pedazo) {
        self.u32(p.max_slots as u32);
        self.u32(p.constantes.len() as u32);
        for c in &p.constantes {
            self.valor(c);
        }
        self.u32(p.nombres.len() as u32);
        for n in &p.nombres {
            self.texto(n);
        }
        self.u32(p.parametros.len() as u32);
        for n in &p.parametros {
            self.texto(n);
        }
        self.u32(p.codigo.len() as u32);
        for instr in &p.codigo {
            self.instruccion(instr);
        }
    }
}

// Convierte un programa ya compilado a bytes ECBIN.
pub fn guardar_programa(programa: &Programa) -> Vec<u8> {
    let mut e = Escritor::nuevo();
    e.datos.extend_from_slice(MARCA_ECBIN);
    e.u8(VERSION_ECBIN);
    e.pedazo(&programa.principal);
    let mut nombres: Vec<&String> = programa.funciones.keys().collect();
    nombres.sort();
    e.u32(nombres.len() as u32);
    for nombre in nombres {
        e.texto(nombre);
        e.pedazo(&programa.funciones[nombre]);
    }
    e.datos
}

// =============== LEER ===============

// Lee bytes poco a poco, con errores en español.
struct Lector<'a> {
    datos: &'a [u8],
    posicion: usize,
}

impl<'a> Lector<'a> {
    fn nuevo(datos: &'a [u8]) -> Self {
        Lector {
            datos,
            posicion: 0,
        }
    }

    fn u8(&mut self) -> Result<u8, String> {
        let b = self
            .datos
            .get(self.posicion)
            .ok_or_else(|| "el paquete se acabó antes de tiempo".to_string())?;
        self.posicion += 1;
        Ok(*b)
    }

    fn u32(&mut self) -> Result<u32, String> {
        if self.posicion + 4 > self.datos.len() {
            return Err("el paquete se acabó antes de tiempo".to_string());
        }
        let mut bytes = [0u8; 4];
        bytes.copy_from_slice(&self.datos[self.posicion..self.posicion + 4]);
        self.posicion += 4;
        Ok(u32::from_le_bytes(bytes))
    }

    fn u64(&mut self) -> Result<u64, String> {
        if self.posicion + 8 > self.datos.len() {
            return Err("el paquete se acabó antes de tiempo".to_string());
        }
        let mut bytes = [0u8; 8];
        bytes.copy_from_slice(&self.datos[self.posicion..self.posicion + 8]);
        self.posicion += 8;
        Ok(u64::from_le_bytes(bytes))
    }

    fn texto(&mut self) -> Result<String, String> {
        let largo = self.u32()? as usize;
        if self.posicion + largo > self.datos.len() {
            return Err("el paquete se acabó antes de tiempo".to_string());
        }
        let trozo = &self.datos[self.posicion..self.posicion + largo];
        self.posicion += largo;
        String::from_utf8(trozo.to_vec())
            .map_err(|_| "el paquete trae un texto que no es válido".to_string())
    }

    fn valor(&mut self) -> Result<Valor, String> {
        match self.u8()? {
            0 => Ok(Valor::Nulo),
            1 => Ok(Valor::Numero(f64::from_bits(self.u64()?))),
            2 => Ok(Valor::Texto(self.texto()?)),
            3 => Ok(Valor::Logico(self.u8()? != 0)),
            4 => {
                let n = self.u32()? as usize;
                let mut items = Vec::new();
                for _ in 0..n {
                    items.push(self.valor()?);
                }
                Ok(Valor::lista(items))
            }
            5 => {
                let n = self.u32()? as usize;
                let mut pares = Vec::new();
                for _ in 0..n {
                    let clave = self.valor()?;
                    let valor = self.valor()?;
                    pares.push((clave, valor));
                }
                Ok(Valor::mapa(pares))
            }
            otro => Err(format!(
                "el paquete trae un valor raro (etiqueta {})",
                otro
            )),
        }
    }

    fn instruccion(&mut self) -> Result<Instruccion, String> {
        let codigo = self.u8()?;
        let u = |l: &mut Self| l.u32().map(|n| n as usize);
        let b = |l: &mut Self| l.u8().map(|n| n != 0);
        match codigo {
            OP_CONSTANTE => Ok(Instruccion::Constante(u(self)?)),
            OP_NULO => Ok(Instruccion::Nulo),
            OP_VERDADERO => Ok(Instruccion::Verdadero),
            OP_FALSO => Ok(Instruccion::Falso),
            OP_SUMAR => Ok(Instruccion::Sumar),
            OP_RESTAR => Ok(Instruccion::Restar),
            OP_MULTIPLICAR => Ok(Instruccion::Multiplicar),
            OP_DIVIDIR => Ok(Instruccion::Dividir),
            OP_RESIDUO => Ok(Instruccion::Residuo),
            OP_POTENCIA => Ok(Instruccion::Potencia),
            OP_NEGATIVO => Ok(Instruccion::Negativo),
            OP_NO => Ok(Instruccion::No),
            OP_CONVERTIR_LOGICO => Ok(Instruccion::ConvertirLogico),
            OP_IGUAL => Ok(Instruccion::Igual),
            OP_DIFERENTE => Ok(Instruccion::Diferente),
            OP_MENOR => Ok(Instruccion::Menor),
            OP_MENOR_IGUAL => Ok(Instruccion::MenorIgual),
            OP_MAYOR => Ok(Instruccion::Mayor),
            OP_MAYOR_IGUAL => Ok(Instruccion::MayorIgual),
            OP_SACAR => Ok(Instruccion::Sacar),
            OP_DEFINIR_VARIABLE => Ok(Instruccion::DefinirVariable(
                u(self)?,
                b(self)?,
                b(self)?,
                u(self)?,
            )),
            OP_DEFINIR_CONSTANTE => Ok(Instruccion::DefinirConstante(
                u(self)?,
                b(self)?,
                u(self)?,
            )),
            OP_OBTENER_VARIABLE => Ok(Instruccion::ObtenerVariable(
                u(self)?,
                b(self)?,
                u(self)?,
            )),
            OP_ASIGNAR_VARIABLE => Ok(Instruccion::AsignarVariable(
                u(self)?,
                b(self)?,
                u(self)?,
            )),
            OP_MOVER => Ok(Instruccion::Mover(u(self)?, b(self)?, u(self)?, u(self)?)),
            OP_PRESTAR_MUT => Ok(Instruccion::PrestarMut(
                u(self)?,
                b(self)?,
                u(self)?,
                u(self)?,
            )),
            OP_PRESTAR => Ok(Instruccion::Prestar(u(self)?, b(self)?, u(self)?, u(self)?)),
            OP_VERIFICAR_TIPO => Ok(Instruccion::VerificarTipo(
                u(self)?,
                u(self)?,
                b(self)?,
                u(self)?,
            )),
            OP_VERIFICAR_LISTA => Ok(Instruccion::VerificarLista(
                u(self)?,
                b(self)?,
                u(self)?,
            )),
            OP_AGREGAR_EN_SITIO => Ok(Instruccion::AgregarEnSitio(
                u(self)?,
                b(self)?,
                u(self)?,
            )),
            OP_CREAR_MAPA => Ok(Instruccion::CrearMapa(u(self)?)),
            OP_ASIGNAR_INDICE => Ok(Instruccion::AsignarIndice(
                u(self)?,
                b(self)?,
                u(self)?,
            )),
            OP_INTENTAR => {
                let tiene_falla = self.u8()? != 0;
                let salto_falla = if tiene_falla {
                    Some(u(self)?)
                } else {
                    None
                };
                let tiene_final = self.u8()? != 0;
                let salto_final = if tiene_final {
                    Some(u(self)?)
                } else {
                    None
                };
                Ok(Instruccion::Intentar(InfoManejador {
                    salto_falla,
                    salto_final,
                }))
            }
            OP_QUITAR_MANEJADOR => Ok(Instruccion::QuitarManejador),
            OP_RELANZAR => Ok(Instruccion::Relanzar),
            OP_PARA_AVANZAR => Ok(Instruccion::ParaAvanzar(InfoParaAvanzar {
                var_slot: u(self)?,
                var_global: b(self)?,
                fin_slot: u(self)?,
                fin_global: b(self)?,
                paso_slot: u(self)?,
                paso_global: b(self)?,
                salto_cuerpo: u(self)?,
                inclusivo: b(self)?,
            })),
            OP_CREAR_LISTA => Ok(Instruccion::CrearLista(u(self)?)),
            OP_INDICE => Ok(Instruccion::Indice),
            OP_LEER_ENTRADA => {
                let slot = u(self)?;
                let g = b(self)?;
                let n = u(self)?;
                let tiene = self.u8()? != 0;
                let mensaje = if tiene { Some(u(self)?) } else { None };
                Ok(Instruccion::LeerEntrada(slot, g, n, mensaje))
            }
            OP_INCREMENTAR => Ok(Instruccion::Incrementar(
                u(self)?,
                b(self)?,
                u(self)?,
                b(self)?,
            )),
            OP_SALTAR => Ok(Instruccion::Saltar(u(self)?)),
            OP_SALTAR_SI_FALSO => Ok(Instruccion::SaltarSiFalso(u(self)?)),
            OP_LLAMAR => Ok(Instruccion::Llamar(u(self)?, u(self)?)),
            OP_REGISTRAR_FUNCION => Ok(Instruccion::RegistrarFuncion(u(self)?)),
            OP_RETORNAR => Ok(Instruccion::Retornar),
            otro => Err(format!(
                "el paquete trae una instrucción rara (código {})",
                otro
            )),
        }
    }

    fn pedazo(&mut self) -> Result<Pedazo, String> {
        let mut p = Pedazo::nuevo();
        p.max_slots = self.u32()? as usize;
        let n_const = self.u32()? as usize;
        for _ in 0..n_const {
            p.constantes.push(self.valor()?);
        }
        let n_nombres = self.u32()? as usize;
        for _ in 0..n_nombres {
            p.nombres.push(self.texto()?);
        }
        let n_params = self.u32()? as usize;
        for _ in 0..n_params {
            p.parametros.push(self.texto()?);
        }
        let n_codigo = self.u32()? as usize;
        for _ in 0..n_codigo {
            p.codigo.push(self.instruccion()?);
        }
        Ok(p)
    }
}

// Convierte bytes ECBIN en un programa listo para correr.
pub fn cargar_programa(datos: &[u8]) -> Result<Programa, String> {
    let mut l = Lector::nuevo(datos);
    if datos.len() < 6 || &datos[0..5] != MARCA_ECBIN {
        return Err("eso no es un paquete de Eclipse (le falta la marca ECBIN)".to_string());
    }
    l.posicion = 5;
    let version = l.u8()?;
    if version != VERSION_ECBIN {
        return Err(format!(
            "ese paquete es de la versión {}, y este Eclipse solo entiende la versión {}",
            version, VERSION_ECBIN
        ));
    }
    let principal = l.pedazo()?;
    let n_func = l.u32()? as usize;
    let mut funciones: HashMap<String, Pedazo> = HashMap::new();
    for _ in 0..n_func {
        let nombre = l.texto()?;
        let pedazo = l.pedazo()?;
        funciones.insert(nombre, pedazo);
    }
    Ok(Programa {
        principal,
        funciones,
    })
}

// =============== EMPAQUETAR ===============

// Crea un ejecutable standalone: el propio programa `eclipse`
// con el bytecode de tu programa pegado al final.
// No necesita rustc, ni Rust, ni Cargo: solo Eclipse.
// Devuelve la ruta del ejecutable creado.
pub fn empaquetar(ruta_ec: &str, salida: Option<&str>) -> Result<PathBuf, String> {
    let fuente = fs::read_to_string(ruta_ec)
        .map_err(|_| format!("no encontré el archivo '{}'", ruta_ec))?;
    let base = Path::new(ruta_ec)
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| Path::new(".").to_path_buf());
    let programa = crate::motor::compilador::compilar(&fuente, base, ruta_ec)?;
    let paquete = guardar_programa(&programa);

    let ruta_path = Path::new(ruta_ec);
    let tallo = ruta_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("programa")
        .to_string();
    let dir = ruta_path
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| Path::new(".").to_path_buf());
    let nombre_salida = match salida {
        Some(s) => s.to_string(),
        None => {
            if cfg!(windows) {
                format!("{}.exe", tallo)
            } else {
                tallo
            }
        }
    };
    // Si la salida ya trae su propia carpeta (o es absoluta),
    // se usa tal cual; si es solo un nombre, va junto al .ec.
    let ruta_salida = {
        let p = Path::new(&nombre_salida);
        let tiene_carpeta = p.parent().map(|d| !d.as_os_str().is_empty()).unwrap_or(false);
        if tiene_carpeta {
            p.to_path_buf()
        } else {
            dir.join(nombre_salida)
        }
    };

    // El ejecutable de Eclipse que está corriendo ahora.
    let yo = env::current_exe()
        .map_err(|_| "no pude encontrar el programa eclipse que está corriendo".to_string())?;
    let mut yo_bytes = fs::read(&yo)
        .map_err(|_| "no pude leer el programa eclipse que está corriendo".to_string())?;
    // Pegar el paquete al final, con su largo y la marca.
    yo_bytes.extend_from_slice(&paquete);
    yo_bytes.extend_from_slice(&(paquete.len() as u64).to_le_bytes());
    yo_bytes.extend_from_slice(MARCA_PIE);
    fs::write(&ruta_salida, &yo_bytes)
        .map_err(|_| format!("no pude escribir '{}'", ruta_salida.display()))?;
    // En Linux y Mac, el archivo tiene que ser ejecutable.
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(&ruta_salida, fs::Permissions::from_mode(0o755));
    }
    Ok(ruta_salida)
}

// Mira si el propio ejecutable trae un paquete pegado al
// final. Si lo trae, devuelve el programa que trae dentro.
// Si no lo trae, devuelve None (es el Eclipse normal).
pub fn paquete_incrustado() -> Option<Programa> {
    let yo = env::current_exe().ok()?;
    let bytes = fs::read(&yo).ok()?;
    if bytes.len() < 16 {
        return None;
    }
    let pie = &bytes[bytes.len() - 16..];
    let largo_bytes: [u8; 8] = pie[0..8].try_into().ok()?;
    let largo = u64::from_le_bytes(largo_bytes) as usize;
    if &pie[8..16] != MARCA_PIE {
        return None;
    }
    if largo > bytes.len() - 16 {
        return None;
    }
    let inicio = bytes.len() - 16 - largo;
    let paquete = &bytes[inicio..inicio + largo];
    cargar_programa(paquete).ok()
}
