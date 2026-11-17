// El servidor de lenguaje (LSP), en Rust puro.
// - `json`: un JSON chiquito, hecho a mano.
// - `servidor`: habla JSON-RPC con el editor (autocompletado, hover, diagnósticos).
pub mod json;
pub mod servidor;
