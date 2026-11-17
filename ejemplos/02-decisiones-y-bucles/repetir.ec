// repetir: el cuerpo corre una vez, y luego repite
// mientras la condición sea verdadera (como do-while)
// Para correrlo:  cargo run -- ejemplos/repetir.ec

sea n = 0;
repetir {
    n = n + 1;
    imprimir(n);
} mientras n < 3;

// Aunque la condición sea falsa desde el inicio,
// el cuerpo corre UNA vez:
sea m = 10;
repetir {
    imprimir("esto se imprime una vez, m =", m);
} mientras m < 5;
