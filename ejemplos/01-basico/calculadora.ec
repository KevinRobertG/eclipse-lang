// La calculadora de Eclipse
// Para correrla:  cargo run -- ejemplos/calculadora.ec

funcion sumar(a, b) {
    devolver a + b;
}

funcion multiplicar(a, b) {
    devolver a * b;
}

imprimir("=== Calculadora de Eclipse ===");
imprimir("Escribe el primer número:");
sea texto1 = leer();
sea n1 = a_numero(texto1);

imprimir("Escribe el segundo número:");
sea texto2 = leer();
sea n2 = a_numero(texto2);

imprimir("La suma es:", sumar(n1, n2));
imprimir("La multiplicación es:", multiplicar(n1, n2));
imprimir("La resta es:", n1 - n2);
imprimir("La división es:", n1 / n2);
