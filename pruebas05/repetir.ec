sea n = 0;
repetir {
    n = n + 1;
    imprimir(n);
} mientras n < 3;
sea m = 10;
repetir {
    imprimir(m);
} mientras m < 5;
sea i = 0;
repetir {
    i = i + 1;
    si i == 2 {
        seguir;
    }
    si i == 4 {
        romper;
    }
    imprimir(i);
} mientras i < 10;
