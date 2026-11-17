// Las herramientas: lo que ayuda a escribir programas .ec.
// - `autocompletar`: qué puedes escribir en una posición (lo usa el LSP y --completar).
// - `progreso`: la barra de progreso (como la de Cargo).
// - `proyecto`: los proyectos (Eclipse.toml, nuevo, construir, correr-todos).
// - `instalar`: instala Eclipse y su extensión de VS Code.
// - `bibliotecas`: las librerías de Eclipse (usar, publicar...).
pub mod autocompletar;
pub mod bibliotecas;
pub mod instalar;
pub mod progreso;
pub mod proyecto;
