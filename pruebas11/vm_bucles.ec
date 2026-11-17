sea total = 0;
para i en 1..=5 {
    si i == 3 {
        seguir;
    }
    si i == 5 {
        romper;
    }
    total = total + i;
}
imprimir(total);
sea n = 0;
mientras n < 10 {
    n = n + 1;
    si n == 4 {
        seguir;
    }
    si n == 7 {
        romper;
    }
}
imprimir(n);
