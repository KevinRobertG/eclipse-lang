sea mut total = 0;
para a en 1..=3 {
    para b en 1..=3 {
        si b == 2 {
            seguir;
        }
        total = total + a * b;
    }
}
imprimir(total);
