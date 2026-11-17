// Los motores: los caminos para correr un programa .ec.
// - `compilador` + `bytecode` + `vm`: traducen a bytecode y lo corre la máquina virtual.
//   La máquina virtual es EL motor de Eclipse: todo corre con ella.
// - `transpilador`: traduce a Rust y `rustc` lo convierte en un .exe nativo.
// - `paquete`: guarda el bytecode en un archivo solo (ECBIN) y lo
//   pega al final de Eclipse para hacer un ejecutable standalone.
// - `valor`: los valores (número, texto, lista...) y las operaciones entre ellos.
// - `verificador`: el Borrow Checker (regaña antes de correr).
pub mod bytecode;
pub mod compilador;
pub mod paquete;
pub mod sistema;
pub mod transpilador;
pub mod valor;
pub mod verificador;
pub mod vm;
