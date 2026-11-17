// potencia: el operador ** (elevar a una potencia)
// Para correrlo:  cargo run -- ejemplos/potencia.ec

imprimir("2 ** 3 =", 2 ** 3);
imprimir("2 ** 10 =", 2 ** 10);
imprimir("3 ** 2 =", 3 ** 2);
// ** se lee de derecha a izquierda: 2 ** 3 ** 2 es 2 ** 9
imprimir("2 ** 3 ** 2 =", 2 ** 3 ** 2);
// ** manda más que *: 2 * 3 ** 2 es 2 * 9
imprimir("2 * 3 ** 2 =", 2 * 3 ** 2);
imprimir("El área de un cuadrado de lado 5 es:", 5 ** 2);
