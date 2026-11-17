# Eclipse 1.2 — Tu lenguaje en español

Eclipse es un lenguaje de programación en español, hecho en Rust por Kevin y Leo.
Esta es la versión 1.2. Corre programas `.ec` de verdad, con compilador
nativo a `.exe`, máquina virtual por slots, depurador, servidor de lenguaje
(LSP) en Rust puro con diagnósticos, esquema, definición y referencias,
ownership avanzado como Rust, listas, `leer` como Input con mensaje,
proyectos con manifiesto (como Cargo), ejecutables standalone que no
necesitan Rust ni Cargo, y botón ▶ Ejecutar en VS Code. Sin Garbage
Collector. Y todo el proyecto es Rust puro: ya no hay Python ni Shell
ni JavaScript en las pruebas.
**Lo nuevo de la 1.2:** la sintaxis es **más completa y 100% propia**
(no es copia de Python: del ejemplo solo se tomó la idea de cómo se
ve un programa — un diccionario, funciones y atrapar errores): **mapas**
(diccionarios `{clave: valor}`), **`intentar` / `si_falla` /
`al_final`** (atrapar errores sin que el programa se caiga, y el
`al_final` corre siempre), **13 funciones nuevas** (fecha y hora,
dormir, archivos y carpetas, reemplazar textos), **cambiar un lugar
de una lista o un mapa** (`lista[0] = 5;`, `mapa[21] = "FTP";`) y un
**icono propio** (`assets/icon.png`: un eclipse total con corona
dorada, que aparece en la extensión de VS Code y en todo). Sin
cambiar ninguna lógica.

## Cómo correrlo

**Para correr programas `.ec` no necesitas Rust ni Cargo:**
solo descarga Eclipse y escribe `eclipse programa.ec`. Rust y
Eclipse son lenguajes totalmente distintos: Rust solo se encarga
de los archivos del proyecto Eclipse (el código de Eclipse está
escrito en Rust), pero los programas `.ec` no necesitan nada de
Rust para funcionar: corren en la máquina virtual. Rust solo hace
falta si quieres compilar el propio Eclipse desde su código o
generar un `.exe` nativo con `rustc` (ver abajo).

La forma más fácil es instalarlo una vez:

```bash
# Instala Eclipse y su extensión de VS Code (colores, botón Run, LSP)
eclipse instalar
```

Y después, en cualquier terminal:

```bash
# Correr un ejemplo
eclipse ejemplos/01-basico/hola.ec

# Abrir el modo interactivo (escribe 'salir' para terminar)
eclipse

# Ver la ayuda (todos los comandos)
eclipse --ayuda
```

Si prefieres compilarlo desde el código (necesitas Rust: https://rustup.rs):

```bash
cargo build
./target/debug/eclipse ejemplos/01-basico/hola.ec
```

## Un programa en Eclipse

```rust
sea nombre = "Kevin";
sea mut puntos = 10;

si nombre == "Kevin" {
    imprimir("Hola,", nombre);
}

puntos = puntos + 5;
imprimir("Ahora tienes", puntos, "puntos");
```

## Las 34 palabras clave

| Palabra | Para qué sirve | Como en Rust |
|---|---|---|
| `sea` | Crear una variable fija | `let` |
| `mut` | La variable sí se puede cambiar (`sea mut x`) | `mut` |
| `si` | Si pasa esto... | `if` |
| `sino` | Si no pasa... | `else` |
| `mientras` | Repetir mientras algo sea verdad | `while` |
| `para` | Repetir con contador (`para i en 1..=5`) | `for` |
| `en` | Dentro de un `para` | `in` |
| `funcion` | Crear una función | `fn` |
| `devolver` | Regresar un valor de una función | `return` |
| `imprimir` | Mostrar cosas en pantalla | `println!` |
| `leer` | Lee lo que escribe el usuario y lo guarda en una variable (`leer nombre;`) | `read_line` |
| `lista` | Tipo: lista de valores (`sea numeros: lista = [1, 2, 3];`) | `Vec` |
| `para_cada` | Repetir con cada elemento de una lista (`para_cada n en numeros`) | `for x in lista` |
| `constante` | Crear una constante: nunca cambia (`constante PI = 3.14;`) | `const` |
| `verdadero` | Sí | `true` |
| `falso` | No | `false` |
| `y` | Y lógico | `&&` |
| `o` | O lógico | `||` |
| `no` | Negar | `!` |
| `romper` | Salir del bucle | `break` |
| `seguir` | Saltar a la siguiente vuelta | `continue` |
| `nulo` | Nada, vacío | `null` / `None` |
| `importar` | Cargar otro archivo `.ec` | `mod` / `use` |
| `usar` | Usar una librería de otro usuario (solo arriba del archivo: `usar matematicas;`) | `use` |
| `numero` | Tipo: número | `f64` |
| `texto` | Tipo: texto | `String` |
| `logico` | Tipo: verdadero/falso | `bool` |
| `segun` | Elegir según un valor (como `match`) | `match` |
| `paso` | El paso de un `para` (`para i en 1..10 paso 2`) | `.step_by()` |
| `repetir` | Repetir el cuerpo una vez y luego mientras algo sea verdad | `loop` + `while` |
| `mapa` | Tipo: mapa de claves y valores (`sea puertos: mapa = {21: "FTP"};`) | `HashMap` |
| `intentar` | Intentar algo: si falla, se atrapa el error en vez de caerse el programa | `try` |
| `si_falla` | Qué hacer si el `intentar` falló (con o sin variable del error: `si_falla e`) | `catch` |
| `al_final` | Qué hacer siempre, pase lo que pase (también si hubo `devolver`, `romper` o `seguir`) | `finally` |

## Los 9 símbolos

| Símbolo | Para qué sirve |
|---|---|
| `+` | Sumar (y pegar textos) |
| `-` | Restar |
| `*` | Multiplicar |
| `/` | Dividir |
| `%` | Residuo (lo que sobra) |
| `=` | Guardar un valor |
| `==` | Comparar si son iguales |
| `<` | Menor que |
| `>` | Mayor que |

**Nuevo en la 0.6: `++` y `--`:** `puntos++` suma 1 y `puntos--`
resta 1 (solo con números, y la variable debe ser `sea mut`).

Con esos símbolos se forman también `!=` (diferente), `<=` y `>=`,
`=>` (flecha de `segun`) y las asignaciones con operación:
`+=`, `-=`, `*=`, `/=`, `%=` (por ejemplo, `puntos += 5;` es lo mismo
que `puntos = puntos + 5;`). **Nuevo en la 0.7: `**=`** es la potencia
con asignación: `puntos **= 3;` es lo mismo que `puntos = puntos ** 3;`.
**Nuevo en la 0.5: `**` es la potencia**:
`2 ** 3` es 8, y `2 ** 3 ** 2` es 512 (se lee de derecha a izquierda).
El símbolo `&` es el préstamo: `saludar(&nombre)` le presta el texto
a la función sin que se lo quede; **nuevo en la 0.5: `&mut`** es el
préstamo para cambiar: `subir(&mut puntos)` le presta el número y la
función puede cambiarlo. Y `!` es lo mismo que `no`: `!verdadero` es falso.
Los signos de estructura `( ) { } , ; :` separan las partes del programa.

Los números se pueden escribir con guiones bajos para leerlos mejor:
`1_000` es lo mismo que `1000`, y `1_234_567` es lo mismo que `1234567`.

## Variables: fijas y que cambian

```rust
sea nombre = "Kevin";      // Fija: no se puede cambiar
sea mut puntos = 10;       // Cambia: sí se puede cambiar
puntos = puntos + 5;       // Esto funciona

// nombre = "Otro";        // Esto da error: la variable es fija
```

También puedes ponerle tipo: `sea edad: numero = 13;`

## Funciones que trae Eclipse

| Función | Qué hace |
|---|---|
| `imprimir(...)` | Muestra valores en pantalla |
| `leer()` | Lee una línea que escribe el usuario |
| `a_numero("123")` | Convierte texto a número |
| `a_texto(123)` | Convierte un valor a texto |
| `leer_archivo("notas.txt")` | Lee un archivo y devuelve su texto |
| `escribir_archivo("notas.txt", "hola")` | Escribe texto en un archivo |
| `largo("Kevin")` | Cuántas letras tiene un texto (5); con una lista, cuántos elementos tiene |
| `tipo_de(valor)` | El tipo: "numero", "texto", "logico" o "nulo" |
| `aleatorio()` | Un número al azar entre 0 y 99 |
| `copiar(texto)` | Una copia nueva del texto (para ownership) |
| `min(3, 7)` | El más chico de dos números (3) |
| `max(3, 7)` | El más grande de dos números (7) |
| `leer_numero()` | Lee un número que escribe el usuario |
| `agregar(mi_lista, 3)` | Devuelve una lista NUEVA con el 3 al final (la vieja no cambia) |
| `elemento(mi_lista, 0)` | Lo mismo que `mi_lista[0]`: el elemento en ese lugar |
| `ahora()` | Los segundos desde 1970 (el reloj de la computadora) |
| `redondear(3.7)` | Redondea un número (3.7 -> 4) |
| `absoluto(-5)` | El valor sin el signo (-5 -> 5) |
| `mayusculas("hola")` | El texto en MAYÚSCULAS ("HOLA") |
| `minusculas("HOLA")` | El texto en minúsculas ("hola") |
| `contiene("hola kevin", "kevin")` | Verdadero si el texto tiene esa parte |
| `ordenar([3, 1, 2])` | Una lista NUEVA ordenada: [1, 2, 3] (con textos, por abecedario) |
| `invertir([1, 2, 3])` | Una lista NUEVA al revés: [3, 2, 1]; con texto, "hola" -> "aloh" |
| `unir(["a", "b"], ", ")` | Pega una lista con un separador: "a, b" |
| `separar("a,b,c", ",")` | Parte un texto en una lista: ["a", "b", "c"] |
| `raiz(16)` | La raíz cuadrada de un número (4) |
| `sistema()` | Qué sistema operativo es: "Windows", "Linux" o "macOS" |
| `usuario()` | Quién eres (tu nombre de usuario) |
| `carpeta()` | En qué carpeta estás parado |
| `variable_entorno("HOME")` | El valor de una variable del sistema |
| `comando("echo hola")` | Corre un comando del sistema y devuelve lo que imprimió |
| `argumentos()` | La lista de datos con los que arrancó el programa |
| `procesadores()` | Cuántos procesadores tiene la computadora |
| `memoria()` | Cuánta memoria (RAM) tiene la computadora, en MB |
| `caracter(65)` | La letra de un número (65 -> "A") |
| `claves(mi_mapa)` | Las claves de un mapa, en una lista: [21, 22, 80] |
| `valores(mi_mapa)` | Los valores de un mapa, en una lista: ["FTP", "SSH", "HTTP"] |
| `tiene(mi_mapa, 21)` | Verdadero si el mapa tiene esa clave |
| `fecha()` | La fecha de hoy en UTC: "2026-11-17" (año-mes-día) |
| `hora()` | La hora de ahora en UTC: "14:32:05" (horas:minutos:segundos) |
| `fecha_hora()` | La fecha y la hora de ahora en UTC: "2026-11-17 14:32:05" |
| `dormir(2)` | Espera 2 segundos sin hacer nada (para automatizar) |
| `existe("notas.txt")` | Verdadero si ese archivo o carpeta existe |
| `listar_carpeta("mis_fotos")` | Los archivos y carpetas que hay ahí, en orden alfabético |
| `crear_carpeta("respaldo")` | Crea esa carpeta (y las que falten en el camino) |
| `mover_archivo("viejo.txt", "nuevo.txt")` | Mueve un archivo o le cambia el nombre |
| `borrar_archivo("viejo.txt")` | Borra un archivo |
| `reemplazar("hola mundo", "mundo", "Kevin")` | Cambia un texto por otro, todas las veces: "hola Kevin" |

## Ownership (como Rust)

Cada valor tiene UN solo dueño.

- **Los números y los lógicos son fotocopias:** cuando se los das a otra
  variable o a una función, se copian. La variable vieja sigue viva.
- **Los textos son originales:** cuando se lo das a otra variable o a
  una función, se MUEVE. La variable vieja ya no se puede usar:
  Eclipse te dice *"la variable se movió, ya no puedes usarla"*.
- **Las listas también se mueven** (como los textos): `sea b = a;`
  mueve la lista `a` a `b`. `copiar()` también copia listas.
- **Las constantes nunca se mueven** (son copias siempre) y nunca
  se pueden cambiar: `constante PI = 3.14;` es para siempre.
- **Un dato prestado (`&` o `&mut`) no se puede mover** mientras
  corre la función: si una función recibe `&nombre` y adentro
  intenta `sea otro = nombre;`, Eclipse dice *"no se puede mover
  'nombre' porque está prestado a una función"*.
- **`&mut` también sirve para listas:** `meter(&mut numeros)` deja
  que la función cambie tu lista y te la devuelva cambiada.
- **`copiar(texto)`** hace una copia nueva: el original sigue vivo.
- **`&nombre` PRESTA el texto a una función:** `saludar(&nombre)` le
  presta el texto sin que la función se lo quede. Es como el `&` de
  Rust: prestar, no regalar. Solo se usa al llamar a una función.
- **`imprimir` y las demás funciones que trae Eclipse solo MIRAN el
  texto:** no se lo quedan, así que la variable sigue viva después.
- **Tus funciones SÍ se quedan con el texto** que les pasas.
- **Si le das un valor nuevo a una variable movida, revive:**
  `a = "otro";` después de mover `a` funciona.
- **Cuando una variable se mueve, Eclipse te dice A DÓNDE se fue:**
  *"la variable 'a' se movió a la variable 'b', ya no puedes usarla.
  Para quedarte con una copia, usa copiar(a)"* — o *"se movió a la
  función 'saludar'"* si fue a una función.

```rust
sea a = "Kevin";
sea b = a;           // a se mueve a b
// imprimir(a);      // Error: a ya se movió

sea c = copiar(b);   // b sigue vivo, c es una copia
imprimir(b);
imprimir(c);
```

## El Borrow Checker: el compilador regaña ANTES de correr

Desde la 1.0, Eclipse revisa tu programa **antes** de correrlo,
en los 3 motores, en el LSP y cuando publicas una librería.
Si algo está mal, te regaña en español, con la línea exacta:

```ec
sea a = "hola";
sea b = a;
imprimir(a);
```

```
Error: mi_programa.ec: Error en la línea 3: Oye, no puedes usar 'a' aquí:
se la regalaste a la variable 'b' en la línea 2. Si quieres quedarte
con una copia, usa copiar(a)
```

Lo que revisa:

- **Usar una variable que regalaste** (se movió a otra variable o a una función).
- **Usar una variable que no existe** (¿la creaste con `sea` antes?).
- **Cambiar una variable fija** (`sea` sin `mut`) o una `constante`.
- **Prestar con `&mut` una variable fija** o prestarla dos veces a la vez.
- **Llamar a una función que no existe** (¿te falta un `usar` arriba?).
- **Darle a una función más o menos datos de los que necesita.**
- **Decir que una variable es un tipo y darle otro** (`sea nombre: numero = "Kevin";`).
- **`devolver`, `romper` o `seguir` donde no valen.**
- **Dividir o calcular el resto entre un cero escrito a mano**
  (`10 / 0`, `10 % (0)`): te avisa antes de correr.
- **Pedir un lugar de una lista que no existe** (si la lista
  está escrita a mano, o es una variable con una lista escrita
  a mano: `lista[5]` cuando solo tiene 3 elementos).
- **Que una función devuelva tipos distintos** (un `numero` en
  una rama y un `texto` en otra, o un `devolver;` vacío y luego
  un `devolver 5;`).

Lo que NO revisa (todavía se para en tiempo de ejecución,
con el mismo mensaje de siempre):

- Si regalas una variable dentro de un `si` y la usas después:
  el compilador no sabe si el `si` va a pasar o no, así que te
  deja correr y el motor te para si de verdad la regalaste.
- Lo que pasa dentro de los archivos que importas (solo revisa
  que existan y qué funciones traen).
- Qué tipo devuelve una función hecha por ti, si el tipo no se
  sabe (por ejemplo, si devuelve un dato que le pasaron). Si los
  dos tipos se saben y son distintos, sí te avisa (ver arriba).

En el modo interactivo (REPL) NO se revisa, porque cada línea
usa variables de las líneas anteriores.

## `segun` (como el `match` de Rust)

Elige un brazo según el valor. `_` significa "cualquier otro valor".
Si ningún brazo coincide, no pasa nada.

```rust
segun puntos {
    10 => { imprimir("¡diez puntos, premio!"); }
    5 => { imprimir("cinco puntos"); }
    _ => { imprimir("otro puntaje"); }
}
```

## Interpolación de textos

Dentro de un texto, `{nombre}` pega el valor de la variable. También
se pueden poner cuentas: `{puntos * 2}`. Para escribir una llave de
verdad, escribe `{{` o `}}`.

```rust
sea nombre = "Kevin";
sea puntos = 10;
imprimir("Hola, {nombre}, tienes {puntos} puntos");
imprimir("El doble es {puntos * 2}");
```

## `para` con paso

`para i en 1..10 paso 2` avanza de 2 en 2: 1, 3, 5, 7, 9. El paso
también puede ser negativo (para ir hacia atrás) o una variable.
Si no lo escribes, el paso es 1. Si el paso es 0, el bucle no corre.

```rust
para i en 1..10 paso 2 {
    imprimir(i);
}
para i en 10..1 paso -2 {
    imprimir(i);
}
```

## Listas (como el `Vec` de Rust)

Una lista guarda varios valores en orden. Se crea con corchetes y se
lee con `[lugar]` (el primer lugar es el 0).

```rust
sea mut numeros: lista = [1, 2, 3];
imprimir(numeros);        // [1, 2, 3]
imprimir(numeros[0]);     // 1
imprimir(largo(numeros)); // 3
numeros = agregar(numeros, 4);
imprimir(numeros);        // [1, 2, 3, 4]
imprimir(numeros + [5]);  // [1, 2, 3, 4, 5] (pegar dos listas)
```

- Una **lista vacía es falsa** en un `si` o un `mientras`; una lista
  con algo es verdadera.
- `mi_lista[9]` cuando solo tiene 3 cosas da error en español:
  *"ese lugar no existe en la lista: pediste el 9 pero la lista
  tiene 3 elementos"*.
- Las listas **se mueven** como los textos (ver Ownership).
- Ejemplos: `ejemplos/06-listas/listas.ec`.

## `para_cada`: repetir con cada elemento

Recorre una lista, elemento por elemento. La variable del bucle es
fija adentro (no la puedes cambiar).

```rust
sea frutas: lista = ["manzana", "pera", "uva"];
para_cada fruta en frutas {
    imprimir(fruta);
}
```

También sirven `romper` (salir) y `seguir` (saltar a la siguiente
vuelta). Ejemplo: `ejemplos/06-listas/listas.ec`.

## `leer` como Input

`leer nombre;` lee una línea que escribe el usuario y la guarda en
la variable. La variable debe ser `sea mut` (si es fija, da error).
El texto se convierte según lo que la variable ya es: si era un
número, lo convierte a número; si era texto, guarda el texto.

**Nuevo en la 0.7: `leer` con mensaje.** Puedes escribir el mensaje
antes del nombre, y Eclipse lo muestra sin salto de línea, para que
el usuario escriba en la misma línea:

```rust
sea mut nombre = "";
leer "¿Cómo te llamas? " nombre;
imprimir("Hola, " + nombre);
```

**Nuevo en la 0.7: `leer` para listas.** Si la variable es una
lista, Eclipse separa lo que escribes por comas. Si la lista ya
tenía números, cada parte debe ser un número; si estaba vacía,
decide sola: si todo son números, guarda números, si no, textos:

```rust
sea mut numeros: lista = [];
leer "Dame números con comas: " numeros;
imprimir(numeros);        // [1, 2, 3] si escribiste 1, 2, 3
```

```rust
sea mut edad: numero = 0;
leer edad;
imprimir(edad + 1);
```

Si escribes letras donde iba un número, Eclipse dice en español:
*"eso no era un número: 'hola'"*. La función `leer()` de antes sigue
funcionando igual (`leer()` sin nombre entre paréntesis es la función;
`leer nombre;` sin paréntesis es la orden). Ejemplos:
`ejemplos/06-listas/leer_input.ec` y `pruebas05/leer_lista.ec`.

## Constantes: `constante` y `++` / `--`

```rust
constante PI = 3.14;
imprimir(PI);
// PI = 3.0;   // Error: la constante nunca se puede cambiar

sea mut puntos = 0;
puntos++;   // 1
puntos++;   // 2
puntos--;   // 1
```

- Una constante **necesita un valor** al crearla y **nunca cambia ni
  se mueve**: es una copia siempre.
- `++` y `--` solo sirven con números, y la variable debe ser
  `sea mut`. Ejemplo: `ejemplos/06-listas/constante.ec`.

## Mapas (como un diccionario)

Un mapa guarda pares **clave → valor**: cada clave tiene su valor.
Las claves son números, textos o verdadero/falso (no una lista).
Si una clave se repite, gana el último valor (y se queda en el
primer lugar donde apareció). Un mapa vacío es falso en un `si`.

```rust
// Como el ejemplo del diccionario, pero con sintaxis 100% Eclipse.
sea mut servicios = {21: "FTP", 22: "SSH", 80: "HTTP"};
imprimir(servicios[22]);            // SSH
imprimir(servicios["nombre"]);      // Error: ese mapa no tiene la clave nombre
servicios[443] = "HTTPS";           // Agrega una clave nueva
servicios[21] = "FTP seguro";       // Cambia un valor que ya existe
imprimir(claves(servicios));        // [21, 22, 80, 443]
imprimir(valores(servicios));       // [FTP seguro, SSH, HTTP, HTTPS]
imprimir(tiene(servicios, 80));     // verdadero
imprimir(largo(servicios));         // 4
imprimir(tipo_de(servicios));       // mapa
```

- **Cambiar un lugar** (`lista[0] = 5;`, `mapa[21] = "FTP";`):
  la variable tiene que ser `sea mut` (no fija ni constante) y
  no haberse movido. Si la clave no existe en el mapa, la crea.
  Solo vale con un lugar directo: `m[0][1] = 5;` da error
  ("para cambiar un lugar dentro de otro lugar, guárdalo
  primero en una variable").
- Un mapa **se mueve** como un texto o una lista: `sea b = a;`
  se lleva `a` (y `copiar(a)` te deja una copia para quedarte
  con una). Ejemplo: `ejemplos/11-mapas-automatizacion/mapas.ec`.

## `intentar` / `si_falla` / `al_final` (atrapar errores)

Antes, si algo fallaba, el programa se paraba. Ahora puedes
**intentarlo** y decidir qué hacer si falla, sin que se caiga
el programa. El `al_final` corre **siempre**: si todo salió
bien, si hubo error, si hubo un `devolver`, un `romper` o un
`seguir` dentro.

```rust
sea mut cero = 0;
intentar {
    imprimir(10 / cero);
} si_falla e {
    imprimir("No se pudo dividir:");
    imprimir(e);              // No se puede dividir entre cero
} al_final {
    imprimir("Esto siempre se imprime.");
}

funcion seguro(x) {
    intentar {
        devolver 10 / x;
    } si_falla e {
        devolver 0;           // Si falla, devuelve 0
    }
}
imprimir(seguro(2));          // 5
imprimir(seguro(0));          // 0
```

- `intentar` necesita al menos un `si_falla` o un `al_final`
  (si no tiene ninguno, da error antes de correr).
- La variable del error (`si_falla e`) es opcional y es un
  texto con el mensaje del error.
- Solo con `al_final` (sin `si_falla`), el error sigue su
  camino después del `al_final`: el programa se para igual,
  pero el `al_final` ya corrió.
- Se puede anidar (`intentar` dentro de `intentar`) y usarlo
  dentro de bucles y funciones: un `romper` o `seguir` dentro
  de un `intentar` rompe o sigue el bucle de afuera después
  de correr el `al_final`. Ejemplos:
  `ejemplos/11-mapas-automatizacion/intentar.ec`.

## Automatización (fecha, hora, archivos y carpetas)

Programas completos que hacen cosas solos: revisar si un
archivo existe, listar una carpeta, mover o borrar archivos,
esperar un rato, y saber la fecha y la hora.

```rust
imprimir(fecha());                      // 2026-11-17 (UTC)
imprimir(hora());                       // 14:32:05 (UTC)
imprimir(existe("notas.txt"));          // verdadero o falso
imprimir(listar_carpeta("mis_fotos"));  // [foto1.png, foto2.png]
crear_carpeta("respaldo");
mover_archivo("viejo.txt", "respaldo/viejo.txt");
imprimir(reemplazar("hola mundo", "mundo", "Kevin"));  // hola Kevin
dormir(2);                              // Espera 2 segundos
```

- **La fecha y la hora son en UTC** (tiempo universal): Rust
  solo no sabe tu zona horaria, así que la hora sale con la
  diferencia de tu país (en México, 6 horas más que tu reloj).
  El formato es fijo: fecha `año-mes-día`, hora
  `horas:minutos:segundos`.
- `dormir` no acepta números negativos ni infinitos.
- Ejemplo completo: `ejemplos/11-mapas-automatizacion/automatizacion.ec`.

## El icono de Eclipse

Eclipse tiene su propio icono: un **eclipse total** (un círculo
negro con una corona dorada) sobre fondo azul noche, en
`assets/icon.png`. Es el icono que aparece en la extensión de
VS Code (`vscode-eclipse/assets/icon.png`, declarado en el
`package.json` de la extensión) y en todo lo de Eclipse.

## Proyectos: varios archivos juntos (como Cargo)

Un proyecto es una carpeta con un `Eclipse.toml` (su manifiesto:
nombre, archivo principal y versión) y una carpeta `src/` con
tus archivos `.ec`. Así Eclipse sabe qué archivos van juntos.

```bash
# Crear un proyecto nuevo
eclipse nuevo mi_juego

# Entrar y correrlo (corre el archivo principal)
cd mi_juego
eclipse correr

# Compilar TODOS los .ec, con barra de progreso
eclipse construir

# Compilar y además crear el ejecutable standalone
eclipse construir --release

# Correr TODOS los .ec de una carpeta, uno por uno,
# con barra de progreso y un resumen al final
eclipse correr-todos src
```

El `Eclipse.toml` es así:

```toml
[proyecto]
nombre = "mi_juego"
principal = "src/principal.ec"
version = "0.1.0"
```

Los archivos se enlazan con `importar`: el principal importa a
los demás, y el compilador los mete todos en un solo programa.
Ejemplo completo: `ejemplos/07-proyectos/mi_juego/`.

## Librerías: `usar` para usar código de otros

Cualquier persona puede hacer una librería en Eclipse (para
videojuegos, dibujo en ventanas, matemáticas o lo que se le
ocurra) y cualquiera puede usarla con una sola línea **arriba**
del archivo:

```ec
usar matematicas;
usar dibujo;

imprimir(factorial(5));   // 120
titulo("Mi juego");
```

Reglas de `usar`:

- **Siempre arriba**, antes de cualquier otra línea. Si quieres
  usar 2, 3 o 4 librerías, pon cada una en su propio renglón.
- **En cualquier otro lugar da error:** después de una línea de
  código, dentro de una función, dentro de un `si`... siempre
  error, porque el compilador tiene que saber qué trae la
  librería antes de leer tu código.
- Solo funciona **si la librería está instalada**; si no, el
  error te dice cómo publicarla.

Comandos para las librerías:

```bash
# Publicar una librería (te hace 6 preguntas y la envía a revisión)
eclipse biblioteca publicar bibliotecas/matematicas.ec
# Ver las librerías instaladas (y las que están en revisión)
eclipse biblioteca listar
# Ver qué trae una librería
eclipse biblioteca info matematicas
# Ver si una librería está disponible o en revisión
eclipse biblioteca estado matematicas
# Aprobar las librerías que ya cumplieron su revisión
eclipse biblioteca revisar
# Quitar una librería (o cancelar un envío en revisión)
eclipse biblioteca quitar matematicas
```

**El Registro (un sistema aparte, del mismo proyecto):** las
librerías NO viven dentro de Eclipse: viven en el Registro, su
propio sistema en `~/.eclipse/registro/` (las disponibles en
`registro/bibliotecas/<nombre>/`, las que están en revisión en
`registro/cola/<nombre>/`). Si tenías librerías de la 1.0 en
`~/.eclipse/bibliotecas/`, se mueven solas al Registro la
primera vez que usas un comando de librerías.

**Para subir una librería** (lo que hace y revisa `publicar`):

1. Te hace **6 preguntas por terminal**: nombre de la
   librería, tu nombre, descripción, versión, categoría y
   cómo te pueden contactar. Si no escribes nada, usa el
   valor que te sugiere entre corchetes.
2. **Solo se puede subir una librería cada 5 horas.** Si
   acabas de subir una, te dice cuánto te falta para la
   siguiente.
3. **Tiene que estar escrita SOLO en Eclipse:** cualquier
   otro lenguaje no se entiende (el lexer y el parser lo
   rechazan) y no se publica. Y **tiene que estar en
   español:** si encuentra una palabra en inglés (`let`,
   `if`, `print`, `function`...), te dice en qué línea está
   y no la publica.
4. **No puede ser nada peligroso:** si el nombre es `firmware`,
   `virus`, `malware`, `troyano` o parecido, no se publica.
   Tampoco si el nombre es una palabra de Eclipse (`si`,
   `para`, `usar`...), porque entonces nadie podría escribir
   `usar si;`. Si quiere correr comandos del sistema
   (`comando`) o usar archivos fuera de su carpeta (`..`,
   rutas absolutas o con letra de unidad), tampoco. Y el
   Borrow Checker la revisa antes de aceptarla.
5. **Tiene que tener al menos una función:** una librería sin
   funciones no sirve para nada y no se publica (nada de
   librerías mal hechas).
6. **Después de subirla, tarda de 1 a 2 días en llegar a
   Eclipse, sin excepciones:** queda en la cola de revisión
   (1 día y medio) y solo se puede usar cuando se cumple el
   tiempo. Mientras tanto, `usar` te dice cuánto falta. El
   comando `biblioteca revisar` aprueba las que ya cumplieron
   su tiempo (también se aprueban solas cuando las usas).
7. Cada vez que usas una librería, Eclipse revisa que no haya
   cambiado desde que se publicó (su número de control). Si
   alguien la tocó, te avisa y no la usa.

**Honestidad:** ningún escáner puede garantizar al 100% que
una librería es segura (un programa puede esconder cosas raras
de muchas formas). El escáner atrapa los patrones conocidos y
peligrosos, pero **instala solo librerías de personas en las
que confíes**, igual que con cualquier programa que descargas.

Las 3 librerías de ejemplo (en `bibliotecas/`):

- **`matematicas`**: `factorial`, `es_par`, `es_primo`, `mcd`,
  `mcm`, `fibonacci`, `promedio`.
- **`juegos`**: `dado`, `moneda`, `elegir`, `tirar_dados`.
- **`dibujo`**: `limpiar_pantalla`, `color_texto`, `linea`,
  `caja`, `titulo` (dibuja en la **ventana de la terminal**
  con letras y colores; las ventanas de verdad con botones
  y ratón necesitan una librería gráfica, eso viene después).

## Ejecutables standalone: sin Rust ni Cargo

`eclipse empaquetar` crea UN SOLO archivo que corre solo, sin
instalar nada: es el propio Eclipse con tu programa ya compilado
a bytecode pegado al final. Lo copias a otra computadora, le das
doble clic (o lo corres en la terminal) y funciona.

```bash
eclipse empaquetar juego.ec
# -> juego (o juego.exe en Windows)
./juego
```

Con honestidad: esto NO es código máquina como el `.exe` de
`rustc` (para ese sí hace falta Rust). Es Eclipse + tu programa
en un solo archivo. Para quien lo corre, es lo mismo: no
necesita instalar nada. El `.exe` nativo sigue existiendo
con `eclipse --exe` (ese sí necesita `rustc`).

## Instalar Eclipse (y su extensión)

```bash
eclipse instalar      # Copia Eclipse a ~/.eclipse/bin, lo agrega
                      # al PATH y instala la extensión de VS Code
eclipse desinstalar   # Quita Eclipse y la extensión
```

La extensión se instala desde los archivos que van incrustados
dentro del propio Eclipse, así que `instalar` no necesita
descargar nada más. Después de instalar, cierra y vuelve a
abrir la terminal y VS Code.

## Hablar con el sistema (hardware y software)

Eclipse es un programa normal, así que no toca el hardware
directo (eso es trabajo del sistema operativo y de los
controladores). Lo que sí hace es **pedirle cosas al sistema
operativo** y **correr otros programas**, con 9 funciones nuevas:

```ec
imprimir(sistema());              // "Linux" (o "Windows" o "macOS")
imprimir(usuario());              // quién eres
imprimir(carpeta());              // en qué carpeta estás
imprimir(procesadores());         // cuántos procesadores tienes
imprimir(memoria());              // cuánta RAM tienes, en MB
imprimir(comando("echo hola"));   // corre un comando y te da lo que imprimió
imprimir(variable_entorno("HOME")); // una variable del sistema
imprimir(argumentos());           // con qué datos arrancó el programa
imprimir(caracter(65));           // "A"
```

`comando` corre lo que le pidas en la terminal del sistema
(`cmd` en Windows, `sh` en Linux y Mac). Si el comando falla,
te dice con qué código terminó y qué error imprimió. Ejemplo
completo: `ejemplos/10-librerias/sistema_demo.ec`.

## Cómo está hecho por dentro

```
archivo.ec  →  Lexer  →  Tokens  →  Parser  →  AST
                                                    ├→  Compilador  →  ByteCode  →  Máquina Virtual  →  resultado
                                                    ├→  Transpilador  →  Rust  →  rustc  →  .exe nativo  →  resultado
                                                    └→  Paquete (ECBIN)  →  Eclipse + bytecode  →  ejecutable solo  →  resultado
```

- **`src/lenguaje/lexer.rs`** — Corta el texto en pedacitos (tokens).
- **`src/lenguaje/parser.rs`** — Revisa el orden y arma el arbolito (AST).
- **`src/lenguaje/ast.rs`** — Las formas del arbolito (qué es una suma, un `si`, una función...).
- **`src/motor/valor.rs`** — Los valores de Eclipse (`Valor`: número, texto, lógico, nulo, lista) y las operaciones con ellos. Antes vivía dentro del intérprete; ahora es un archivo propio que usan la VM y el compilador.
- **`src/motor/vm.rs`** — La máquina virtual: EL motor de Eclipse (desde la 1.1 ya no hay intérprete). Corre el bytecode por slots, con depurador (`--depurar`, `--trazar`) y modo interactivo.
- **`src/motor/compilador.rs`** — Traduce el arbolito a bytecode (con slots).
- **`src/motor/bytecode.rs`** — Las instrucciones y los pedazos de bytecode.
- **`src/motor/transpilador.rs`** — Traduce el arbolito a código Rust, usando `rt/eclipse_rt.rs`.
- **`rt/eclipse_rt.rs`** — La librería que usan los `.exe`: los mismos valores, las mismas reglas y los mismos errores en español.
- **`src/motor/paquete.rs`** — El paquete standalone: guarda el bytecode en formato binario (ECBIN) y lo pega al final de Eclipse para hacer un ejecutable solo, sin rustc.
- **`src/lsp/servidor.rs` + `src/lsp/json.rs`** — El servidor de lenguaje (LSP) en Rust puro: autocompletado, ayuda (hover), diagnósticos, esquema, definición y referencias.
- **`src/herramientas/autocompletar.rs`** — El autocompletado inteligente (qué puedes escribir), que también usa el LSP.
- **`src/herramientas/progreso.rs`** — La barra de progreso (como la de Cargo).
- **`src/herramientas/proyecto.rs`** — Los proyectos: Eclipse.toml, `nuevo`, `construir`, `correr`, `correr-todos`.
- **`src/herramientas/instalar.rs`** — El instalador: copia Eclipse, lo agrega al PATH y instala la extensión de VS Code.
- **`src/bin/probar.rs`** — El probador: compara la VM (el motor), el paquete y el `.exe` en todos los ejemplos y casos difíciles, y prueba los proyectos, la extensión y el sistema de librerías (publicar, revisión, límite de 5 horas, aprobar, usar). Todo en Rust. También compara los mensajes de error.
- **`src/motor/verificador.rs`** — El Borrow Checker: revisa el programa ANTES de correrlo y regaña en español si usas una variable que regalaste, una que no existe, una fija que quieres cambiar, o una función con los datos equivocados.
- **`src/motor/sistema.rs`** — Hablar con el sistema: qué sistema es, usuario, carpeta, variables, comandos, procesadores y memoria.
- **`src/herramientas/bibliotecas.rs`** — Las librerías y el Registro: `usar`, publicar (con las 6 preguntas), listar, info, estado, revisar, quitar, la cola de revisión de 1 a 2 días, el límite de una subida cada 5 horas, la migración desde la 1.0, y la seguridad (solo Eclipse, que esté en español, el escáner de cosas peligrosas y el número de control).
- **`bibliotecas/`** — Las 3 librerías de ejemplo: `matematicas.ec`, `juegos.ec` y `dibujo.ec`.
- **`pruebas09/`, `pruebas10/` y `pruebas11/`** — Casos difíciles nuevos: la 09 prueba el cortocircuito, la recursión y el `agregar` en su sitio; la 10 prueba el Borrow Checker, el `usar` y las funciones del sistema; la 11 prueba las 3 revisiones nuevas del Borrow Checker y que todo corre en la VM.
- **`src/bin/bench.rs`** — El banco de velocidad: mide cuánto tarda la VM, el `.exe` y el paquete en los mismos programas y dice cuántas veces más rápido es uno que otro (el intérprete ya no existe).
- **`banco/`** — Los 5 programas del banco: `fib`, `bucle`, `para`, `texto` y `lista`.
- **`pruebas08/`** — 27 casos difíciles nuevos: bordes de cada palabra y símbolo, y 8 programas que DEBEN fallar igual en los 3 motores.
- **`src/bin/probar_lsp.rs`** — El probador del LSP: habla JSON-RPC con el servidor y revisa sus respuestas. Todo en Rust.
- **`src/main.rs`** — La puerta de entrada: lee el archivo o abre el modo interactivo.
- **`ejemplos/`** — Los ejemplos, ordenados por tema: `01-basico`, `02-decisiones-y-bucles`, `03-funciones`, `04-archivos`, `05-juegos`, `06-listas`, `07-texto`, `07-proyectos`, `08-texto-listas`, `09-rapido` y `10-librerias`.

## VS Code: colores, autocompletado inteligente y ayuda (LSP)

En la carpeta `vscode-eclipse/` hay una extensión que le pone colores a
los archivos `.ec` **y te sugiere qué escribir mientras escribes**: tus
variables (con su tipo), tus funciones (con cuántos datos necesitan),
las palabras clave y las funciones de Eclipse. Solo sugiere lo que se
ve desde donde estás escribiendo: una variable dentro de un bloque no
aparece fuera de él. Y si pasas el mouse sobre una variable o una
función, te dice qué es y de qué tipo es (hover).

Todo eso lo hace el **servidor de lenguaje de Eclipse, que es Rust
puro** (`eclipse --lsp`): es el mismo programa `eclipse` el que
responde. La extensión de VS Code es solo un puente chiquito que
enciende ese servidor y le pasa tus mensajes, porque VS Code solo
acepta extensiones en JavaScript. La inteligencia es 100% Rust: si
usas otro editor (Neovim, Helix, Zed), no necesitas la extensión,
solo `eclipse --lsp`.

**Nuevo en la 0.6: el botón ▶ Ejecutar.** Arriba a la derecha del
editor (y también en la paleta de comandos y con el atajo
Ctrl+Alt+R) hay un botón que corre tu archivo `.ec` en la terminal
de VS Code, sin salir del editor. También hay un enlace
"▶ Ejecutar" arriba de tu archivo. El botón guarda el archivo
primero, para que corra lo último que escribiste.

**Nuevo en la 0.6: diagnósticos y esquema.** El servidor ahora te
avisa de los errores **mientras escribes** (una línea roja con el
mensaje en español, en la línea exacta), y te muestra el esquema
del archivo (tus variables, constantes y funciones, con su línea)
en el panel de esquema de VS Code.

**Nuevo en la 0.7: definición y referencias.** Haz clic derecho en
una variable o función y elige "Ir a la definición" para saltar a
donde se definió, o "Buscar referencias" para ver todos los lugares
donde se usa. Y hay dos botones Run más: **Ejecutar con máquina
virtual** (Ctrl+Alt+V) y **Ejecutar con depurador** (Ctrl+Alt+D),
en la paleta de comandos. El depurador ahora también te deja ver
una variable con `p nombre` y al final te dice cuántos pasos
corrió y cuánto tardó.

Para instalarla, la forma fácil es:

```bash
eclipse instalar
```

Eso instala Eclipse y la extensión a la vez. La forma manual es:
compilar con `cargo build --release` (el programa queda en
`target/release/eclipse`) y copiar la carpeta `vscode-eclipse/`
a la carpeta de extensiones de VS Code, con el nombre
`kevin.eclipse-lenguaje-0.7.0`. Si VS Code no encuentra el
programa `eclipse`, pon su ruta en la configuración:
`"eclipse.ruta": "C:\\ruta\\a\\eclipse.exe"`.
3. Copia la carpeta `vscode-eclipse` a:
   - Windows: `C:\Users\TU_USUARIO\.vscode\extensions\eclipse-lenguaje`
4. Cierra VS Code y vuelve a abrirlo.
5. Abre un archivo `.ec`, escribe y verás las sugerencias.

También puedes probarlo desde la terminal:
```bash
cargo run -- --completar ejemplos/01-basico/hola.ec 5 10
```
Te dice qué puedes escribir en la línea 5, columna 10.

## Los límites de esta versión (con honestidad)

Eclipse 1.2 corre tus programas con **la máquina virtual por
slots** (EL motor: el intérprete ya no existe) y con el **.exe
nativo** (compilado por `rustc`). La VM y el `.exe` hacen
exactamente lo mismo, y el paquete standalone corre en la VM.
Lo que todavía NO tiene:

- **Hablar directo con el hardware** (sensores, luces, puertos, controladores):
  eso es trabajo del núcleo del sistema operativo, no de un programa
  normal. Eclipse 1.2 sí habla con el sistema operativo (qué sistema es,
  usuario, carpeta, comandos, procesadores, memoria: sección de arriba)
  y con el software (archivos, teclado y pantalla), que es como hablan
  los programas normales.
- **Escanear puertos o redes: no.** Del ejemplo que llegó con el
  pedido de la 1.2 solo se tomó la **estructura** (un diccionario,
  funciones y atrapar errores), no el escáner: Eclipse no trae
  ninguna función para conectarse a otras computadoras ni para
  revisar qué puertos tienen abiertos.
- El autocompletado no sugiere nada si el archivo tiene un error de
  lexer (por ejemplo, un texto sin cerrar). Cuando cierras el texto,
  vuelve a funcionar.
- En el `.exe`, si una función recibe la cantidad equivocada de datos,
  el error sale después de calcular los datos (el mensaje es el mismo
  y el programa se para igual). En la VM sale antes.
- En Linux y Mac el programa compilado no lleva la terminación `.exe`
  (en Windows sí).
- Pediste **1 símbolo nuevo** y te di 2: `++` y `--` van juntos,
  porque no tiene sentido subir sin poder bajar.
- VS Code sigue necesitando un puente chiquito en JavaScript para
  encender el servidor y el botón ▶ Ejecutar, porque VS Code solo
  acepta extensiones en JavaScript. La inteligencia es 100% Rust.
- Un `para` con números gigantescos (cerca del máximo) corre una
  sola vuelta y se detiene solo en la VM por su límite de seguridad
  de 100 millones de instrucciones; en el `.exe` se comporta como
  un `mientras` infinito (Ctrl+C para pararlo), igual que cualquier
  bucle que no termina.
- **El ejecutable de `empaquetar` no es código máquina:** es
  Eclipse + tu programa en bytecode, en un solo archivo. Para
  quien lo corre es lo mismo (no necesita instalar nada), pero
  pesa más que un .exe nativo y corre en la máquina virtual.
  El `.exe` de verdad (código máquina) sigue siendo
  `eclipse --exe`, y ese sí necesita `rustc`.
- **El `.exe` nativo sigue necesitando Rust instalado** (para
  `rustc`). Todo lo demás (correr .ec, la VM, el paquete, los
  proyectos, el LSP) funciona con solo tener Eclipse.
- **Armar una lista con `agregar` en un bucle ya NO es lento
  (desde la 0.9):** cuando escribes `x = agregar(x, v)`, el
  valor se mete en su sitio sin copiar la lista entera
  (10,000 elementos: 9 ms en vez de 530 ms). Solo sigue
  copiando cuando el resultado va a OTRA variable
  (`z = agregar(x, v)`), porque ahí `x` no se puede mover:
  `z` es una lista nueva y `x` sigue igual.
- **La VM es EL motor de Eclipse desde la 1.1** (el intérprete
  ya no existe): el bucle `para` avanza en una sola instrucción
  fusionada en la VM, y en recursión la VM corre 2.8x más rápido
  que lo que tardaba el intérprete. Los números están en la
  sección 1.2 (el banco ahora mide VM, `.exe` y paquete).
- **Un `para` con texto como límite da un error con palabras
  distintas en la VM:** el `.exe` dice "el inicio del 'para'
  debe ser un numero" y la VM dice que solo puede comparar
  números. En los dos motores el programa se para con error,
  solo cambian las palabras en la VM (es la única diferencia
  de mensajes que queda entre motores).
- **0.01 ms por programa no es real:** un programa que suma
  1,000,000 de números tarda unos 300 ms en cualquier motor,
  porque son 1,000,000 de sumas de verdad. Y ningún programa
  puede ser más rápido que Rust: el `.exe` de Eclipse ES
  código Rust, así que corre a velocidad Rust, no más rápido.
  Lo que sí se puede (y se hizo) es acercar la VM a esa
  velocidad y medirlo todo con el banco (sección 1.2).
- **`y` no puede ser nombre de variable:** `y` es la palabra
  del Y lógico (`si a y b`), así que `sea y = 5;` da error.
  Lo mismo pasa con las demás palabras clave (usa `sea lista`
  solo como se explicó en la 0.6: `lista`, `para_cada`,
  `constante` y `leer` sí se pueden usar como nombres).
- **El Borrow Checker no lo atrapa todo:** si regalas una
  variable dentro de un `si` y la usas después, te deja correr
  y el motor te para en ejecución. Tampoco revisa el código
  dentro de los archivos que importas, ni qué tipo devuelve una
  función si el tipo no se sabe (por ejemplo, si devuelve un
  dato que le pasaron). Eso sigue siendo trabajo del motor.
  Desde la 1.1 sí avisa si una función devuelve dos tipos
  distintos que sí se saben, si divides entre un cero escrito
  a mano, y si pides un lugar de una lista que no existe.
  Desde la 1.2 también avisa si la clave de un mapa no es un
  número, un texto o verdadero/falso, si cambias un lugar de
  una lista con un número fuera de rango, y si cambias un lugar
  de una variable fija, constante o que ya se movió.
- **El escáner de librerías no garantiza seguridad al 100%:**
  atrapa los patrones conocidos (comandos, rutas peligrosas,
  palabras en inglés, nombres de virus, librerías sin funciones,
  nombres con palabras de Eclipse), pero un programa puede
  esconder cosas de muchas formas. Por eso la revisión tarda de
  1 a 2 días y solo se puede subir una librería cada 5 horas:
  nada se sube a lo tonto. Aun así, **instala solo librerías
  de personas en las que confíes**.
- **La librería `dibujo` dibuja en la terminal**, no en
  ventanas de verdad con botones y ratón. Eso necesita una
  librería gráfica, que todavía no existe en Eclipse.
- **`usar` solo vale arriba del archivo.** Si lo pones en
  cualquier otro lugar, da error. Es a propósito: el
  compilador tiene que saber qué trae la librería antes de
  leer tu código.
- **La fecha y la hora son en UTC**, no en tu hora local
  (Rust solo no sabe tu zona horaria). `fecha()` es
  "año-mes-día" y `hora()` es "horas:minutos:segundos",
  siempre en tiempo universal.
- **Cambiar un lugar dentro de otro lugar no se puede:**
  `m[0][1] = 5;` da error; hay que guardar el lugar de
  adentro en una variable primero y cambiarlo ahí.
- **El escáner de librerías revisa más cosas desde la 1.2:**
  también mira dentro de `constante`, de `intentar` y de
  cambiar un lugar, y bloquea rutas peligrosas en
  `mover_archivo`, `borrar_archivo`, `listar_carpeta`,
  `crear_carpeta` y `existe`. Aun así no garantiza seguridad
  al 100% (ver arriba).

---

# Eclipse 0.2 — Compilador propio, máquina virtual propia y ByteCode

La versión 0.2 agrega el segundo camino para correr tus programas, sin
cambiar la lógica de Eclipse: el intérprete y la máquina virtual hacen
exactamente lo mismo, solo que por caminos distintos.

## Los dos caminos

```
archivo.ec  →  Lexer  →  Tokens  →  Parser  →  AST
                                                    ├──→  Intérprete  →  resultado
                                                    └──→  Compilador → ByteCode → Máquina Virtual → resultado
```

- **Intérprete** (como antes): recorre el arbolito y lo ejecuta.
- **Compilador + VM** (nuevo): traduce el arbolito a bytecode, una lista
  de instrucciones chiquitas, y la máquina virtual las ejecuta con una pila.

## Cómo usarlo

```bash
# Correr con la máquina virtual
cargo run -- --vm ejemplos/02-decisiones-y-bucles/juego.ec

# Ver el bytecode y correrlo en la máquina virtual
cargo run -- --bytecode ejemplos/01-basico/hola.ec

# Correr con el intérprete (como siempre)
cargo run -- ejemplos/01-basico/hola.ec
```

## Qué es el ByteCode

El bytecode es la receta paso a paso que el compilador escribe para la
máquina virtual. Para `sea x = 10 + 5;` el compilador escribe:

```
0000 Constante 0 (10)
0001 Constante 1 (5)
0002 Sumar
0003 DefinirVariable x (fija)
```

La máquina virtual tiene una pila: `Constante` pone valores arriba,
`Sumar` saca dos valores y pone el resultado. Así corre todo tu programa,
instrucción por instrucción.

## Los archivos nuevos (.rs)

| Archivo | Qué hace |
|---|---|
| `src/bytecode.rs` | Las instrucciones, el pedazo (chunk) con su tabla de constantes, y el desensamblador que muestra el bytecode |
| `src/compilador.rs` | Traduce el AST a bytecode: `si` se vuelve saltos, `mientras` salta hacia atrás, `para` se desarma en un `mientras` con contador |
| `src/vm.rs` | La máquina virtual: pila, marcos (una función corriendo) y ámbitos de variables |

## Cómo traduce el compilador cada cosa

- **`si`**: dos saltos. Si la condición es falsa, salta al `sino`.
- **`mientras`**: un salto hacia atrás. Corre el cuerpo y vuelve a preguntar.
- **`para`**: se desarma en un `mientras` con una variable que cuenta.
  El fin se guarda en una variable escondida llamada `$fin` (empieza con
  `$`, así ningún programa puede chocar con ella).
- **`funcion`**: se compila aparte, en su propio pedazo. Se registra para
  poder llamarla cuando el programa llega a su definición, igual que el
  intérprete: no se puede llamar a una función antes de definirla.
- **`romper` y `seguir`**: son saltos que se parchean cuando ya se sabe
  a dónde van. `seguir` en un `para` salta al incremento, no a la condición.
- **`y` / `o`**: con cortocircuito, como el intérprete: si el primero ya
  decide, el segundo ni se mira.

## Una decisión honesta

La máquina virtual busca las variables **por su nombre**, como el
intérprete. Las máquinas virtuales profesionales usan numeritos (slots)
para ir más rápido. Eso es el paso 3. Por claridad, la 0.2 usa nombres:
así el bytecode se puede leer y entender.

## Qué sigue (0.3)

1. Variables por numeritos (slots) para que la VM corra más rápido.
2. Compilador a `.exe` (código nativo).
3. Autocompletado inteligente de VS Code.

---

# Eclipse 0.3 — Estabilidad, optimización y ownership

La versión 0.3 pule Eclipse sin cambiar su lógica: todo lo que corría
en la 0.2 sigue corriendo igual, en el intérprete y en la máquina
virtual, con los mismos resultados y los mismos errores en español.

## Estabilidad

- **La máquina virtual tiene límites:** máximo 1000 funciones anidadas,
  máximo 10,000 valores en la pila, máximo 10,000,000 de instrucciones.
  Un programa con una función que se llama a sí misma sin parar, o con
  un bucle sin fin, se para solo con un error en español en vez de
  comerse toda la memoria o romperse.
- **El intérprete también tiene límite:** máximo 200 funciones anidadas
  (cada llamada usa pila de Rust).
- **El parser tiene límite:** máximo 100 paréntesis y signos unarios
  anidados, para que un programa demasiado profundo no rompa la pila.
- **Los errores de archivo son claros:** si el archivo no existe dice
  que no lo encontró; si no es texto válido, dice que no es texto válido.
- **`eclipse --version`** muestra la versión.

## Optimización (poca RAM, más rápido)

- **Doblado de constantes:** `10 + 5` se calcula al compilar y queda
  solo el `15`. También funciona anidado: `2 * 3 + 4` queda `10`.
  Míralo con `cargo run -- --bytecode ejemplos/01-basico/hola.ec`.
- **Nombres internados:** cada nombre de variable o función se guarda
  UNA sola vez en la tabla del pedazo; las instrucciones solo llevan
  su numerito. Un nombre largo usado cien veces pesa como uno.
- **Peephole:** el compilador quita los saltos que van a la
  instrucción siguiente (saltar al siguiente paso es no saltar).
- **Constantes sin repetir:** las constantes iguales se guardan una
  sola vez (esto ya estaba en la 0.2).

## Como Rust: ownership

Eclipse 0.3 tiene ownership, como Rust. Cada valor tiene UN solo dueño.
Los números y los lógicos son fotocopias (se copian). Los textos son
originales (se mueven). `copiar(texto)` hace una copia nueva.
Ver la sección "Ownership" arriba y el ejemplo `ejemplos/ownership.ec`.

## Cosas nuevas

- **`segun`** (24ª palabra clave): como el `match` de Rust.
  Ejemplo: `ejemplos/segun.ec`.
- **Asignación con operación:** `+=`, `-=`, `*=`, `/=`, `%=`.
  `puntos += 5;` es lo mismo que `puntos = puntos + 5;`.
- **Comentarios de bloque:** `/* ... */` pueden ocupar varias líneas.
- **`sea x: numero;` sin valor:** ahora se puede crear una variable
  con tipo pero sin valor (vale `nulo`); el tipo se revisa cuando le
  das un valor. Para cambiarla después, usa `sea mut`.
- **Funciones nuevas:** `largo(texto)`, `tipo_de(valor)`,
  `aleatorio()` (0 a 99), `copiar(texto)`.
- **Corrección:** `y` y `o` ahora devuelven verdadero/falso también
  en la máquina virtual cuando los valores no son lógicos
  (en la 0.2 la VM dejaba el valor sin convertir).

---

# Eclipse 0.4 — Slots, autocompletado inteligente y ownership mejorado

La versión 0.4 mejora Eclipse sin cambiar su lógica: todo lo que corría
en la 0.3 sigue corriendo igual, en el intérprete y en la máquina
virtual, con los mismos resultados y los mismos errores en español.
Se probaron los 12 ejemplos y más de 40 casos difíciles: todos dan
exactamente lo mismo en los dos motores.

## Más optimización: la máquina virtual por slots

- **Slots (cajones numerados):** las variables ya no se buscan por su
  nombre en un mapa: cada una tiene un numerito de slot. Las variables
  globales viven en un array compartido; las locales, en los slots del
  marco de la función. Buscar por numerito es mucho más rápido y gasta
  menos RAM. Los slots se reutilizan cuando un bloque se cierra.
  Míralo con `cargo run -- --bytecode ejemplos/01-basico/hola.ec`: ahora verás
  `DefinirVariable x en slot 0 (global, fija)`.
- **Una función solo puede usar las globales que ya existían cuando se
  definió** (como en Rust). Si usas una variable que todavía no existe,
  el compilador te lo dice en español antes de correr nada. Y si una
  función usa una global que se define después de ella, Eclipse la
  encuentra igual (primera pasada), como el intérprete.
- **El doblado de constantes ahora es anidado de verdad:** `2 * 3 + 4`
  queda `10` (en la 0.3 solo se doblaba un nivel).
- **El peephole sigue quitando los saltos inútiles** y los nombres
  siguen internados (una sola vez en la tabla).

## Mejora en la sintaxis

- **Interpolación de textos:** `"Hola, {nombre}, tienes {puntos} puntos"`.
  También con cuentas: `{puntos * 2}`. `{{` y `}}` son llaves literales.
  Ejemplo: `ejemplos/interpolacion.ec`.
- **`para ... paso N` (25ª palabra clave: `paso`):** `para i en 1..10 paso 2`
  avanza de 2 en 2. El paso puede ser negativo (hacia atrás) o una
  variable. Si es 0, el bucle no corre. Ejemplo: `ejemplos/paso.ec`.
- **`!` es lo mismo que `no`:** `!verdadero` es falso.
- **Números con guiones bajos:** `1_000` es lo mismo que `1000`.

## Autocompletado inteligente

- **`eclipse --completar archivo.ec linea columna`** te dice qué puedes
  escribir en esa posición, una sugerencia por línea:
  `nombre<TAB>tipo<TAB>descripción`.
- **La extensión de VS Code lo usa de verdad:** te sugiere tus variables
  (con su tipo), tus funciones (con cuántos datos necesitan), las 25
  palabras clave y las funciones de Eclipse. Solo sugiere lo que se ve
  desde donde estás escribiendo: una variable dentro de un bloque no
  aparece fuera de él, y los datos de una función aparecen dentro de
  su cuerpo. Funciona aunque el archivo esté a medias (trabaja con el
  lexer, no con el parser).
- Para instalarla, mira la sección "VS Code" arriba.

## Ownership mejorado

- **Prestar con `&`:** `saludar(&nombre)` le presta el texto a la
  función sin que se lo quede. Es el `&` de Rust. Solo se usa al llamar
  a una función. Ejemplo: `ejemplos/03-funciones/prestar.ec`.
- **El error te dice A DÓNDE se fue el valor:** *"la variable 'a' se
  movió a la variable 'b'"* o *"se movió a la función 'saludar'"*.
- **`copiar(texto)`** sigue siendo la forma de quedarte con una copia.

## Más cosas

- **Funciones nuevas:** `min(3, 7)`, `max(3, 7)` y `leer_numero()` (lee
  un número que escribe el usuario y da error si no era un número).
- **Ejemplos nuevos:** `interpolacion.ec`, `paso.ec`, `prestar.ec`.
- **La extensión de VS Code** ahora colorea `paso`, `min`, `max`,
  `leer_numero` y el `&`.

---

# Eclipse 0.5 — Compilación nativa a .exe, depurador, LSP en Rust puro y &mut

La versión 0.5 pule Eclipse sin cambiar su lógica: todo lo que corría
en la 0.4 sigue corriendo igual, ahora en **tres motores** (intérprete,
máquina virtual y .exe nativo) con los mismos resultados, los mismos
errores en español y el mismo código de salida. Se probaron los 16
ejemplos y 24 casos difíciles: todos dan exactamente lo mismo en los
tres motores (44 pruebas en total, con el script `probar3.sh`).

## Compilación nativa: tu programa se vuelve un .exe de verdad

```bash
# Generar el .exe (lo compila rustc, que ya viene con Rust)
cargo run -- --exe ejemplos/01-basico/hola.ec
./hola        # en Windows: hola.exe

# Ver el código Rust que se genera (sin compilarlo)
cargo run -- --rust ejemplos/01-basico/hola.ec
```

El transpilador (`src/transpilador.rs`) traduce tu programa `.ec` a
código Rust que usa la librería `rt/eclipse_rt.rs`, y `rustc` lo
convierte en un programa nativo: **corre solo, sin intérprete ni
máquina virtual**. Los archivos generados quedan en una carpeta
`hola_eclipse/` junto a tu `.ec`, y el código Rust es legible: lo
puedes abrir y ver cómo se ve tu programa de Eclipse traducido a
Rust, para aprender Rust con tus propios programas.

- El `.exe` usa las **mismas reglas** que el intérprete y la VM:
  ownership, préstamos, los mismos errores en español.
- Las llamadas anidadas (`doble(sumar(3, 4))`) se evalúan en
  temporales antes de llamar, porque Rust no deja pedir prestado el
  entorno dos veces a la vez.
- Si una función recibe la cantidad equivocada de datos, el `.exe`
  lo dice con el mismo mensaje que los otros motores (solo que
  después de calcular los datos).

## Depurador: corre paso a paso y mira tus variables

```bash
# Paso a paso: Enter avanza, v muestra las variables,
# c corre hasta el final, s para la depuración
cargo run -- --depurar ejemplos/01-basico/hola.ec

# Traza: muestra cada paso y las variables, sin preguntar
cargo run -- --trazar ejemplos/01-basico/hola.ec
```

En cada paso ves la orden que se va a ejecutar y todas las variables
que se ven en ese momento, con su valor y si son fijas o cambian.
Es la mejor forma de entender qué hace tu programa por dentro.
Ejemplos para depurar: `ejemplos/02-decisiones-y-bucles/juego.ec`, `ejemplos/03-funciones/fibonacci.ec`.

## LSP en Rust puro: autocompletado y ayuda en cualquier editor

```bash
# El servidor de lenguaje: habla JSON-RPC por stdin/stdout
cargo run -- --lsp
```

El servidor (`src/lsp.rs` + `src/json.rs`, escritos a mano en Rust,
sin librerías) entiende el protocolo LSP: `initialize`, abrir/cambiar/
cerrar documentos, **completion** (autocompletado) y **hover** (ayuda
al pasar el mouse). El autocompletado reusa el mismo analizador de
`src/autocompletar.rs`, así que sugiere lo mismo que `--completar`:
tus variables con su tipo, tus funciones con sus parámetros, las 26
palabras clave y las funciones de Eclipse. El hover te dice qué es
cada cosa: `"**nombre** (texto)\n\nvariable"`.

La extensión de VS Code ahora es solo un lanzador mínimo
(`vscode-eclipse/extension.js`): enciende `eclipse --lsp` y le pasa
tus mensajes. Toda la inteligencia está en Rust. Otros editores
(Neovim, Helix, Zed) pueden usar `eclipse --lsp` directamente, sin
nada de JavaScript.

## Ownership mejorado: préstamo para cambiar (&mut)

```rust
funcion subir(n) {
    n = n + 5;
    devolver n;
}

sea mut puntos = 10;
subir(&mut puntos);   // la función cambia tus puntos: ahora valen 15
imprimir(puntos);     // 15
```

- **`&mut`** solo vale para **números y lógicos** (que son copias):
  la función recibe el valor, lo cambia, y al terminar Eclipse
  escribe el valor nuevo en tu variable. Es como prestar tu cuaderno
  para que lo escriban y te lo devuelvan.
- **El texto no se puede prestar para cambiar**: te dice
  *"solo se puede prestar para cambiar un número o un lógico, no texto"*.
- **Una variable fija tampoco**: *"no se puede prestar para cambiar
  'puntos' porque es fija"*.
- Si la función cambia el **tipo** (de número a texto), te dice
  cuál era y cuál quedó.
- Funciona igual en los tres motores. Ejemplo: `ejemplos/03-funciones/prestar_mut.ec`.

## Palabras clave y símbolos nuevos

- **`repetir`** (26ª palabra clave): el cuerpo corre **una vez** y
  luego repite mientras la condición sea verdadera. Es el `do-while`
  de otros lenguajes. Ejemplo: `ejemplos/02-decisiones-y-bucles/repetir.ec`.
  ```rust
  repetir {
      imprimir(n);
      n = n + 1;
  } mientras n < 3;
  ```
- **`**` (potencia):** `2 ** 3` es 8. Se lee de derecha a izquierda:
  `2 ** 3 ** 2` es 512. Manda más que `*`: `2 * 3 ** 2` es 18.
  Ejemplo: `ejemplos/02-decisiones-y-bucles/potencia.ec`.

## Sin Garbage Collector (por diseño)

Eclipse **no tiene** recolector de basura, y no lo necesita:

- Eclipse está hecho en **Rust**, y en Rust cada valor tiene UN solo
  dueño: cuando el dueño deja de usarse, Rust libera su memoria solo,
  sin un programa corriendo en segundo plano que la recoja.
- El ownership de Eclipse (mover textos, copiar números) es la misma
  idea: nada se queda sin dueño, nada se libera dos veces.
- El `.exe` también es Rust puro: la librería `rt/eclipse_rt.rs` no
  usa `Rc` ni contadores de referencias ni recolector. Solo dueño,
  préstamo y liberación automática (RAII).
- Por eso Eclipse gasta poca RAM: los nombres se internan (una sola
  vez), las constantes se doblan al compilar y la VM usa slots.

## Más cosas

- **Ejemplos nuevos:** `adivina.ec` (adivina el número secreto con
  `aleatorio()` y `leer_numero()`), `potencia.ec`, `repetir.ec`,
  `prestar_mut.ec`.
- **Compilación sin advertencias** (`cargo build` y
  `cargo build --release` no muestran ningún warning).
- **Corrección:** el mensaje de `a_numero` ahora es el mismo en los
  tres motores (*"eso no era un número: '...'"*).
- **La extensión de VS Code** ahora colorea `repetir` y es un lanzador
  LSP mínimo. La versión es 0.5.0.
- **Pruebas:** `probar3.sh` compara intérprete, VM y .exe en los 16
  ejemplos y en 24 casos difíciles (`pruebas05/`); `probar_lsp.py`
  prueba el servidor LSP por JSON-RPC.

# Eclipse 0.6 — Ownership avanzado, listas, leer como Input, LSP mejorado y botón Run

La versión 0.6 pule Eclipse sin cambiar su lógica: todo lo que corría
en la 0.5 sigue corriendo igual, ahora en **tres motores** (intérprete,
máquina virtual y .exe nativo) con los mismos resultados, los mismos
errores en español y el mismo código de salida. Se probaron los 25
ejemplos (con entrada por teclado en la calculadora y en `leer`) y 20
casos difíciles: todos dan exactamente lo mismo en los tres motores
(45 pruebas en total, con el script `probar3.sh`).

## Ownership avanzado

- **Constantes:** `constante PI = 3.14;` nunca cambia ni se mueve.
  Asignarle un valor da error en los tres motores.
- **`&mut` para listas:** `meter(&mut numeros)` deja que la función
  cambie tu lista y te la devuelva cambiada, con la misma regla de
  siempre: si la función cambia el tipo, da error con su nombre.
- **Préstamos que no se mueven:** un dato que pasaste con `&` o
  `&mut` no se puede mover adentro de la función. Antes, el `.exe`
  no marcaba ese préstamo (un error real de la 0.5, ya arreglado).
- Los errores de mover ahora dicen el nombre de la variable en los
  tres motores (*"no se puede mover 'nombre' porque está prestado..."*).

## 3 palabras clave nuevas, 1 papel nuevo para `leer` y 1 símbolo nuevo

Pediste 4 palabras clave nuevas, y aquí está la cuenta honesta:
`lista`, `para_cada` y `constante` son 3 palabras nuevas de verdad.
La cuarta es **`leer` con un papel nuevo**: ya era palabra clave
desde la 0.1 (la función `leer()`), y ahora también es una orden de
Input. En total, Eclipse tiene 29 palabras clave.

- **`leer` como Input:** `leer nombre;` lee lo que escribe el usuario
  y lo guarda. Convierte a número si la variable era un número.
  La función `leer()` sigue igual (con paréntesis).
- **`lista`:** el tipo de las listas: `sea numeros: lista = [1, 2, 3];`.
  Las listas se imprimen `[1, 2, 3]`, se leen con `[0]`, se pegan
  con `+`, y se mueven como los textos. Funciones nuevas: `agregar`
  (devuelve una lista nueva) y `elemento` (lo mismo que `[ ]`).
- **`para_cada`:** `para_cada n en numeros { ... }`, con `romper` y
  `seguir`. La variable del bucle es fija adentro.
- **`constante`:** `constante PI = 3.14;` — valor obligatorio, nunca
  cambia, nunca se mueve.
- **`++` y `--`:** `puntos++` suma 1, `puntos--` resta 1 (solo
  números, con `sea mut`).
- **Compatibilidad:** `sea lista = 5;` sigue funcionando (las palabras
  nuevas son contextuales: si las usas como nombre de variable, siguen
  siendo tu variable). Y `5 - -3` sigue siendo 8.

## Bugs arreglados (encontrados al probar, con honestidad)

- **El `.exe` no movía los textos que le pasabas a una función:**
  `saludar(a); imprimir(a);` imprimía el texto dos veces en el `.exe`
  y daba error (bien) en el intérprete y la VM. Ahora el transpilador
  mueve el texto antes de llamar, igual que los otros motores.
- **El `.exe` se rompía con `x = f(y);`:** el código generado pedía
  prestado el entorno dos veces a la vez (E0499). Ahora todas las
  expresiones van a una temporal antes de `asignar` o `definir`.
- **`10 % 0` daba NaN en la VM:** el compilador doblaba la constante
  `10 % 0` a NaN al compilar. Ahora el resto entre cero no se dobla
  y da el error en español en los tres motores.
- **El intérprete se caía (panic) con un `para` cerca del número
  máximo:** `para i en 9223372036854775800..=9223372036854775807`
  sumaba y se desbordaba. Ahora usa suma revisada y se detiene solo.
- **El mensaje de cambiar el tipo en el `.exe` no decía qué función
  fue:** ahora los tres motores dicen *"la función 'cambiar' cambió
  el tipo de 'puntos': era numero y ahora es texto"*.
- **Las funciones de la VM no revisaban cuántos datos les dabas:**
  `largo()` sin datos o `a_numero(verdadero)` daban mensajes raros
  o distintos a los del intérprete. Ahora la VM revisa la cantidad
  y usa los mismos mensajes exactos.
- **El archivo de prueba `pruebas05/muttipo.ec` estaba mal:** usaba
  `sea puntos` (fija) en vez de `sea mut puntos`, así que el camino
  de cambiar el tipo nunca se probaba de verdad. Ya está corregido.
- **Clippy limpio:** `cargo clippy --all-targets` no muestra ninguna
  advertencia, y los `unwrap()` peligrosos se cambiaron por errores
  en español (o por `expect` con su motivo, donde el valor siempre
  existe).

## Código profesional (sin cambiar las lógicas)

- **Carpetas ordenadas:** `src/lenguaje/` (lexer, parser, AST),
  `src/motor/` (intérprete, compilador, bytecode, VM, transpilador),
  `src/herramientas/` (autocompletado) y `src/lsp/` (servidor y JSON).
  Los ejemplos están por tema: `01-basico` a `06-listas`.
- **El `.exe` ahora recibe los préstamos `&`:** la función generada
  recibe una cuarta lista (`prestados`) y define esos datos como
  prestados, para que no se puedan mover.
- **`cargo build`, `cargo build --release` y `cargo clippy
  --all-targets` no muestran ninguna advertencia.**

## LSP mejorado: diagnósticos y esquema

- **Diagnósticos:** al abrir o cambiar un archivo, el servidor te
  avisa de su primer error con la línea exacta y el mensaje en
  español (por ejemplo, *"esperaba un valor, un nombre o '('"* en la
  línea 2). Si no hay errores, borra los avisos de antes.
- **Esquema del documento (`documentSymbol`):** tus variables,
  constantes y funciones, con su clase (variable o función) y su
  línea, para el panel de esquema de tu editor.
- **Autocompletado al día:** las 29 palabras clave, las 15 funciones
  (con `agregar` y `elemento`), las constantes, las variables de
  `para_cada` y el tipo `lista` (de `sea x: lista` o de `[1, 2]`).
- Prueba: `python3 probar_lsp.py` (TODO OK).

## Botón ▶ Ejecutar en VS Code

- Un botón ▶ arriba a la derecha del editor (solo en archivos `.ec`),
  en la paleta de comandos ("Eclipse: Ejecutar archivo"), con el
  atajo **Ctrl+Alt+R** (Cmd+Alt+R en Mac) y un enlace "▶ Ejecutar"
  arriba de tu archivo.
- Corre tu archivo en la terminal integrada de VS Code (la terminal
  se llama "Eclipse" y se reusa), guardándolo primero.
- Prueba: `node vscode-eclipse/probar_extension.js`.

## Más cosas

- **Ejemplos nuevos:** `ejemplos/06-listas/listas.ec`,
  `leer_input.ec` (con entrada por teclado), `constante.ec`,
  `prestar_lista.ec`.
- **Pruebas:** `probar3.sh` compara los tres motores en los 25
  ejemplos y 20 casos difíciles (`pruebas05/`); `probar_lsp.py`
  prueba el LSP (incluye diagnósticos y esquema);
  `node vscode-eclipse/probar_extension.js` prueba la extensión
  (incluye el botón Run); el depurador se probó con `--trazar` y
  `--depurar`.

# Eclipse 0.7 — Todo en Rust, proyectos, paquete standalone y más poder

La versión 0.7 pule Eclipse sin cambiar su lógica: todo lo que
corría en la 0.6 sigue corriendo igual, ahora en **tres motores
más el paquete** (intérprete, máquina virtual, .exe nativo y
ejecutable standalone) con los mismos resultados, los mismos
errores en español y el mismo código de salida. Se probaron
26 ejemplos y 23 casos difíciles en los 3 motores + el paquete
(51 pruebas en total, con el probador nuevo en Rust),
más el LSP (TODO OK) y los proyectos.

## Todo en Rust: adiós Python y Shell

- Las pruebas ya no usan Python ni Shell: `probar3.sh` ahora
  es **`src/bin/probar.rs`** y `probar_lsp.py` es
  **`src/bin/probar_lsp.rs`**, todo en Rust puro, con la
  librería estándar. Se corren con `cargo run --bin probar`
  y `cargo run --bin probar_lsp`.
- El probador nuevo compara los 3 motores + el paquete en
  cada archivo, con tiempo máximo de 10 segundos por programa
  (si un programa se cuelga, lo mata y lo marca como falla),
  y también prueba los proyectos y la extensión.
- Lo único que no es Rust en todo el proyecto es el
  `extension.js` de VS Code, porque VS Code solo acepta
  extensiones en JavaScript (la inteligencia sigue siendo
  100% Rust).

## Comandos propios en la terminal (como Rust)

- **`eclipse instalar`**: copia Eclipse a `~/.eclipse/bin`,
  lo agrega al PATH y instala la extensión de VS Code (los
  archivos van incrustados dentro del propio Eclipse, así
  que no descarga nada). **`eclipse desinstalar`** lo quita.
- **`eclipse version` / `--version` / `-v`**: muestra la versión.
- **`eclipse --ayuda`**: todos los comandos, en español.

## Proyectos (como Cargo)

- **`eclipse nuevo mi_juego`**: crea la carpeta, el
  `Eclipse.toml` y el `src/principal.ec`.
- **`eclipse construir`**: compila TODOS los .ec del proyecto
  con **barra de progreso** (`[##########----] 3/5 (60%)`).
  Con **`--release`** además crea el ejecutable standalone.
- **`eclipse correr`**: corre un archivo, o el principal del
  proyecto si no le das archivo. Con `--vm`, en la VM.
- **`eclipse correr-todos [carpeta]`**: corre TODOS los .ec
  de la carpeta, uno por uno, con barra de progreso y un
  resumen (cuántos pasaron y cuáles fallaron).
- Los archivos se enlazan con `importar`, como siempre.
  Ejemplo: `ejemplos/07-proyectos/mi_juego/`.

## Ejecutable standalone: Eclipse ya no depende de Rust

- **`eclipse empaquetar juego.ec`**: crea UN SOLO archivo
  (Eclipse + tu programa en bytecode, formato ECBIN) que
  corre en cualquier computadora sin instalar nada: no
  necesita Rust, ni Cargo, ni `rustc`.
- El formato ECBIN es binario y chiquito (marca, versión,
  pedazos, constantes, nombres y código). El ejecutable,
  al arrancar, mira si trae un paquete pegado al final y,
  si lo trae, corre ese programa en su máquina virtual.
- Con honestidad: no es código máquina (eso sigue siendo
  `eclipse --exe` con `rustc`). Es Eclipse + bytecode en
  un solo archivo, y para quien lo corre es lo mismo.

## Mejoras en `leer` y en los símbolos

- **`leer` con mensaje:** `leer "¿Cómo te llamas? " nombre;`
  muestra el mensaje sin salto de línea, para que el usuario
  escriba en la misma línea.
- **`leer` para listas:** `leer numeros;` con una lista
  separa lo escrito por comas; si la lista era de números,
  cada parte debe ser un número; si estaba vacía, decide
  sola (todo números -> números, si no, textos).
- **Símbolo nuevo `**=`:** `puntos **= 3;` es lo mismo que
  `puntos = puntos ** 3;` (2 **= 3 es 8).

## Más funciones

Seis funciones nuevas, en los 4 motores (intérprete, VM,
.exe y paquete), con autocompletado y colores:

- **`ahora()`**: los segundos desde 1970 (el reloj).
- **`redondear(3.7)`** -> 4; **`absoluto(-5)`** -> 5.
- **`mayusculas("hola")`** -> "HOLA";
  **`minusculas("HOLA")`** -> "hola".
- **`contiene("hola kevin", "kevin")`** -> verdadero.

## Mejoras en LSP y autocompletado

- **Definition:** salta a donde se definió la variable o
  función que tienes debajo del ratón.
- **References:** todos los lugares donde se usa esa palabra.
- El autocompletado ya sugiere las 6 funciones nuevas, el
  tipo `lista` y las variables de `para_cada`.
- Prueba: `cargo run --bin probar_lsp` (TODO OK: initialize,
  diagnósticos, completion, hover, documentSymbol,
  definition, references, archivo roto, shutdown).

## Mejoras en Run y Debug

- **Dos comandos Run más en VS Code:** Ejecutar con máquina
  virtual (Ctrl+Alt+V) y Ejecutar con depurador (Ctrl+Alt+D),
  en la paleta de comandos.
- **Depurador:** ahora puedes ver una variable con
  `p nombre` (sin verlas todas), y al final te dice
  cuántos pasos corrió y cuánto tardó
  ("--- depurador: 7 pasos en 0.00 segundos ---").

## Más estabilidad y código profesional

- **`cargo clippy --all-targets` no muestra ninguna
  advertencia**, y `cargo build` (debug y release) tampoco.
- El probador nuevo tiene tiempo máximo por programa, así
  que un bucle sin fin no cuelga las pruebas.
- Se encontró y se dejó documentado que `y` no puede ser
  nombre de variable (es la palabra del Y lógico).
- Ejemplos nuevos: `ejemplos/07-texto/texto.ec` y el proyecto
  de ejemplo `ejemplos/07-proyectos/mi_juego/`; casos
  difíciles nuevos: `leer_lista.ec`, `potencia_igual.ec`,
  `texto_funciones.ec`.

# Eclipse 0.8 — Auditoría, rendimiento y sintaxis pulida

La versión 0.8 no trae palabras clave nuevas ni símbolos nuevos:
es una versión de **arreglar bugs, pulir la sintaxis y correr más
rápido**, sin cambiar ninguna lógica. Todo lo que corría en la 0.7
sigue corriendo igual. Se probaron 28 ejemplos, 27 casos difíciles
nuevos y 23 casos difíciles viejos en los 3 motores + el paquete
(78 pruebas en total, con el probador en Rust), más el LSP
(TODO OK) y los proyectos.

## Auditoría: 6 bugs reales, encontrados y arreglados

Se escribió una batería nueva (`pruebas08/`, 27 archivos) que
prueba los bordes de cada palabra y cada símbolo en los 3
motores + el paquete, y que también compara los mensajes de
error (el probador ahora compara el stderr, no solo la salida).
Los bugs que encontró, todos arreglados:

1. **`nulo` no se podía usar como patrón de `segun`:**
   `segun x { nulo => { ... } }` daba error. Ahora `nulo`
   es un patrón válido, como los números y los textos.
2. **`{}` vacío en un texto daba un error confuso:**
   `"Vacío: {}"` fallaba con un error doble. Ahora `{}` vacío
   se imprime tal cual (las llaves literales siguen siendo
   `{{` y `}}`).
3. **`importar` anidado buscaba en la carpeta equivocada:**
   si `a.ec` importaba a `b.ec` de otra carpeta, y `b.ec`
   importaba a `c.ec` de SU carpeta, el intérprete y la VM
   buscaban `c.ec` en la carpeta del programa principal y
   fallaban, mientras el `.exe` lo encontraba. Ahora los 3
   motores buscan desde la carpeta del archivo que importa.
4. **`agregar` clonaba la lista entera en la VM y en el .exe:**
   el intérprete la movía y empujaba en su lugar (O(1)), pero
   la VM y el runtime la clonaban (O(n) por llamada). Ahora
   los 3 la mueven.
5. **`empaquetar` con una ruta de salida que ya traía carpeta
   la duplicaba** (`banco/fib.paq` se volvía `banco/banco/
   fib.paq`). Ahora, si la salida ya trae carpeta, se usa tal cual.
6. **La VM se quedaba corta con bucles largos:** su límite
   era 10,000,000 de instrucciones y un bucle de 1,000,000
   de vueltas lo tocaba. Ahora el límite es 100,000,000
   (sigue parando los bucles sin fin).

Además, el probador ahora trata bien los programas que DEBEN
fallar: si el intérprete falla con un error y el `.exe` falla
con el mismo error, es que los motores están de acuerdo, no
una falla. Hay 8 programas así en `pruebas08/` (romper fuera
de un bucle, dividir entre cero, variable movida, etc.).

## Rendimiento: la VM corre más rápido, con números reales

El banco de velocidad (`cargo run --bin bench`, el mejor de
3 corridas, en milisegundos, medido en esta computadora):

| programa | intérprete | VM | .exe | paquete |
|---|---:|---:|---:|---:|
| fib(20) | 26.92 | 12.55 | 10.12 | 12.47 |
| bucle (sumar 1..1M) | 358.72 | 357.01 | 272.94 | 317.55 |
| para (sumar 1..1M) | 275.39 | 346.33 | 449.64 | 356.59 |
| texto (10,000 pegados) | 21.50 | 20.70 | 22.39 | 20.52 |
| lista (10,000 agregar) | 531.07 | 537.21 | 780.59 | 529.78 |

Qué se hizo para correr más rápido:

- **Las instrucciones ya no se clonan:** `Instruccion` ahora
  es `Copy` (solo guarda números), así que la VM ya no copia
  cada instrucción antes de ejecutarla.
- **Las listas viven en un `Rc`:** copiar una lista (al
  leerla de una variable o pasarla a una función) solo sube
  un contador, no copia los elementos. Solo se copia de
  verdad cuando alguien la cambia y hay más de un dueño
  (`agregar` con `Rc::make_mut`).
- **La VM ya no clona el nombre de cada variable en cada
  acceso:** `obtener`/`asignar`/`mover` reciben el número del
  nombre y solo lo buscan si hay un error. En un bucle, eso
  era una copia de texto por cada lectura y cada escritura.
- **El `para` con paso fijo ya no evalúa la condición
  completa:** si el paso es 1 (o un número fijo), la condición
  es solo `var <= fin` o `var >= fin`, como el bucle nativo
  del intérprete. Antes eran 25 instrucciones por vuelta.
- **Los límites de la VM se revisan cada 1024
  instrucciones,** no en cada una.

Con honestidad: 0.01 ms por programa no es real (un millón
de sumas tarda ~300 ms en cualquier motor), y ningún programa
puede ser más rápido que Rust (el `.exe` de Eclipse ES código
Rust). La VM es 2.2x más rápida que el intérprete en recursión
y empata en bucles `mientras` (1.0x); en `para` el intérprete
sigue ganando porque corre un bucle nativo. Los números están
arriba, medidos, no prometidos.

## Tamaño: los scripts ya son de menos de 100 KB

- **El archivo `.ec` más grande tiene 1,293 bytes** (1.3 KB).
  Los 77 archivos `.ec` del proyecto juntos pesan 19,396 bytes.
  Todos están muy por debajo de 100 KB.
- **El programa Eclipse en release pesa 1.3 MB** (1,369,848
  bytes); con `strip` baja a 1.1 MB. El ejecutable standalone
  (`empaquetar`) pesa lo mismo que Eclipse, porque es Eclipse
  + tu programa: no puede pesar menos de 100 KB, porque lleva
  dentro toda la máquina virtual. Para paquetes más chicos,
  compila Eclipse en release y pásale `strip` antes de
  empaquetar.

## Sintaxis pulida (sin palabras ni símbolos nuevos)

- **Coma final permitida:** ahora puedes escribir `[1, 2, 3,]`,
  `suma(10, 20,)` y `funcion suma(a, b,) { ... }` con una coma
  al final, como en Rust. Antes daba error.
- **Sin palabras nuevas ni símbolos nuevos:** las 29 palabras
  clave y los símbolos son los mismos de la 0.7. Solo se
  arreglaron los bugs de los que ya había.

## 5 funciones nuevas (sin palabras ni símbolos nuevos)

- **`ordenar([3, 1, 2])`** -> `[1, 2, 3]`; con textos, por
  abecedario. Devuelve una lista NUEVA (la vieja no cambia).
- **`invertir([1, 2, 3])`** -> `[3, 2, 1]`; con texto,
  `"hola"` -> `"aloh"`.
- **`unir(["a", "b"], ", ")`** -> `"a, b"`.
- **`separar("a,b,c", ",")`** -> `["a", "b", "c"]`.
- **`raiz(16)`** -> `4`; un número negativo da error en español.
- Las 5 están en los 4 motores (intérprete, VM, .exe y
  paquete), con autocompletado y colores en VS Code.
- Ejemplos nuevos: `ejemplos/08-texto-listas/texto_listas.ec`
  y `ejemplos/08-texto-listas/nombres.ec`.

## Más estabilidad y código profesional

- **`cargo clippy --all-targets` no muestra ninguna
  advertencia**, y `cargo build` (debug y release) tampoco.
- El probador nuevo compara también los mensajes de error
  (stderr) en los 3 motores + el paquete.
- Ejemplos nuevos en `ejemplos/08-texto-listas/`; casos
  difíciles nuevos en `pruebas08/` (27 archivos).

## Qué sigue (0.9)

1. Fecha y hora con formato (`ahora` ya da los segundos).
2. Más listas: filtrar y buscar (ordenar ya está).
3. Compilador a `.exe` sin `rustc` (código máquina directo).

---

# Eclipse 0.9 — Máquina virtual más rápida y auditoría

La versión 0.9 tampoco trae palabras clave nuevas ni símbolos
nuevos: es una versión de **arreglar bugs y de correr más rápido**,
sin cambiar ninguna lógica. Todo lo que corría en la 0.8 sigue
corriendo igual. Se probaron 32 ejemplos, 23 casos difíciles
viejos, 25 casos de la 0.8 y 23 casos nuevos de la 0.9 en los 3
motores + el paquete (105 pruebas en total, con el probador en
Rust), más el LSP (TODO OK) y los proyectos.

## Auditoría: 6 bugs reales, encontrados y arreglados

Se escribió una batería nueva (`pruebas09/`, 23 archivos) que
prueba los bordes de cada palabra y cada símbolo en los 3
motores + el paquete, y que también compara los mensajes de
error (el probador compara el stderr, no solo la salida).
Los bugs que encontró, todos arreglados:

1. **El `.exe` no respetaba el atajo de `y` y `o`:** el
   intérprete y la VM no llaman al lado derecho si el izquierdo
   ya decide (`falso y lado()` no llama a `lado()`), pero el
   `.exe` sí lo llamaba, porque al preparar la llamada sacaba
   los datos del lado derecho antes de decidir. Ahora el
   `.exe` mete el lado derecho dentro de la rama que decide, y
   los 3 motores dan lo mismo, incluso cuando el lado derecho
   mueve una variable (`falso y f(t)` no mueve `t`).
2. **La recursión paraba distinto en cada motor:** el
   intérprete se caía con 250 llamadas anidadas (su pila de
   Rust era chiquita), la VM paraba en 1000 con un error en
   español y el `.exe` no paraba nunca (se rompía con un error
   del sistema, sin palabras en español). Ahora los 3 paran en
   1000 con el mismo mensaje: "demasiadas funciones anidadas,
   ¿tienes una función que se llama a sí misma sin parar?".
   Para eso el intérprete ahora corre en un hilo con una pila
   grande (128 MB) y sus listas pasaron de `Rc` a `Arc` (el
   `.exe` sigue con `Rc`, porque corre en un solo hilo).
3. **`leer` en una constante daba otro mensaje en el `.exe`:**
   el intérprete y la VM decían "Si quieres que el usuario la
   escriba, créala con 'sea mut'" y el `.exe` decía otra cosa.
   Ahora los 3 dicen lo mismo.
4. **`para_cada` con un texto daba otro error en la VM y el
   `.exe`:** el intérprete decía "'para_cada' necesita una
   lista, no texto" y los otros daban el error de `[]`. Ahora
   hay una instrucción nueva, `VerificarLista`, que revisa el
   tipo antes de empezar el bucle, y los 3 dicen lo mismo.
5. **El `para` con decimales cortaba los números en el
   intérprete:** `para i en 1.5..=3.5` daba 1, 2, 3 y un paso
   de 0.5 no corría nunca (0.5 cortado a entero es 0). La VM
   y el `.exe` ya usaban decimales. Ahora el intérprete
   también: 1.5, 2.5, 3.5 y 0, 0.5, 1, 1.5, 2.
6. **El `para` con `seguir` se rompía en la VM:** `para i en
   1..=5 { si i == 3 { seguir; } imprimir(i); }` daba "la
   pila se quedó vacía" en la VM (los otros motores daban
   1, 2, 4, 5). La instrucción fusionada `ParaAvanzar` guarda
   a dónde volver, pero el optimizador de saltos (peephole)
   movía las instrucciones sin mover ese destino. Ahora el
   peephole también mueve el destino de `ParaAvanzar`.

Además, una diferencia que se encontró y se dejó como está
(a propósito, documentada en los límites): un `para` con
texto como límite da error en los dos motores, pero con
palabras distintas.

## Rendimiento: la VM ya le gana al intérprete en `para`

El banco de velocidad (`cargo run --release --bin bench`, el
mejor de 3 corridas, en milisegundos, medido en esta
computadora):

| programa | intérprete | VM | .exe | paquete |
|---|---:|---:|---:|---:|
| fib(20) | 35.71 | 12.89 | 10.11 | 13.01 |
| bucle (sumar 1..1M) | 348.23 | 321.31 | 276.53 | 347.61 |
| para (sumar 1..1M) | 281.01 | 189.00 | 450.90 | 182.92 |
| texto (10,000 pegados) | 22.60 | 21.01 | 22.60 | 21.04 |
| lista (10,000 agregar) | 8.82 | 9.60 | 11.48 | 9.90 |

Qué se hizo para correr más rápido:

- **El `para` avanza en una sola instrucción (`ParaAvanzar`):**
  antes, cada vuelta gastaba 5 instrucciones (leer la variable,
  leer el paso, sumar, guardarla, saltar). Ahora es una sola
  que lee, suma, guarda y decide si seguir. El `para` pasó de
  346 ms a 189 ms en la VM (1.5x más rápida que el intérprete),
  y hasta le gana al `.exe` (451 ms) en ese programa, porque
  el `.exe` busca cada variable en un mapa.
- **`x = agregar(x, v)` mete el valor en su sitio
  (`AgregarEnSitio`):** antes, cada `agregar` copiaba la lista
  entera, porque `agregar` solo mira la variable (no se queda
  con ella). Armar una lista de 10,000 elementos era 10,000
  copias: 530 ms. Ahora, cuando el resultado va a la MISMA
  variable, se mete el valor sin copiar nada (9 ms, unas 55
  veces más rápido). Si la lista la tiene otro también (por
  ejemplo, porque la copiaste con `copiar`), se clona como
  antes, para que la otra no cambie. Y las revisiones son las
  mismas: variable movida, no es lista, constante o fija dan
  el mismo error en los 3 motores.

Con honestidad: "igual que Rust o más rápido" no se puede
prometer en general, porque el `.exe` de Eclipse ES código
Rust y corre a velocidad Rust. Lo que sí se puede (y se hizo)
es acercar la VM a esa velocidad en los programas que más se
usan: en recursión la VM es 2.8x más rápida que el intérprete,
en `para` 1.5x, en `mientras` 1.1x, y en listas los 3 motores
ahora empatan en ~9 ms. Los números están arriba, medidos, no
prometidos.

## Tamaño: los scripts siguen siendo de menos de 100 KB

- **El archivo `.ec` más grande tiene 1,293 bytes** (1.3 KB).
  Los 108 archivos `.ec` del proyecto juntos pesan 24,461 bytes.
  Todos están muy por debajo de 100 KB.
- **El programa Eclipse en release pesa 1.4 MB** (1,481,136
  bytes); con `strip` baja a 1.2 MB. El ejecutable standalone
  (`empaquetar`) pesa lo mismo que Eclipse, porque es Eclipse
  + tu programa: no puede pesar menos de 100 KB, porque lleva
  dentro toda la máquina virtual.

## Ejemplos nuevos y más estabilidad

- Ejemplos nuevos en `ejemplos/09-rapido/`: `para_decimal.ec`
  (decimales en el `para`), `lista_grande.ec` (armar una lista
  de 1,000 y sumarla), `corto_circuito.ec` (el atajo de `y` y
  `o` con una función que cuenta sus llamadas),
  `recursion.ec` (una función que baja de 500 a 0).
- Casos difíciles nuevos en `pruebas09/` (23 archivos), que
  ahora también corren en el probador por defecto.
- **`cargo clippy --all-targets` no muestra ninguna
  advertencia**, y `cargo build` (debug y release) tampoco.

## Qué sigue (1.0)

1. Fecha y hora con formato (`ahora` ya da los segundos).
2. Más listas: filtrar y buscar (ordenar ya está).
3. Compilador a `.exe` sin `rustc` (código máquina directo).

---

# Eclipse 1.0 — Borrow Checker, librerías y hablar con el sistema

## El Borrow Checker (el compilador regaña antes de correr)

- Nuevo archivo `src/motor/verificador.rs`: análisis estático que
  corre **antes** de ejecutar, en los 3 motores, en el LSP (como
  diagnóstico) y al publicar una librería. El REPL no lo corre
  (cada línea usa variables de las anteriores).
- Para poder decir la línea exacta, **todas** las declaraciones
  del AST ahora llevan su `linea` (antes el AST no tenía líneas).
- Todos los mensajes empiezan con "Oye, " y dicen qué pasó, dónde
  y cómo se arregla ("Si quieres quedarte con una copia, usa
  copiar(a)").
- Reglas: una variable solo tiene UN dueño; regalar (`sea b = a;`,
  `b = a;`, pasarla a una función) la mueve; los números y los
  lógicos son copias; los builtins nunca mueven; `(a)` entre
  paréntesis es una copia; `devolver` no mueve; `x = agregar(x, v)`
  solo pide prestado (como el AgregarEnSitio).
- Fusión de ramas: un movimiento dentro de un `si` sin `sino` o
  dentro de un bucle es "quizá" (el compilador te deja correr y
  el motor te para si de verdad pasó); movida en las DOS ramas
  de un `si`/`sino` o dentro de un `repetir` es definitivo
  (error antes de correr). Los movimientos dentro de una función
  se tratan como "quizá" después (puede que nunca se llame).
- También revisa: variable desconocida, función desconocida
  ("¿te falta un 'usar' arriba?"), cantidad de datos, tipo
  declarado en `sea`, `++`/`--` con algo que no es número,
  `&mut` en una fija o dos veces a la vez, `devolver` fuera
  de función, `romper`/`seguir` fuera de bucle.
- `importar` solo revisa que el archivo exista y qué funciones
  trae; `usar` revisa las firmas de la librería instalada.

## Las librerías (la palabra nueva `usar`)

- **Palabra clave número 30: `usar`.** Solo arriba del archivo,
  una por renglón (2, 3 o 4 librerías). En cualquier otro lugar
  da error de parser ("'usar' solo vale arriba del archivo...").
- Nuevo archivo `src/herramientas/bibliotecas.rs`: registro en
  `~/.eclipse/bibliotecas/<nombre>/` con `<nombre>.ec` +
  `biblioteca.toml` (nombre, versión, autor, descripción y
  número de control FNV-1a del contenido).
- Comandos nuevos: `eclipse biblioteca publicar <archivo>.ec`,
  `listar`, `info <nombre>`, `quitar <nombre>`.
- Seguridad al publicar: tamaño máximo 500 KB; el nombre no
  puede tener palabras peligrosas (firmware, bios, malware,
  virus, troyano, keylogger, ransomware, driver, kernel,
  rootkit, spyware, gusano); el código no puede tener palabras
  en inglés (unas 60: let, if, print, function...); no puede
  llamar a `comando`; no puede usar archivos fuera de su
  carpeta (`..`, rutas absolutas, letra de unidad); el Borrow
  Checker la revisa antes de aceptarla. Al usarla, se revisa
  el número de control: si cambió, error.
- Los 3 motores resuelven `usar` con el mismo camino que
  `importar` (la librería se mete en el programa al compilar),
  así que también funciona en el paquete standalone y en el
  `.exe`. Las 3 librerías de ejemplo están en `bibliotecas/`:
  `matematicas` (factorial, es_par, es_primo, mcd, mcm,
  fibonacci, promedio), `juegos` (dado, moneda, elegir,
  tirar_dados) y `dibujo` (limpiar_pantalla, color_texto,
  linea, caja, titulo).
- Dos detalles que se encontraron al publicarlas (y que ya
  estaban documentados): `y` no puede ser nombre de variable
  y `numero`/`texto` no pueden ser nombres de parámetros ni
  de variables de `para_cada`.

## Hablar con el sistema (hardware y software)

- Nuevo archivo `src/motor/sistema.rs` (solo librería estándar):
  `sistema()` (Windows/Linux/macOS), `usuario()`, `carpeta()`,
  `variable_entorno()`, `comando()` (cmd en Windows, sh en el
  resto; devuelve lo impreso, error con el código de salida),
  `argumentos()`, `procesadores()`, `memoria()` (Linux lee
  /proc/meminfo; Windows usa wmic; macOS usa sysctl).
- 9 funciones nuevas en los 3 motores + el `.exe` (el runtime
  lleva su propia copia del módulo `sistema`, porque el `.exe`
  no ve `src/`): `sistema`, `usuario`, `carpeta`,
  `variable_entorno`, `comando`, `argumentos`, `procesadores`,
  `memoria` y `caracter` (la letra de un número, para que
  `dibujo` pueda emitir el carácter ESC).
- **Honestidad:** Eclipse corre como un programa normal: le
  pide cosas al sistema operativo y corre otros programas.
  No toca el hardware directo (sensores, controladores): eso
  es trabajo del núcleo del sistema. Y `dibujo` dibuja en la
  terminal, no en ventanas de verdad.

## Más velocidad: el compilador a `.exe` es mejor

- El `para` del transpilador era el más lento del banco (451 ms
  en la 0.9, contra 189 ms de la VM): cada vuelta buscaba la
  variable, el fin y el paso en un mapa. Ahora la variable que
  cuenta, el fin y el paso viven en **variables locales de
  Rust** (`let mut __v1`, `while __v1 <= __v2`), con la
  condición en Rust directo y el incremento como una suma. El
  cuerpo sigue viendo la variable en el entorno (se sincroniza
  en cada vuelta, y `seguir` también la sincroniza antes de
  saltar), así que por fuera nada cambia.
- **La suite encontró un error en este cambio antes de
  entregar:** la variable local se declaró sin `mut` y 13
  programas fallaron al compilar el `.exe` (rustc E0384).
  Se arregló con `let mut` y la suite volvió a 0 fallas.
- Números medidos (mejor de 3, en milisegundos, release):

  | programa | intérprete | VM | .exe | paquete |
  |---|---|---|---|---|
  | fib.ec | 35.7 | 12.6 | 10.1 | 12.8 |
  | bucle.ec | 356.3 | 294.8 | 272.3 | 311.3 |
  | para.ec | 278.3 | 166.6 | **234.4** | 168.0 |
  | texto.ec | 23.0 | 21.1 | 22.7 | 21.6 |
  | lista.ec | 9.3 | 10.0 | 11.1 | 9.1 |

  El `.exe` en `para` pasó de 451 ms a 234 ms (casi 2x más
  rápido). La VM sigue ganando en `para` (167 ms) porque el
  cuerpo del `.exe` todavía busca `total` e `i` en el entorno
  en cada vuelta. En recursión la VM es 2.8x más rápida que
  el intérprete. "Igual que Rust o más rápido" NO se promete
  en general: el `.exe` ES código Rust, así que su velocidad
  es la de Rust.

## Tamaño

- Eclipse en release: 1,652,336 bytes (~1.6 MB); con `strip`:
  1,363,400 bytes (~1.3 MB). El paquete standalone lleva
  dentro toda la VM, así que no puede pesar menos de 100 KB.
- El archivo `.ec` más grande es `bibliotecas/dibujo.ec`
  (2,127 bytes). Los 133 archivos `.ec` juntos pesan 30,651
  bytes: todos los programas están muy por debajo de 100 KB.

## Auditoría y pruebas

- Suite nueva `pruebas10/` (20 archivos): 14 casos del Borrow
  Checker (variable movida, desconocida, función desconocida,
  fija, constante, aridad, tipo, doble `&mut`, `&mut` en fija,
  `devolver`/`romper` fuera, movida en las dos ramas, movida
  en `repetir`, movida en una rama que el motor para), 3 de
  `usar` (una librería, dos librerías, librería que falta,
  `usar` abajo), 1 de sistema y 1 de aridad de builtin.
- Ejemplos nuevos `ejemplos/10-librerias/`: `usar_demo.ec`
  (matemáticas + dibujo) y `sistema_demo.ec`.
- **Total: 127 pruebas, 0 fallas** (intérprete, VM, paquete y
  `.exe`, con stdout, stderr y código de salida comparados).
  LSP: TODO OK. `cargo clippy --all-targets`: 0 advertencias.
  `cargo build` y `cargo build --release`: 0 advertencias.

## Qué sigue (1.2)

La 1.2 tomó otro camino (el que pidió Kevin): sintaxis más
completa y 100% propia — mapas, `intentar` / `si_falla` /
`al_final`, cambiar un lugar de una lista o un mapa, 13
funciones nuevas y el icono (ver la sección 1.2 abajo).
De esta lista, la 1.2 trae `fecha()` y `hora()` (en UTC, sin
formato todavía). Siguen pendientes: filtrar y buscar en las
listas, y el compilador a `.exe` sin `rustc`.

---

# Eclipse 1.1 — Todo con la máquina virtual, librerías con Registro y Borrow Checker pulido

## Sin intérprete: la VM es EL motor

Eclipse ya no tiene intérprete. Todo corre con la **máquina
virtual por slots**, que habla directo: tu programa `.ec` se
compila a bytecode y la VM lo corre. El modo interactivo
(`eclipse` sin argumentos) y el depurador (`--depurar`,
`--trazar`) también corren en la VM, con las mismas órdenes
de siempre (`Enter` = paso, `v` = variables, `p nombre` = ver
una variable, `c` = seguir, `s` = salir). El compilador a
`.exe` y el paquete standalone siguen igual (el `.exe` es
código Rust, el paquete es Eclipse + bytecode).

El depurador de la VM muestra las variables buscando en el
bytecode qué variable vive en cada slot, así que `v` y
`p nombre` funcionan igual que antes.

## El Registro de librerías

Las librerías ya no viven dentro de Eclipse: viven en el
**Registro**, un sistema aparte del mismo proyecto, en
`~/.eclipse/registro/` (las disponibles en
`registro/bibliotecas/`, las que están en revisión en
`registro/cola/`). Las librerías de la 1.0 se mueven solas
al Registro la primera vez que usas un comando de librerías.

**Subir una librería (por terminal):** `eclipse biblioteca
publicar archivo.ec` te hace **6 preguntas**: nombre de la
librería, tu nombre, descripción, versión, categoría y cómo
te pueden contactar. Después de subirla, **tarda de 1 a 2
días en llegar a Eclipse, sin excepciones** (1 día y medio
en la cola de revisión); mientras tanto `usar` te dice
cuánto falta. Y **solo se puede subir una librería cada 5
horas**.

**Seguridad (para que no se suban librerías a lo tonto):**

- Solo se publica código escrito **en Eclipse** (cualquier
  otro lenguaje no se entiende y no se publica).
- **Tiene que estar en español:** si encuentra una palabra
  en inglés, te dice en qué línea está y no la publica.
- **Tiene que tener al menos una función:** una librería
  sin funciones no sirve para nada.
- El nombre no puede ser una palabra de Eclipse (`si`,
  `para`, `usar`...) ni un nombre peligroso (`firmware`,
  `virus`, `malware`...).
- El escáner sigue revisando comandos del sistema y rutas
  peligrosas, y el Borrow Checker revisa la librería antes
  de aceptarla. Y cada vez que usas una librería, Eclipse
  revisa que no haya cambiado desde que se publicó.

Comandos nuevos: `biblioteca estado <nombre>` (te dice si
está disponible o en revisión y cuánto falta) y `biblioteca
revisar` (aprueba las que ya cumplieron su tiempo; también
se aprueban solas cuando las usas).

**Honestidad:** la revisión de 1 a 2 días la hace Eclipse
en tu propia computadora (una cola local con fecha de
llegada), no un servidor en internet: Eclipse no tiene
servidor. El tiempo se cumple de verdad (no hay forma de
saltárselo por la terminal), pero alguien con acceso a los
archivos del Registro podría tocarlos a mano, igual que
cualquier archivo de tu computadora.

## El Borrow Checker, pulido (3 revisiones nuevas)

1. **Dividir o calcular el resto entre un cero escrito a
   mano** (`10 / 0`, `10 % (0)`): antes el programa corría
   y se paraba en ejecución; ahora el compilador te regaña
   antes de correr.
2. **Pedir un lugar de una lista que no existe:** si la
   lista está escrita a mano (`[10, 20, 30][5]`) o es una
   variable con una lista escrita a mano, el compilador te
   dice que solo tiene 3 elementos (del lugar 0 al 2).
3. **Que una función devuelva tipos distintos:** si en una
   rama devuelve un número y en otra un texto (o un
   `devolver;` vacío y luego un `devolver 5;`), el
   compilador te dice en qué línea devolvió cada tipo.

## Velocidad (banco, el mejor de 3, en milisegundos)

  | programa | VM    | .exe   | paquete |
  |----------|-------|--------|---------|
  | fib.ec   | 13.14 | 10.03  | 12.56   |
  | bucle.ec | 328.09| 276.13 | 335.29  |
  | para.ec  | 193.35| 236.59 | 190.05  |
  | texto.ec | 20.87 | 22.60  | 20.99   |
  | lista.ec | 10.04 | 11.37  | 9.93    |

  La VM sigue siendo rápida (en `para` y en `lista` le
  gana al `.exe`), y el `.exe` sigue ganando en recursión
  y en `mientras`. "Igual que Rust o más rápido" NO se
  promete en general: el `.exe` ES código Rust, así que su
  velocidad es la de Rust.

## Tamaño

- Eclipse en release: 1,528,688 bytes (~1.5 MB); con `strip`:
  1,268,896 bytes (~1.2 MB). El paquete standalone lleva
  dentro toda la VM, así que no puede pesar menos de 100 KB.
  (La 1.1 pesa MENOS que la 1.0: quitar el intérprete
  achicó el programa en ~120 KB.)
- El archivo `.ec` más grande es `bibliotecas/dibujo.ec`
  (2,127 bytes). Los 143 archivos `.ec` juntos pesan 31,649
  bytes: todos los programas están muy por debajo de 100 KB.

## Auditoría y pruebas

- Suite nueva `pruebas11/` (10 archivos): las 3 revisiones
  nuevas del Borrow Checker (dividir entre cero, resto entre
  cero, índice fuera de la lista, `devolver` con tipos
  distintos) y 3 programas que prueban que todo corre en la
  VM (bucles con `seguir`/`romper`, recursión, `para` con
  `si` sin `sino`).
- Prueba nueva en el probador: el sistema de librerías
  completo con una casa temporal (publicar con las 6
  preguntas, que quede en revisión, que `usar` antes de
  tiempo dé error, el límite de 5 horas, aprobar y usar).
- **Total: 138 pruebas, 0 fallas** (VM, paquete y `.exe`,
  con stdout, stderr y código de salida comparados). LSP:
  TODO OK. `cargo clippy --all-targets`: 0 advertencias.
  `cargo build` y `cargo build --release`: 0 advertencias.
---

# Eclipse 1.2 — Sintaxis más completa y 100% propia, mapas, intentar y automatización

La sintaxis de Eclipse ahora es más completa: el código se lee
línea por línea como siempre, pero con las formas nuevas que
faltaban — un mapa para guardar datos con nombre, `intentar`
para atrapar errores, y funciones para automatizar. Del ejemplo
que llegó con el pedido (un programa en Python) solo se tomó
la **idea de cómo se ve un programa**: un diccionario, unas
funciones y atrapar errores. La sintaxis es 100% de Eclipse,
en español, y no es copia de Python. Tampoco trae el escáner
del ejemplo: Eclipse no se conecta a otras computadoras.

## Mapas, `intentar` y cambiar un lugar

- **Mapas** (`{21: "FTP", 22: "SSH"}`): claves número, texto
  o verdadero/falso; se leen con `mapa[clave]`, se cambian con
  `mapa[clave] = valor;` (si la clave no existe, la crea), y
  se recorren con `claves(mapa)` / `valores(mapa)` /
  `tiene(mapa, clave)`. Un mapa se mueve como un texto o una
  lista, y `copiar` deja una copia. Tipo nuevo: `mapa`.
- **Cambiar un lugar** (`lista[0] = 5;`, `mapa[21] = "FTP";`):
  la variable tiene que ser `sea mut`. Solo vale un lugar
  directo (nada de `m[0][1] = 5;`).
- **`intentar { } si_falla e { } al_final { }`**: si algo falla
  dentro, el programa no se cae: el `si_falla` recibe el error
  (la variable es opcional) y el `al_final` corre siempre —
  también si hubo `devolver`, `romper` o `seguir` dentro.
  Funciona en la VM y en el `.exe` (el `.exe` convierte cada
  `intentar` en dos funciones Rust: el cuerpo y una envoltura
  que cierra el ámbito pase lo que pase). Se puede anidar.
- **13 funciones nuevas** (48 en total): `claves`, `valores`,
  `tiene`, `fecha`, `hora`, `fecha_hora`, `dormir`, `existe`,
  `listar_carpeta`, `crear_carpeta`, `mover_archivo`,
  `borrar_archivo`, `reemplazar`. Con ellas se hacen proyectos
  completos y automatización (revisar archivos, moverlos,
  esperar, saber la fecha).
- **4 palabras nuevas** (34 en total): `mapa`, `intentar`,
  `si_falla`, `al_final`. Ningún símbolo nuevo.

## El icono

Eclipse ya tiene icono propio: `assets/icon.png` (256x256, un
eclipse total con corona dorada sobre azul noche). Aparece en
la extensión de VS Code (`vscode-eclipse/assets/icon.png`,
declarado como `"icon"` en su `package.json`) y en todo lo de
Eclipse.

## Bugs arreglados (encontrados en la auditoría de la 1.2)

1. **`romper` y `seguir` dentro de un `mientras` no
   funcionaban:** el Borrow Checker decía que estaban fuera
   de un bucle (se le olvidó contar el `mientras` como bucle).
   La VM y el compilador sí lo aceptaban; ahora el verificador
   también.
2. **`mapa["clave"]` fallaba en la VM:** la instrucción de
   índice pedía un número antes de mirar si era un mapa, así
   que una clave de texto siempre daba error. Ahora mira
   primero qué es (lista o mapa) y luego valida el índice.
3. **`elemento(mapa, clave)` no sabía leer mapas:** el `.exe`
   genera los `[ ]` con esta función, así que el `.exe`
   también fallaba con mapas. Ahora los lee igual que la VM.
4. **El escáner de librerías tenía huecos:** no miraba dentro
   de `constante` (ahí se podía esconder un `comando`), ni
   dentro de `intentar` ni de cambiar un lugar, ni revisaba
   las rutas de `mover_archivo`, `borrar_archivo`,
   `listar_carpeta`, `crear_carpeta` y `existe`. Ahora todo eso
   se revisa antes de publicar.

## Velocidad (banco, el mejor de 3, en milisegundos)

  | programa | VM     | .exe   | paquete |
  |----------|--------|--------|---------|
  | fib.ec   | 13.00  | 10.17  | 12.76   |
  | bucle.ec | 315.37 | 271.19 | 310.93  |
  | para.ec  | 176.18 | 229.65 | 178.78  |
  | texto.ec | 21.09  | 22.23  | 21.15   |
  | lista.ec | 10.70  | 11.39  | 10.34   |

  La VM sigue cerca del `.exe` (en `para` y en `lista` le
  gana), y el `.exe` sigue ganando en recursión y en
  `mientras`. "Igual que Rust o más rápido" NO se promete en
  general: el `.exe` ES código Rust, así que su velocidad es
  la de Rust.

## Tamaño

- Eclipse en release: 1,655,144 bytes (~1.6 MB); con `strip`:
  1,381,800 bytes (~1.3 MB).
- El archivo `.ec` más grande sigue siendo
  `bibliotecas/dibujo.ec` (2,127 bytes). Los 170 archivos
  `.ec` juntos pesan 36,465 bytes: todos los programas están
  muy por debajo de 100 KB.

## Auditoría y pruebas

- Suite nueva `pruebas12/` (24 archivos): mapas (literal,
  claves, valores, tipo `mapa`, mover, clave con variable),
  cambiar un lugar (en lista y en mapa, anidado, fija,
  constante, movida), `intentar` (con y sin variable del
  error, solo `al_final`, `devolver`, `romper`, `seguir`,
  anidado), `romper`/`seguir` dentro de `mientras`, las 13
  funciones nuevas, y 9 programas de error (clave de mapa
  mala, clave que falta, índice anidado, `intentar` vacío,
  fecha con datos, dormir negativo).
- Ejemplos nuevos `ejemplos/11-mapas-automatizacion/` (3
  archivos): mapas, intentar y automatización.
- **Total: 165 pruebas, 0 fallas** (VM, paquete y `.exe`,
  con stdout, stderr y código de salida comparados). LSP:
  TODO OK. `cargo clippy --all-targets`: 0 advertencias.
  `cargo build` y `cargo build --release`: 0 advertencias.

## Qué sigue (1.3)

Lo que Kevin pida: la 1.2 cerró lo que él pidió (sintaxis más
completa, mapas, intentar, automatización, icono). Quedan
pendientes de antes: filtrar y buscar en las listas, fecha y
hora con formato, y el compilador a `.exe` sin `rustc`.
