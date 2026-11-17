// Bucles y decisiones en Eclipse
// Para correrlo:  cargo run -- ejemplos/bucles.ec

// Contar del 1 al 10 con mientras
sea mut n = 1;
mientras n <= 10 {
    imprimir(n);
    n = n + 1;
}

imprimir("---");

// Parar antes con romper
sea mut m = 1;
mientras m <= 10 {
    si m == 5 {
        imprimir("Me paro en el 5");
        romper;
    }
    imprimir(m);
    m = m + 1;
}

imprimir("---");

// Saltar un número con seguir
para i en 1..=5 {
    si i == 3 {
        seguir;
    }
    imprimir("Número:", i);
}
