sea mut total = 0;
para i en 1..=3 {
    para j en 1..=3 {
        si j == 2 {
            seguir;
        }
        si i == 2 y j == 3 {
            romper;
        }
        total = total + 1;
    }
}
imprimir(total);
sea mut n = 0;
repetir {
    n = n + 1;
    si n >= 3 {
        romper;
    }
} mientras falso;
imprimir(n);
