// adivina: el juego de adivinar el número secreto
// Para correrlo:  cargo run -- ejemplos/adivina.ec

imprimir("=== Adivina el número ===");
imprimir("Pienso un número del 0 al 9. ¿Cuál es?");
sea secreto = aleatorio();
sea mut intentos = 0;
sea mut adivinado = falso;

mientras no adivinado {
    imprimir("Escribe tu número:");
    sea intento = leer_numero();
    intentos = intentos + 1;
    si intento == secreto {
        adivinado = verdadero;
        imprimir("¡Lo adivinaste! Era el {secreto}");
        imprimir("Lo lograste en {intentos} intentos");
    } sino si intento < secreto {
        imprimir("Muy bajo, prueba más alto");
    } sino {
        imprimir("Muy alto, prueba más bajo");
    }
}
