// para con paso en Eclipse
// Para correrlo:  cargo run -- ejemplos/paso.ec
// O en la máquina virtual:  cargo run -- --vm ejemplos/paso.ec

// para i en 1..10 paso 2  ->  1, 3, 5, 7, 9
// El paso es de cuánto en cuánto avanza el contador.
// Si no lo escribes, el paso es 1.

// De 2 en 2:
para i en 1..10 paso 2 {
    imprimir(i);
}

imprimir("---");

// Hacia atrás, de 2 en 2:
para i en 10..1 paso -2 {
    imprimir(i);
}

imprimir("---");

// De 3 en 3, sin incluir el 10:
para i en 1..10 paso 3 {
    imprimir(i);
}

imprimir("---");

// El paso puede ser una variable:
sea salto = 5;
para i en 0..20 paso salto {
    imprimir(i);
}

// Si el paso es 0, el bucle no corre (no se puede avanzar).
para i en 1..5 paso 0 {
    imprimir(i);
}
imprimir("fin");
