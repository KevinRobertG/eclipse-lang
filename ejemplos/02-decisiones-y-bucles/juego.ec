// El mini juego en Eclipse (como la lección 12 de Rust)
// Para correrlo:  cargo run -- ejemplos/juego.ec

sea mut total = 0;

para ronda en 1..=3 {
    sea puntos = ronda * 2;
    total = total + puntos;
    imprimir("Ronda", ronda, "- ganaste", puntos, "puntos. Total:", total);
}

si total >= 10 {
    imprimir("¡Ganaste un premio!");
} sino {
    imprimir("Sigue intentando.");
}
