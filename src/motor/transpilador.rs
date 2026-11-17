// ============================================================
//  ECLIPSE 1.2 - El transpilador (compilador a código nativo)
//  Archivo: src/transpilador.rs
//
//  Qué hace:
//  Traduce tu programa de Eclipse a un programa en Rust
//  (un archivo main.rs) que usa la librería eclipse_rt.rs.
//  Después, `eclipse --exe` llama a `rustc` para convertir
//  ese Rust en un .exe de verdad (código nativo, sin
//  intérprete ni máquina virtual: tu programa corre solo).
//
//  Cómo funciona:
//  Cada orden de Eclipse se convierte en código Rust que
//  llama a las mismas funciones que usa el intérprete
//  (definir, obtener, asignar, llamar...). Por eso el .exe
//  hace exactamente lo mismo que el intérprete y la VM:
//  las mismas reglas de ownership, los mismos errores en
//  español, los mismos resultados.
//
//  No hay Garbage Collector: el .exe es Rust puro, y en
//  Rust cada valor tiene UN solo dueño y se libera cuando
//  deja de usarse. No hay un recolector corriendo en
//  segundo plano.
//
//  Para generar solo el código Rust (sin compilarlo):
//      eclipse --rust programa.ec
//  Para generar el .exe:
//      eclipse --exe programa.ec
// ============================================================

use crate::lenguaje::ast::{Declaracion, Expresion, OperadorBinario, OperadorUnario, ParteCadena};
use crate::lenguaje::lexer::Lexer;
use crate::lenguaje::parser::Parser;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

// La librería de ejecución, incrustada en el programa
// `eclipse` para que siempre esté disponible, estés donde
// estés. Se escribe junto al main.rs generado.
pub const RUNTIME_FUENTE: &str = include_str!("../../rt/eclipse_rt.rs");

// La información de un bucle que se está transpilando:
// a qué profundidad de ámbitos empezó, y si es un `para`
// (para que `seguir` corra el incremento).
struct InfoBucleT {
    profundidad_base: usize,
    // A qué profundidad de `intentar` se abrió: si un
    // `romper` está dentro de un `intentar` y el bucle se
    // abrió fuera de él, hay que avisar (Rompio) en vez
    // de hacer `break` dentro del ayudante.
    en_intento: usize,
    es_para: bool,
    // Si es un `para_cada`, `seguir` suma 1 a $indice
    // (no al paso del `para`).
    es_para_cada: bool,
    variable: String,
    // El `para` rápido guarda la variable que cuenta y el
    // paso en variables locales de Rust (no en el entorno):
    // sus nombres, para que `seguir` las incremente.
    local: String,
    local_paso: String,
}

pub struct Transpilador {
    // Cuántas funciones llevamos (para darles un nombre único).
    contador_fn: usize,
    // Nombres de variables temporales (__v0, __v1...).
    contador_temp: usize,
    // Cuántos ámbitos llevamos abiertos en la función actual.
    profundidad: usize,
    // Cuánta sangría llevamos en el código generado.
    indent: usize,
    bucles: Vec<InfoBucleT>,
    en_funcion: bool,
    // Cuántos `intentar` llevamos anidados (0 = ninguno).
    // Un `devolver`, `romper` o `seguir` dentro de un
    // `intentar` no salta directo: avisa con SalidaIntento.
    en_intento: usize,
    // Cuántos ayudantes de `intentar` llevamos (para
    // darles un nombre único: __intento_0, __intento_1...).
    contador_intento: usize,
    importados: HashSet<PathBuf>,
    directorio_base: PathBuf,
    // Líneas que hay que escribir ANTES de la orden actual:
    // son las llamadas anidadas que se izaron a temporales
    // (para que Rust no se queje de pedir prestado el
    // entorno dos veces a la vez). Ver transpilar_expresion.
    pre: Vec<String>,
    // Cuántas llamadas estamos dentro: si una llamada está
    // dentro de los argumentos de otra, hay que izarla.
    en_llamada: usize,
}

// Traduce un programa .ec entero a código Rust (el main.rs).
// `nombre` es el nombre del archivo, para ponerlo en los errores.
pub fn transpile(fuente: &str, base: PathBuf, nombre: &str) -> Result<String, String> {
    let tokens = Lexer::nuevo(fuente)
        .tokenizar()
        .map_err(|e| format!("{}: {}", nombre, e))?;
    let declaraciones = Parser::nuevo(tokens)
        .parsear()
        .map_err(|e| format!("{}: {}", nombre, e))?;
    // El verificador de préstamos te regaña ANTES de correr.
    crate::motor::verificador::verificar(&declaraciones, &base)
        .map_err(|e| format!("{}: {}", nombre, e.texto()))?;
    let mut t = Transpilador {
        contador_fn: 0,
        contador_temp: 0,
        profundidad: 1,
        // Las órdenes de arriba van dentro de `fn correr`,
        // así que empiezan con una sangría.
        indent: 1,
        bucles: Vec::new(),
        en_funcion: false,
        en_intento: 0,
        contador_intento: 0,
        importados: HashSet::new(),
        directorio_base: base,
        pre: Vec::new(),
        en_llamada: 0,
    };
    let mut principal = String::new();
    let mut funciones = String::new();
    for d in &declaraciones {
        t.transpilar_declaracion(d, &mut principal, &mut funciones)?;
    }
    // El archivo Rust completo: la cabecera, el main, la
    // función correr() con tu programa, y tus funciones.
    let mut codigo = String::new();
    codigo.push_str("// Generado por Eclipse 1.2 - no lo edites a mano.\n");
    codigo.push_str("// Para compilarlo: rustc -O --edition 2021 main.rs -o programa\n");
    codigo.push_str("#![allow(unused, unused_mut, dead_code)]\n");
    codigo.push_str("mod eclipse_rt;\n");
    codigo.push_str("use eclipse_rt::*;\n");
    codigo.push_str("\nfn main() {\n");
    codigo.push_str("    if let Err(e) = correr() {\n");
    codigo.push_str("        eprintln!(\"Error: {}\", e);\n");
    codigo.push_str("        std::process::exit(1);\n");
    codigo.push_str("    }\n");
    codigo.push_str("}\n");
    codigo.push_str("\nfn correr() -> Result<(), String> {\n");
    codigo.push_str("    let mut entorno = Entorno::nuevo();\n");
    codigo.push_str("    entorno.entrar();\n");
    codigo.push_str(&principal);
    codigo.push_str("    entorno.salir();\n");
    codigo.push_str("    Ok(())\n");
    codigo.push_str("}\n");
    codigo.push_str(&funciones);
    Ok(codigo)
}

impl Transpilador {
    // Un nombre nuevo para una variable temporal (__v0, __v1...).
    fn temp(&mut self) -> String {
        let n = self.contador_temp;
        self.contador_temp += 1;
        format!("__v{}", n)
    }

    // Escribe una línea en el buffer, con su sangría.
    fn linea(&self, buf: &mut String, texto: &str) {
        for _ in 0..self.indent {
            buf.push_str("    ");
        }
        buf.push_str(texto);
        buf.push('\n');
    }

    // Abre un ámbito: entra y sube la sangría.
    fn entrar_ambito(&mut self, buf: &mut String) {
        self.linea(buf, "entorno.entrar();");
        self.profundidad += 1;
        self.indent += 1;
    }

    // Cierra un ámbito: sale y baja la sangría.
    fn salir_ambito(&mut self, buf: &mut String) {
        self.profundidad -= 1;
        self.indent -= 1;
        self.linea(buf, "entorno.salir();");
    }

    // Cierra ámbitos hasta llegar a una profundidad.
    // OJO: emite los `entorno.salir();` pero NO toca el
    // contador de profundidad. El contador lleva la cuenta
    // de los ámbitos en el camino normal (sin saltos), y el
    // `salir` que cierra cada bloque ya lo baja. Si aquí
    // también lo bajáramos, el contador se iría por debajo
    // y los bloques de afuera cerrarían ámbitos de más.
    // En tiempo de ejecución, el `salir` que cierra el
    // bloque después de un salto es código muerto (el salto
    // ya salió de ahí), así que nada se cierra dos veces.
    fn salir_hasta(&mut self, buf: &mut String, nivel: usize) {
        let mut n = self.profundidad;
        while n > nivel {
            self.linea(buf, "entorno.salir();");
            n -= 1;
        }
    }

    // Escribe las líneas izadas (las llamadas anidadas)
    // antes de la orden actual, y vacía la lista.
    fn vaciar_pre(&mut self, buf: &mut String) {
        let lineas = std::mem::take(&mut self.pre);
        for l in lineas {
            self.linea(buf, &l);
        }
    }

    // El incremento de un `para_cada`: $indice = $indice + 1.
    // ($indice se definió fija, así que se vuelve a definir.)
    fn incremento_para_cada(&mut self, buf: &mut String) {
        let temp = self.temp();
        self.linea(
            buf,
            &format!(
                "let {} = aplicar_binario(entorno.obtener(\"$indice\")?, \"+\", Valor::Numero(1.0))?;",
                temp
            ),
        );
        self.linea(
            buf,
            &format!(
                "entorno.definir(\"$indice\".to_string(), {}, false);",
                temp
            ),
        );
    }

    // La condición de un `para` rápido, en Rust directo:
    // si el paso es fijo, es una comparación sola; si es una
    // variable, el signo del paso decide el sentido.
    // (Paso 0 = el bucle no corre nunca, como antes.)
    fn condicion_para_rust(
        inclusivo: bool,
        paso: &Option<Expresion>,
        local: &str,
        local_fin: &str,
        local_paso: &str,
    ) -> String {
        let paso_fijo: Option<f64> = match paso {
            None => Some(1.0),
            Some(Expresion::Numero(n)) => Some(*n),
            Some(Expresion::Unaria {
                operador: OperadorUnario::Negativo,
                derecha,
            }) => match &**derecha {
                Expresion::Numero(n) => Some(-*n),
                _ => None,
            },
            _ => None,
        };
        if let Some(p) = paso_fijo {
            if p == 0.0 {
                return "false".to_string();
            }
            if p > 0.0 {
                return if inclusivo {
                    format!("{} <= {}", local, local_fin)
                } else {
                    format!("{} < {}", local, local_fin)
                };
            }
            return if inclusivo {
                format!("{} >= {}", local, local_fin)
            } else {
                format!("{} > {}", local, local_fin)
            };
        }
        if inclusivo {
            format!(
                "(({} > 0.0 && {} <= {}) || ({} < 0.0 && {} >= {}))",
                local_paso, local, local_fin, local_paso, local, local_fin
            )
        } else {
            format!(
                "(({} > 0.0 && {} < {}) || ({} < 0.0 && {} > {}))",
                local_paso, local, local_fin, local_paso, local, local_fin
            )
        }
    }

    // El incremento de un `para` rápido: suma en la variable
    // local de Rust y sincroniza el entorno (para que el
    // cuerpo vea el valor nuevo en la siguiente vuelta).
    fn incremento_para_rapido(
        &mut self,
        buf: &mut String,
        variable: &str,
        local: &str,
        local_paso: &str,
    ) {
        self.linea(buf, &format!("{} += {};", local, local_paso));
        self.linea(
            buf,
            &format!(
                "entorno.definir({}, Valor::Numero({}), false);",
                nombre_rust(variable),
                local
            ),
        );
    }

    // Traduce una lista de órdenes (un bloque) a código Rust.
    fn transpilar_bloque(
        &mut self,
        cuerpo: &[Declaracion],
        buf: &mut String,
        fns: &mut String,
    ) -> Result<(), String> {
        self.entrar_ambito(buf);
        for d in cuerpo {
            self.transpilar_declaracion(d, buf, fns)?;
        }
        self.salir_ambito(buf);
        Ok(())
    }

    // Traduce una sola orden.
    fn transpilar_declaracion(
        &mut self,
        d: &Declaracion,
        buf: &mut String,
        fns: &mut String,
    ) -> Result<(), String> {
        match d {
            Declaracion::Sea {
                nombre,
                mutable,
                tipo,
                valor,
                ..
            } => {
                let temp = self.temp();
                match valor {
                    Some(e) => {
                        let codigo = self.transpilar_expresion(e)?;
                        self.vaciar_pre(buf);
                        self.linea(buf, &format!("let {} = {};", temp, codigo));
                    }
                    None => {
                        self.linea(buf, &format!("let {} = Valor::Nulo;", temp));
                    }
                }
                // El tipo se revisa antes de crear la variable.
                if let Some(t) = tipo {
                    if valor.is_some() {
                        self.linea(
                            buf,
                            &format!(
                                "if {}.tipo() != {} {{ return Err(format!(\"la variable '{{}}' debe ser {{}}, pero le diste {{}}\", {}, {}, {}.tipo())); }}",
                                temp,
                                cadena_rust(t),
                                cadena_rust(nombre),
                                cadena_rust(t),
                                temp
                            ),
                        );
                    }
                }
                self.linea(
                    buf,
                    &format!(
                        "entorno.definir({}, {}, {});",
                        nombre_rust(nombre),
                        temp,
                        mutable
                    ),
                );
                // Ownership: si el valor venía de una variable
                // con texto, esa variable se mueve a la nueva.
                if let Some(Expresion::Variable(fuente)) = valor {
                    if fuente != nombre {
                        let destino = format!("a la variable '{}'", nombre);
                        self.linea(
                            buf,
                            &format!(
                                "entorno.mover({}, {})?;",
                                cadena_rust(fuente),
                                cadena_rust(&destino)
                            ),
                        );
                    }
                }
                Ok(())
            }
            Declaracion::Asignacion { nombre, valor, .. } => {
                // Atajo: `x = agregar(x, v)` mete el valor en su
                // sitio, sin clonar la lista entera (igual que
                // en el intérprete y en la máquina virtual).
                if let Expresion::Llamada {
                    nombre: f,
                    argumentos,
                } = valor
                {
                    if f == "agregar" && argumentos.len() == 2 {
                        if let Expresion::Variable(fuente) = &argumentos[0] {
                            if fuente == nombre {
                                let temp = self.temp();
                                let codigo =
                                    self.transpilar_expresion(&argumentos[1])?;
                                self.vaciar_pre(buf);
                                self.linea(buf, &format!("let {} = {};", temp, codigo));
                                self.linea(
                                    buf,
                                    &format!(
                                        "entorno.agregar_en_sitio({}, {})?;",
                                        cadena_rust(nombre),
                                        temp
                                    ),
                                );
                                return Ok(());
                            }
                        }
                    }
                }
                // El valor va primero a una temporal: si es una
                // llamada, no puede ir dentro de entorno.asignar
                // (Rust no deja pedir prestado el entorno dos
                // veces a la vez).
                let temp = self.temp();
                let codigo = self.transpilar_expresion(valor)?;
                self.vaciar_pre(buf);
                self.linea(buf, &format!("let {} = {};", temp, codigo));
                self.linea(
                    buf,
                    &format!(
                        "entorno.asignar({}, {})?;",
                        cadena_rust(nombre),
                        temp
                    ),
                );
                // Ownership: si el valor venía de otra variable
                // con texto, esa variable se mueve a esta.
                if let Expresion::Variable(fuente) = valor {
                    if fuente != nombre {
                        let destino = format!("a la variable '{}'", nombre);
                        self.linea(
                            buf,
                            &format!(
                                "entorno.mover({}, {})?;",
                                cadena_rust(fuente),
                                cadena_rust(&destino)
                            ),
                        );
                    }
                }
                Ok(())
            }
            Declaracion::AsignacionCompuesta {
                nombre,
                operador,
                valor,
                ..
            } => {
                // x += 5 es lo mismo que x = x + 5.
                let temp_a = self.temp();
                let temp_b = self.temp();
                self.linea(
                    buf,
                    &format!(
                        "let {} = entorno.obtener({})?;",
                        temp_a,
                        cadena_rust(nombre)
                    ),
                );
                let codigo = self.transpilar_expresion(valor)?;
                self.vaciar_pre(buf);
                self.linea(buf, &format!("let {} = {};", temp_b, codigo));
                self.linea(
                    buf,
                    &format!(
                        "entorno.asignar({}, aplicar_binario({}, {}, {})?)?;",
                        cadena_rust(nombre),
                        temp_a,
                        cadena_rust(simbolo_operador(operador)),
                        temp_b
                    ),
                );
                Ok(())
            }
            Declaracion::Constante {
                nombre,
                tipo,
                valor,
                ..
            } => {
                // Una constante: se calcula su valor y se guarda
                // con definir_constante (nunca se puede cambiar).
                let temp = self.temp();
                let codigo = self.transpilar_expresion(valor)?;
                self.vaciar_pre(buf);
                self.linea(buf, &format!("let {} = {};", temp, codigo));
                if let Some(t) = tipo {
                    self.linea(
                        buf,
                        &format!(
                            "if {}.tipo() != {} {{ return Err(format!(\"la constante '{{}}' debe ser {{}}, pero le diste {{}}\", {}, {}, {}.tipo())); }}",
                            temp,
                            cadena_rust(t),
                            cadena_rust(nombre),
                            cadena_rust(t),
                            temp
                        ),
                    );
                }
                self.linea(
                    buf,
                    &format!(
                        "entorno.definir_constante({}, {});",
                        nombre_rust(nombre),
                        temp
                    ),
                );
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
                // largo ($largo) y el índice ($indice). La
                // variable del bucle se define en cada vuelta
                // con $lista[$indice].
                self.entrar_ambito(buf);
                // La lista va primero a una temporal: si es una
                // llamada, no puede ir dentro de
                // entorno.definir (dos préstamos a la vez).
                let temp_lista = self.temp();
                let codigo_lista = self.transpilar_expresion(lista)?;
                self.vaciar_pre(buf);
                self.linea(buf, &format!("let {} = {};", temp_lista, codigo_lista));
                self.linea(
                    buf,
                    &format!(
                        "entorno.definir(\"$lista\".to_string(), {}, false);",
                        temp_lista
                    ),
                );
                // `para_cada` necesita una lista: si no lo es,
                // se dice con el mismo mensaje que el intérprete
                // y que la máquina virtual.
                let temp_tipo = self.temp();
                self.linea(
                    buf,
                    &format!("let {} = entorno.obtener(\"$lista\")?;", temp_tipo),
                );
                self.linea(
                    buf,
                    &format!(
                        "if {0}.tipo() != \"lista\" {{ return Err(format!(\"'para_cada' necesita una lista, no {{}}\", {0}.tipo())); }}",
                        temp_tipo
                    ),
                );
                let temp_largo = self.temp();
                self.linea(
                    buf,
                    &format!(
                        "let {} = entorno.llamar(\"largo\", vec![entorno.obtener(\"$lista\")?])?;",
                        temp_largo
                    ),
                );
                self.linea(
                    buf,
                    &format!(
                        "entorno.definir(\"$largo\".to_string(), {}, false);",
                        temp_largo
                    ),
                );
                self.linea(
                    buf,
                    "entorno.definir(\"$indice\".to_string(), Valor::Numero(0.0), false);",
                );
                // Mientras $indice < $largo.
                let codigo_cond = self.transpilar_expresion(
                    &Expresion::Binaria {
                        izquierda: Box::new(Expresion::Variable("$indice".to_string())),
                        operador: OperadorBinario::Menor,
                        derecha: Box::new(Expresion::Variable("$largo".to_string())),
                    },
                )?;
                self.vaciar_pre(buf);
                self.linea(buf, &format!("while ({}).es_verdadero() {{", codigo_cond));
                let base = self.profundidad;
                self.bucles.push(InfoBucleT {
                    profundidad_base: base,
                    es_para: false,
                    es_para_cada: true,
                    variable: variable.clone(),
                    local: String::new(),
                    local_paso: String::new(),
                    en_intento: self.en_intento,
                });
                // La variable del bucle vive en su propio ámbito.
                // El elemento va primero a una temporal, para no
                // pedir prestado el entorno dos veces a la vez.
                self.entrar_ambito(buf);
                let temp_elemento = self.temp();
                self.linea(
                    buf,
                    &format!(
                        "let {} = entorno.llamar(\"elemento\", vec![entorno.obtener(\"$lista\")?, entorno.obtener(\"$indice\")?])?;",
                        temp_elemento
                    ),
                );
                self.linea(
                    buf,
                    &format!(
                        "entorno.definir({}, {}, false);",
                        nombre_rust(variable),
                        temp_elemento
                    ),
                );
                self.transpilar_bloque(cuerpo, buf, fns)?;
                self.salir_ambito(buf);
                self.bucles.pop();
                // Incremento: $indice = $indice + 1
                self.incremento_para_cada(buf);
                self.linea(buf, "}");
                self.salir_ambito(buf);
                Ok(())
            }
            Declaracion::Leer { nombre, mensaje, .. } => {
                // Input: el entorno lee del teclado y guarda.
                // Con mensaje, lo muestra antes de leer.
                if let Some(m) = mensaje {
                    self.linea(buf, &format!("print!({});", cadena_rust(m)));
                    self.linea(buf, "let _ = std::io::Write::flush(&mut std::io::stdout());");
                }
                self.linea(
                    buf,
                    &format!("entorno.leer_entrada({})?;", cadena_rust(nombre)),
                );
                Ok(())
            }
            Declaracion::AsignarIndice { objetivo, indice, valor, .. } => {
                // lista[0] = 5; o mapa[21] = "FTP";. El objetivo
                // es una variable (el parser solo acepta eso).
                let nombre = match objetivo {
                    Expresion::Variable(n) => n.clone(),
                    _ => {
                        return Err(
                            "solo se puede cambiar un lugar de una lista o un mapa guardado en una variable"
                                .to_string(),
                        );
                    }
                };
                let codigo_indice = self.transpilar_expresion(indice)?;
                let codigo_valor = self.transpilar_expresion(valor)?;
                self.vaciar_pre(buf);
                self.linea(
                    buf,
                    &format!(
                        "entorno.asignar_indice({}, {}, {})?;",
                        cadena_rust(&nombre),
                        codigo_indice,
                        codigo_valor
                    ),
                );
                Ok(())
            }
            Declaracion::Intentar { cuerpo, falla, al_final, .. } => {
                // Un `intentar` se convierte en dos funciones
                // Rust: el cuerpo (que devuelve Bien, Salio,
                // Rompio o Siguio, o un error) y una envoltura
                // que abre un ámbito, llama al cuerpo y lo
                // cierra pase lo que pase (así no se escapa
                // ninguna variable si el cuerpo falla a la
                // mitad y el error se atrapa aquí).
                let id = self.contador_intento;
                self.contador_intento += 1;
                let nombre_ayudante = format!("__intento_{}", id);
                let nombre_cuerpo = format!("{}_cuerpo", nombre_ayudante);
                // Guardar el estado y entrar al ayudante.
                let profundidad_vieja = self.profundidad;
                let bucles_viejos = std::mem::take(&mut self.bucles);
                let indent_viejo = self.indent;
                let en_intento_viejo = self.en_intento;
                self.profundidad = 1;
                self.indent = 0;
                self.en_intento += 1;
                // El cuerpo: una función Rust normal.
                let mut cuerpo_fn = String::new();
                self.linea(
                    &mut cuerpo_fn,
                    &format!(
                        "fn {}(entorno: &mut Entorno) -> Result<SalidaIntento, String> {{",
                        nombre_cuerpo
                    ),
                );
                self.indent = 1;
                for d in cuerpo {
                    self.transpilar_declaracion(d, &mut cuerpo_fn, fns)?;
                }
                self.linea(&mut cuerpo_fn, "Ok(SalidaIntento::Bien)");
                self.indent = 0;
                self.linea(&mut cuerpo_fn, "}");
                fns.push_str(&cuerpo_fn);
                // La envoltura: abre el ámbito, llama al cuerpo
                // y lo cierra siempre (también si da error).
                let mut env_fn = String::new();
                self.linea(
                    &mut env_fn,
                    &format!(
                        "fn {}(entorno: &mut Entorno) -> Result<SalidaIntento, String> {{",
                        nombre_ayudante
                    ),
                );
                self.indent = 1;
                self.linea(&mut env_fn, "entorno.entrar();");
                self.linea(
                    &mut env_fn,
                    &format!("let __r = {}(entorno);", nombre_cuerpo),
                );
                self.linea(&mut env_fn, "entorno.salir();");
                self.linea(&mut env_fn, "__r");
                self.indent = 0;
                self.linea(&mut env_fn, "}");
                fns.push_str(&env_fn);
                // Restaurar el estado de afuera.
                self.profundidad = profundidad_vieja;
                self.bucles = bucles_viejos;
                self.indent = indent_viejo;
                self.en_intento = en_intento_viejo;
                // Llamar al ayudante y decidir qué hacer con
                // lo que pasó: bien, error, devolver, romper
                // o seguir. El `al_final` corre en todos los
                // casos (también antes de propagar).
                let error_temp = self.temp();
                let valor_temp = self.temp();
                // En `correr` el entorno es un valor (hay que
                // pedirlo prestado con &mut); dentro de una
                // función o de otro `intentar` ya es &mut.
                let arg_entorno = if self.en_funcion || self.en_intento > 0 {
                    "entorno"
                } else {
                    "&mut entorno"
                };
                self.linea(
                    buf,
                    &format!("match {}({}) {{", nombre_ayudante, arg_entorno),
                );
                self.indent += 1;
                // Todo bien.
                self.linea(buf, "Ok(SalidaIntento::Bien) => {");
                self.indent += 1;
                if let Some(final_cuerpo) = al_final {
                    self.transpilar_bloque(final_cuerpo, buf, fns)?;
                }
                self.indent -= 1;
                self.linea(buf, "}");
                // Hubo un error.
                self.linea(buf, &format!("Err({}) => {{", error_temp));
                self.indent += 1;
                if let Some((variable, cuerpo_falla)) = falla {
                    self.entrar_ambito(buf);
                    if let Some(nombre) = variable {
                        self.linea(
                            buf,
                            &format!(
                                "entorno.definir({}, Valor::Texto({}), false);",
                                nombre_rust(nombre),
                                error_temp
                            ),
                        );
                    }
                    for d in cuerpo_falla {
                        self.transpilar_declaracion(d, buf, fns)?;
                    }
                    self.salir_ambito(buf);
                }
                if let Some(final_cuerpo) = al_final {
                    self.transpilar_bloque(final_cuerpo, buf, fns)?;
                }
                // Sin si_falla, el error sigue su camino.
                if falla.is_none() {
                    self.linea(buf, &format!("return Err({});", error_temp));
                }
                self.indent -= 1;
                self.linea(buf, "}");
                // Salió con un `devolver`.
                self.linea(
                    buf,
                    &format!("Ok(SalidaIntento::Salio({})) => {{", valor_temp),
                );
                self.indent += 1;
                if let Some(final_cuerpo) = al_final {
                    self.transpilar_bloque(final_cuerpo, buf, fns)?;
                }
                if self.en_intento > 0 {
                    self.linea(
                        buf,
                        &format!("return Ok(SalidaIntento::Salio({}));", valor_temp),
                    );
                } else if self.en_funcion {
                    self.linea(buf, "entorno.terminar_prestamos()?;");
                    self.linea(buf, &format!("return Ok({});", valor_temp));
                } else {
                    self.linea(buf, "return Ok(());");
                }
                self.indent -= 1;
                self.linea(buf, "}");
                // Salió con un `romper`.
                self.linea(buf, "Ok(SalidaIntento::Rompio) => {");
                self.indent += 1;
                if let Some(final_cuerpo) = al_final {
                    self.transpilar_bloque(final_cuerpo, buf, fns)?;
                }
                if let Some(b) = self.bucles.last() {
                    let base = b.profundidad_base;
                    self.salir_hasta(buf, base);
                    self.linea(buf, "break;");
                } else if self.en_intento > 0 {
                    self.linea(buf, "return Ok(SalidaIntento::Rompio);");
                }
                self.indent -= 1;
                self.linea(buf, "}");
                // Salió con un `seguir`.
                self.linea(buf, "Ok(SalidaIntento::Siguio) => {");
                self.indent += 1;
                if let Some(final_cuerpo) = al_final {
                    self.transpilar_bloque(final_cuerpo, buf, fns)?;
                }
                if let Some(b) = self.bucles.last() {
                    let (base, es_para, es_para_cada, variable, local, local_paso) = (
                        b.profundidad_base,
                        b.es_para,
                        b.es_para_cada,
                        b.variable.clone(),
                        b.local.clone(),
                        b.local_paso.clone(),
                    );
                    self.salir_hasta(buf, base);
                    if es_para_cada {
                        self.incremento_para_cada(buf);
                    } else if es_para {
                        self.incremento_para_rapido(buf, &variable, &local, &local_paso);
                    }
                    self.linea(buf, "continue;");
                } else if self.en_intento > 0 {
                    self.linea(buf, "return Ok(SalidaIntento::Siguio);");
                }
                self.indent -= 1;
                self.linea(buf, "}");
                self.indent -= 1;
                self.linea(buf, "}");
                Ok(())
            }
            Declaracion::Incrementar { nombre, subir, .. } => {
                // puntos++ / puntos--: el entorno suma o resta 1.
                self.linea(
                    buf,
                    &format!(
                        "entorno.incrementar({}, {})?;",
                        cadena_rust(nombre),
                        subir
                    ),
                );
                Ok(())
            }
            Declaracion::Expresion { expresion: e, .. } => {
                let codigo = self.transpilar_expresion(e)?;
                self.vaciar_pre(buf);
                self.linea(buf, &format!("{};", codigo));
                Ok(())
            }
            Declaracion::Si {
                condicion,
                entonces,
                sino,
                ..
            } => {
                let codigo = self.transpilar_expresion(condicion)?;
                self.vaciar_pre(buf);
                self.linea(buf, &format!("if ({}).es_verdadero() {{", codigo));
                self.transpilar_bloque(entonces, buf, fns)?;
                if let Some(sino) = sino {
                    self.linea(buf, "} else {");
                    self.transpilar_bloque(sino, buf, fns)?;
                }
                self.linea(buf, "}");
                Ok(())
            }
            Declaracion::Mientras { condicion, cuerpo, .. } => {
                let codigo = self.transpilar_expresion(condicion)?;
                self.vaciar_pre(buf);
                let base = self.profundidad;
                self.linea(buf, &format!("while ({}).es_verdadero() {{", codigo));
                self.bucles.push(InfoBucleT {
                    profundidad_base: base,
                    es_para: false,
                    es_para_cada: false,
                    variable: String::new(),
                    local: String::new(),
                    local_paso: String::new(),
                    en_intento: self.en_intento,
                });
                self.transpilar_bloque(cuerpo, buf, fns)?;
                self.bucles.pop();
                self.linea(buf, "}");
                Ok(())
            }
            Declaracion::Repetir { cuerpo, condicion, .. } => {
                // Corre el cuerpo una vez, y luego repite
                // mientras la condición sea verdadera. Se hace
                // con un `while` y una bandera de primera vez.
                let bandera = self.temp();
                self.linea(buf, &format!("let mut {} = true;", bandera));
                let codigo = self.transpilar_expresion(condicion)?;
                self.vaciar_pre(buf);
                self.linea(
                    buf,
                    &format!("while {} || ({}).es_verdadero() {{", bandera, codigo),
                );
                self.linea(buf, &format!("{} = false;", bandera));
                let base = self.profundidad;
                self.bucles.push(InfoBucleT {
                    profundidad_base: base,
                    es_para: false,
                    es_para_cada: false,
                    variable: String::new(),
                    local: String::new(),
                    local_paso: String::new(),
                    en_intento: self.en_intento,
                });
                self.transpilar_bloque(cuerpo, buf, fns)?;
                self.bucles.pop();
                self.linea(buf, "}");
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
                // El `para` rápido: la variable que cuenta, el
                // fin y el paso viven en variables locales de
                // Rust (no en el entorno), así que cada vuelta
                // es una suma y una comparación directas, sin
                // buscar nada en un mapa. El cuerpo sigue viendo
                // la variable en el entorno (se sincroniza en
                // cada vuelta), así que nada cambia por fuera.
                self.entrar_ambito(buf);
                let local = self.temp();
                let local_fin = self.temp();
                let local_paso = self.temp();
                // Cada valor va primero a una temporal: si es
                // una llamada, no puede ir dentro de
                // entorno.definir (dos préstamos a la vez).
                let temp_inicio = self.temp();
                let codigo_inicio = self.transpilar_expresion(inicio)?;
                self.vaciar_pre(buf);
                self.linea(buf, &format!("let {} = {};", temp_inicio, codigo_inicio));
                self.linea(
                    buf,
                    &format!(
                        "let mut {} = match {} {{ Valor::Numero(n) => n, _ => return Err(\"el inicio del 'para' debe ser un numero\".to_string()) }};",
                        local, temp_inicio
                    ),
                );
                let temp_fin = self.temp();
                let codigo_fin = self.transpilar_expresion(fin)?;
                self.vaciar_pre(buf);
                self.linea(buf, &format!("let {} = {};", temp_fin, codigo_fin));
                self.linea(
                    buf,
                    &format!(
                        "let {} = match {} {{ Valor::Numero(n) => n, _ => return Err(\"el fin del 'para' debe ser un numero\".to_string()) }};",
                        local_fin, temp_fin
                    ),
                );
                match paso {
                    Some(e) => {
                        let temp_paso = self.temp();
                        let codigo_paso = self.transpilar_expresion(e)?;
                        self.vaciar_pre(buf);
                        self.linea(buf, &format!("let {} = {};", temp_paso, codigo_paso));
                        self.linea(
                            buf,
                            &format!(
                                "let {} = match {} {{ Valor::Numero(n) => n, _ => return Err(\"el paso del 'para' debe ser un numero\".to_string()) }};",
                                local_paso, temp_paso
                            ),
                        );
                    }
                    None => {
                        self.linea(buf, &format!("let {} = 1.0;", local_paso));
                    }
                }
                // La primera sincronización: el cuerpo ve la
                // variable con su valor inicial.
                self.linea(
                    buf,
                    &format!(
                        "entorno.definir({}, Valor::Numero({}), false);",
                        nombre_rust(variable),
                        local
                    ),
                );
                let cond = Self::condicion_para_rust(*inclusivo, paso, &local, &local_fin, &local_paso);
                self.linea(buf, &format!("while {} {{", cond));
                let base = self.profundidad;
                self.bucles.push(InfoBucleT {
                    profundidad_base: base,
                    es_para: true,
                    es_para_cada: false,
                    variable: variable.clone(),
                    local: local.clone(),
                    local_paso: local_paso.clone(),
                    en_intento: self.en_intento,
                });
                self.transpilar_bloque(cuerpo, buf, fns)?;
                self.bucles.pop();
                self.incremento_para_rapido(buf, variable, &local, &local_paso);
                self.linea(buf, "}");
                self.salir_ambito(buf);
                Ok(())
            }
            Declaracion::Segun { valor, brazos, .. } => {
                // El valor se guarda en una variable escondida
                // ($segun) y se compara brazo por brazo.
                self.entrar_ambito(buf);
                let temp = self.temp();
                let codigo = self.transpilar_expresion(valor)?;
                self.vaciar_pre(buf);
                self.linea(buf, &format!("let {} = {};", temp, codigo));
                self.linea(
                    buf,
                    &format!("entorno.definir(\"$segun\".to_string(), {}, false);", temp),
                );
                let mut primero = true;
                for brazo in brazos {
                    let llave = if primero {
                        primero = false;
                        "if"
                    } else {
                        "} else if"
                    };
                    match &brazo.patron {
                        Some(patron) => {
                            let codigo_patron = self.transpilar_expresion(patron)?;
                            self.vaciar_pre(buf);
                            self.linea(
                                buf,
                                &format!(
                                    "{} entorno.obtener(\"$segun\")? == {} {{",
                                    llave, codigo_patron
                                ),
                            );
                        }
                        None => {
                            self.linea(buf, "} else {");
                        }
                    }
                    self.transpilar_bloque(&brazo.cuerpo, buf, fns)?;
                }
                if !brazos.is_empty() {
                    self.linea(buf, "}");
                }
                self.salir_ambito(buf);
                Ok(())
            }
            Declaracion::Funcion {
                nombre,
                parametros,
                cuerpo,
                ..
            } => {
                // La función se registra aquí (para poder
                // llamarla a partir de este punto), y su código
                // Rust se escribe aparte, al final del archivo.
                let id = self.contador_fn;
                self.contador_fn += 1;
                let nombre_fn = format!("eclipse_fn_{}", id);
                let params_rust: Vec<String> =
                    parametros.iter().map(|p| nombre_rust(p)).collect();
                self.linea(
                    buf,
                    &format!(
                        "entorno.definir_funcion({}, {}, vec![{}]);",
                        cadena_rust(nombre),
                        nombre_fn,
                        params_rust.join(", ")
                    ),
                );
                // Guardar el estado y entrar a la función.
                let profundidad_vieja = self.profundidad;
                let bucles_viejos = std::mem::take(&mut self.bucles);
                let indent_viejo = self.indent;
                let en_funcion_viejo = self.en_funcion;
                let en_intento_viejo = self.en_intento;
                self.profundidad = 1;
                self.indent = 0;
                self.en_funcion = true;
                self.en_intento = 0;
                let mut cuerpo_fn = String::new();
                self.linea(
                    &mut cuerpo_fn,
                    &format!(
                        "fn {}(entorno: &mut Entorno, args: Vec<Valor>, mutables: Vec<String>, prestados: Vec<String>) -> Result<Valor, String> {{",
                        nombre_fn
                    ),
                );
                self.indent = 1;
                self.linea(&mut cuerpo_fn, "entorno.entrar();");
                self.linea(&mut cuerpo_fn, "let mut args = args.into_iter();");
                for param in parametros {
                    // Un dato prestado (& o &mut) se define como
                    // prestado: se puede mirar (y cambiar, si es
                    // &mut), pero no se puede mover a otro dueño
                    // mientras corre la función.
                    self.linea(
                        &mut cuerpo_fn,
                        &format!(
                            "if mutables.iter().any(|m| m == {}) {{",
                            cadena_rust(param)
                        ),
                    );
                    self.indent += 1;
                    self.linea(
                        &mut cuerpo_fn,
                        &format!(
                            "entorno.definir_prestado({}, args.next().unwrap_or(Valor::Nulo), true);",
                            nombre_rust(param)
                        ),
                    );
                    self.indent -= 1;
                    self.linea(
                        &mut cuerpo_fn,
                        &format!(
                            "}} else if prestados.iter().any(|m| m == {}) {{",
                            cadena_rust(param)
                        ),
                    );
                    self.indent += 1;
                    self.linea(
                        &mut cuerpo_fn,
                        &format!(
                            "entorno.definir_prestado({}, args.next().unwrap_or(Valor::Nulo), false);",
                            nombre_rust(param)
                        ),
                    );
                    self.indent -= 1;
                    self.linea(&mut cuerpo_fn, "} else {");
                    self.indent += 1;
                    self.linea(
                        &mut cuerpo_fn,
                        &format!(
                            "entorno.definir({}, args.next().unwrap_or(Valor::Nulo), false);",
                            nombre_rust(param)
                        ),
                    );
                    self.indent -= 1;
                    self.linea(&mut cuerpo_fn, "}");
                }
                for d in cuerpo {
                    self.transpilar_declaracion(d, &mut cuerpo_fn, fns)?;
                }
                self.linea(&mut cuerpo_fn, "entorno.terminar_prestamos()?;");
                self.linea(&mut cuerpo_fn, "Ok(Valor::Nulo)");
                self.indent = 0;
                self.linea(&mut cuerpo_fn, "}");
                fns.push_str(&cuerpo_fn);
                // Restaurar el estado de afuera.
                self.profundidad = profundidad_vieja;
                self.bucles = bucles_viejos;
                self.indent = indent_viejo;
                self.en_funcion = en_funcion_viejo;
                self.en_intento = en_intento_viejo;
                Ok(())
            }
            Declaracion::Devolver { valor, .. } => {
                if !self.en_funcion {
                    return Err("no puedes usar 'devolver' fuera de una función".to_string());
                }
                let temp = self.temp();
                match valor {
                    Some(e) => {
                        let codigo = self.transpilar_expresion(e)?;
                        self.vaciar_pre(buf);
                        self.linea(buf, &format!("let {} = {};", temp, codigo));
                    }
                    None => {
                        self.linea(buf, &format!("let {} = Valor::Nulo;", temp));
                    }
                }
                // Cierra los ámbitos anidados (el de la función
                // lo cierra terminar_prestamos).
                self.salir_hasta(buf, 1);
                if self.en_intento > 0 {
                    // Dentro de un `intentar`: no se termina la
                    // función, se avisa con Salio. El ayudante
                    // no tiene marco de préstamos propio, así
                    // que no se llama a terminar_prestamos (eso
                    // tocaría los préstamos de la función de
                    // afuera).
                    self.linea(
                        buf,
                        &format!("return Ok(SalidaIntento::Salio({}));", temp),
                    );
                } else {
                    self.linea(buf, "entorno.terminar_prestamos()?;");
                    self.linea(buf, &format!("return Ok({});", temp));
                }
                Ok(())
            }
            Declaracion::Romper { .. } => {
                // ¿El bucle de adentro se abrió dentro de este
                // `intentar` (o fuera)? Si se abrió aquí dentro,
                // es un `break` normal. Si el bucle está afuera
                // del `intentar`, hay que salir del ayudante
                // avisando (Rompio) para que el que llamó haga
                // el `break` después del `al_final`.
                let dentro = match self.bucles.last() {
                    Some(b) => b.en_intento == self.en_intento,
                    None => false,
                };
                if dentro {
                    let base = self.bucles.last().unwrap().profundidad_base;
                    self.salir_hasta(buf, base);
                    self.linea(buf, "break;");
                } else if self.en_intento > 0 {
                    self.salir_hasta(buf, 1);
                    self.linea(buf, "return Ok(SalidaIntento::Rompio);");
                } else {
                    return Err("no puedes usar 'romper' fuera de un bucle".to_string());
                }
                Ok(())
            }
            Declaracion::Seguir { .. } => {
                let bucle_dentro = match self.bucles.last() {
                    Some(b) if b.en_intento == self.en_intento => Some((
                        b.profundidad_base,
                        b.es_para,
                        b.es_para_cada,
                        b.variable.clone(),
                        b.local.clone(),
                        b.local_paso.clone(),
                    )),
                    _ => None,
                };
                if let Some((base, es_para, es_para_cada, variable, local, local_paso)) =
                    bucle_dentro
                {
                    self.salir_hasta(buf, base);
                    // En un `para`, `seguir` corre el incremento
                    // antes de volver a la condición. En un
                    // `para_cada`, suma 1 a $indice.
                    if es_para_cada {
                        self.incremento_para_cada(buf);
                    } else if es_para {
                        self.incremento_para_rapido(buf, &variable, &local, &local_paso);
                    }
                    self.linea(buf, "continue;");
                } else if self.en_intento > 0 {
                    // El bucle está afuera del `intentar`: avisar
                    // con Siguio para que el que llamó haga el
                    // incremento y el `continue` después del
                    // `al_final`.
                    self.salir_hasta(buf, 1);
                    self.linea(buf, "return Ok(SalidaIntento::Siguio);");
                } else {
                    return Err("no puedes usar 'seguir' fuera de un bucle".to_string());
                }
                Ok(())
            }
            Declaracion::Importar { ruta, .. } => {
                self.transpilar_importar(ruta, buf, fns)?;
                Ok(())
            }
            Declaracion::Usar { nombre, .. } => {
                let ruta = crate::herramientas::bibliotecas::resolver_biblioteca(nombre)?;
                let ruta = ruta.to_string_lossy().to_string();
                self.transpilar_importar(&ruta, buf, fns)?;
                Ok(())
            }
            Declaracion::Bloque { declaraciones: cuerpo, .. } => self.transpilar_bloque(cuerpo, buf, fns),
        }
    }

    // Carga otro archivo .ec y traduce sus órdenes aquí mismo.
    // Si ya se importó antes, no lo vuelve a traducir.
    fn transpilar_importar(
        &mut self,
        ruta: &str,
        buf: &mut String,
        fns: &mut String,
    ) -> Result<(), String> {
        let completa = if Path::new(ruta).is_absolute() {
            PathBuf::from(ruta)
        } else {
            self.directorio_base.join(ruta)
        };
        let canonica = completa
            .canonicalize()
            .map_err(|_| format!("no encontré el archivo '{}'", ruta))?;
        if self.importados.contains(&canonica) {
            return Ok(());
        }
        self.importados.insert(canonica.clone());
        let fuente = std::fs::read_to_string(&canonica)
            .map_err(|_| format!("no pude leer el archivo '{}'", ruta))?;
        let tokens = Lexer::nuevo(&fuente)
            .tokenizar()
            .map_err(|e| format!("{}: {}", ruta, e))?;
        let declaraciones = Parser::nuevo(tokens)
            .parsear()
            .map_err(|e| format!("{}: {}", ruta, e))?;
        let base_vieja = self.directorio_base.clone();
        if let Some(padre) = canonica.parent() {
            self.directorio_base = padre.to_path_buf();
        }
        for d in &declaraciones {
            self.transpilar_declaracion(d, buf, fns)?;
        }
        self.directorio_base = base_vieja;
        Ok(())
    }

    // Traduce una expresión a código Rust que da un Valor.
    fn transpilar_expresion(&mut self, e: &Expresion) -> Result<String, String> {
        match e {
            Expresion::Numero(n) => Ok(format!("Valor::Numero({:?})", n)),
            Expresion::Cadena(s) => Ok(format!("Valor::Texto({}.to_string())", cadena_rust(s))),
            Expresion::Logico(true) => Ok("Valor::Logico(true)".to_string()),
            Expresion::Logico(false) => Ok("Valor::Logico(false)".to_string()),
            Expresion::Nulo => Ok("Valor::Nulo".to_string()),
            Expresion::Variable(nombre) => {
                Ok(format!("entorno.obtener({})?", cadena_rust(nombre)))
            }
            Expresion::Agrupada(interior) => {
                let codigo = self.transpilar_expresion(interior)?;
                Ok(format!("({})", codigo))
            }
            Expresion::Unaria { operador, derecha } => {
                let codigo = self.transpilar_expresion(derecha)?;
                match operador {
                    OperadorUnario::Negativo => Ok(format!("negar({})?", codigo)),
                    OperadorUnario::No => Ok(format!("no_logico({})", codigo)),
                }
            }
            Expresion::Binaria {
                izquierda,
                operador,
                derecha,
            } => match operador {
                // `y`: si el primero es falso, ya no miro el segundo.
                // OJO: las llamadas del lado derecho se izan a
                // `pre` (por el préstamo del entorno), y si las
                // dejara ahí correrían ANTES de mirar el lado
                // izquierdo, rompiendo el atajo. Por eso las
                // líneas nuevas de `pre` se meten DENTRO de la
                // rama, para que solo corran si hace falta.
                // Y si este `y` va dentro de los datos de una
                // llamada, se iza ENTERO a `pre`: si quedara
                // dentro del `vec![...]` de la llamada, Rust no
                // dejaría pedir prestado el entorno dos veces.
                OperadorBinario::Y => {
                    let izq = self.transpilar_expresion(izquierda)?;
                    let n_pre = self.pre.len();
                    let der = self.transpilar_expresion(derecha)?;
                    let pre_der: Vec<String> = self.pre.split_off(n_pre);
                    let der = if pre_der.is_empty() {
                        format!("({}).es_verdadero()", der)
                    } else {
                        format!("{{ {} ({}).es_verdadero() }}", pre_der.join(" "), der)
                    };
                    let expr = format!(
                        "Valor::Logico({{ let __izq = {}; if __izq.es_verdadero() {{ {} }} else {{ false }} }})",
                        izq, der
                    );
                    if self.en_llamada > 0 {
                        let temp = self.temp();
                        self.pre.push(format!("let {} = {};", temp, expr));
                        Ok(temp)
                    } else {
                        Ok(expr)
                    }
                }
                // `o`: si el primero es verdadero, ya no miro el segundo.
                // (El mismo cuidado con las líneas izadas que en `y`.)
                OperadorBinario::O => {
                    let izq = self.transpilar_expresion(izquierda)?;
                    let n_pre = self.pre.len();
                    let der = self.transpilar_expresion(derecha)?;
                    let pre_der: Vec<String> = self.pre.split_off(n_pre);
                    let der = if pre_der.is_empty() {
                        format!("({}).es_verdadero()", der)
                    } else {
                        format!("{{ {} ({}).es_verdadero() }}", pre_der.join(" "), der)
                    };
                    let expr = format!(
                        "Valor::Logico({{ let __izq = {}; if __izq.es_verdadero() {{ true }} else {{ {} }} }})",
                        izq, der
                    );
                    if self.en_llamada > 0 {
                        let temp = self.temp();
                        self.pre.push(format!("let {} = {};", temp, expr));
                        Ok(temp)
                    } else {
                        Ok(expr)
                    }
                }
                _ => {
                    let izq = self.transpilar_expresion(izquierda)?;
                    let der = self.transpilar_expresion(derecha)?;
                    Ok(format!(
                        "aplicar_binario({}, {}, {})?",
                        izq,
                        cadena_rust(simbolo_operador(operador)),
                        der
                    ))
                }
            },
            Expresion::Llamada { nombre, argumentos } => {
                // &mut: anotar el préstamo antes de llamar.
                // &: anotar el préstamo solo para mirar.
                // Se hace dentro de un bloque, para que la
                // anotación corra antes que la llamada.
                // (Las funciones que trae Eclipse no escriben
                // de vuelta, así que con ellas no se anota.)
                let mut anotaciones: Vec<String> = Vec::new();
                if !es_builtin(nombre) {
                    for (i, arg) in argumentos.iter().enumerate() {
                        if let Expresion::PrestarMut(interior) = arg {
                            if let Expresion::Variable(fuente) = &**interior {
                                anotaciones.push(format!(
                                    "entorno.anotar_prestamo({}, {})?;",
                                    cadena_rust(fuente),
                                    i
                                ));
                            }
                        }
                        if let Expresion::Prestar(interior) = arg {
                            if let Expresion::Variable(fuente) = &**interior {
                                anotaciones.push(format!(
                                    "entorno.anotar_prestamo_lectura({}, {});",
                                    cadena_rust(fuente),
                                    i
                                ));
                            }
                        }
                    }
                }
                // Ownership: si le pasas una variable con texto
                // o lista a una función tuya, esa variable se
                // mueve a la función (ya no la puedes usar
                // después). Las funciones que trae Eclipse solo
                // miran, así que con ellas no se mueve nada.
                // El valor se guarda primero en una temporal
                // (para que la lectura corra antes que el
                // movimiento) y el mover se anota después de
                // calcular todos los datos, igual que en el
                // intérprete y en la máquina virtual.
                let mut partes: Vec<String> = Vec::new();
                let mut movidos: Vec<String> = Vec::new();
                self.en_llamada += 1;
                for arg in argumentos {
                    let codigo = self.transpilar_expresion(arg)?;
                    if !es_builtin(nombre) {
                        if let Expresion::Variable(fuente) = arg {
                            let temp = self.temp();
                            self.pre
                                .push(format!("let {} = {};", temp, codigo));
                            movidos.push(fuente.clone());
                            partes.push(temp);
                            continue;
                        }
                    }
                    partes.push(codigo);
                }
                self.en_llamada -= 1;
                for fuente in &movidos {
                    let destino = format!("a la función '{}'", nombre);
                    self.pre.push(format!(
                        "entorno.mover({}, {})?;",
                        cadena_rust(fuente),
                        cadena_rust(&destino)
                    ));
                }
                let llamada = format!(
                    "entorno.llamar({}, vec![{}])?",
                    cadena_rust(nombre),
                    partes.join(", ")
                );
                let llamada = if anotaciones.is_empty() {
                    llamada
                } else {
                    format!("{{ {} {} }}", anotaciones.join(" "), llamada)
                };
                // Si esta llamada está dentro de los argumentos
                // de otra llamada, hay que izarla a una temporal
                // antes: Rust no deja pedir prestado el entorno
                // dos veces a la vez (una llamada dentro de otra).
                if self.en_llamada > 0 {
                    let temp = self.temp();
                    self.pre
                        .push(format!("let {} = {};", temp, llamada));
                    Ok(temp)
                } else {
                    Ok(llamada)
                }
            }
            // "Hola, {nombre}": pega los trozos y los valores.
            Expresion::Interpolada(partes) => {
                let mut codigo = String::from("Valor::Texto({ let mut __s = String::new();");
                for parte in partes {
                    match parte {
                        ParteCadena::Texto(s) => {
                            codigo.push_str(&format!(" __s.push_str({});", cadena_rust(s)));
                        }
                        ParteCadena::Expresion(e) => {
                            let c = self.transpilar_expresion(e)?;
                            codigo.push_str(&format!(" __s.push_str(&({}).a_texto());", c));
                        }
                    }
                }
                codigo.push_str(" __s })");
                Ok(codigo)
            }
            // [1, 2, 3]: una lista con sus elementos.
            Expresion::Lista(elementos) => {
                let mut partes: Vec<String> = Vec::new();
                for e in elementos {
                    partes.push(self.transpilar_expresion(e)?);
                }
                Ok(format!("Valor::lista(vec![{}])", partes.join(", ")))
            }
            // {21: "FTP", 22: "SSH"}: un mapa con sus pares.
            Expresion::Mapa(pares) => {
                let mut partes: Vec<String> = Vec::new();
                for (clave, valor) in pares {
                    let codigo_clave = self.transpilar_expresion(clave)?;
                    let codigo_valor = self.transpilar_expresion(valor)?;
                    partes.push(format!("({}, {})", codigo_clave, codigo_valor));
                }
                Ok(format!("Valor::mapa(vec![{}])", partes.join(", ")))
            }
            // mi_lista[0]: lo mismo que elemento(mi_lista, 0).
            // Se iza a una temporal siempre: la llamada a
            // elemento pide prestado el entorno, y si quedara
            // dentro de otra llamada (o de otro índice, como
            // en a[0][1]), Rust no dejaría pedirlo dos veces.
            Expresion::Indice { lista, indice } => {
                let codigo_lista = self.transpilar_expresion(lista)?;
                let codigo_indice = self.transpilar_expresion(indice)?;
                let temp = self.temp();
                self.pre.push(format!(
                    "let {} = entorno.llamar(\"elemento\", vec![{}, {}])?;",
                    temp, codigo_lista, codigo_indice
                ));
                Ok(temp)
            }
            // &nombre: prestar, no mover. Solo se calcula el valor.
            Expresion::Prestar(interior) => self.transpilar_expresion(interior),
            // &mut nombre: prestar para cambiar. Solo se calcula
            // el valor; la anotación la hace la llamada.
            Expresion::PrestarMut(interior) => self.transpilar_expresion(interior),
        }
    }
}

// Un texto como literal de Rust, con sus escapes:
// "Hola, \"Kevin\"" se escribe con {:?}.
fn cadena_rust(s: &str) -> String {
    format!("{:?}", s)
}

// Un nombre de variable como String de Rust, para definir():
// "nombre".to_string()
fn nombre_rust(s: &str) -> String {
    format!("{}.to_string()", cadena_rust(s))
}

// El símbolo de un operador, para aplicar_binario.
fn simbolo_operador(op: &OperadorBinario) -> &'static str {
    match op {
        OperadorBinario::Sumar => "+",
        OperadorBinario::Restar => "-",
        OperadorBinario::Multiplicar => "*",
        OperadorBinario::Dividir => "/",
        OperadorBinario::Residuo => "%",
        OperadorBinario::Potencia => "**",
        OperadorBinario::Igual => "==",
        OperadorBinario::Diferente => "!=",
        OperadorBinario::Menor => "<",
        OperadorBinario::MenorIgual => "<=",
        OperadorBinario::Mayor => ">",
        OperadorBinario::MayorIgual => ">=",
        OperadorBinario::Y => "y",
        OperadorBinario::O => "o",
    }
}

// ¿Es una de las funciones que trae Eclipse? Esas solo
// MIRAN el texto que les pasas, no se lo quedan ni lo cambian.
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
