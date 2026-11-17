sea mut n = 0;
mientras n < 5 {
    n = n + 1;
    intentar {
        si n == 3 {
            romper;
        }
        imprimir(n);
    } al_final {
        imprimir("vuelta");
    }
}
imprimir("salí");
