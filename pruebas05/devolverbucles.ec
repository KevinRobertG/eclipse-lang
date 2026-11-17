funcion buscar(n) {
    para i en 1..=10 {
        si i == n {
            devolver i * 100;
        }
    }
    devolver 0;
}
funcion doble(n) {
    devolver n * 2;
}
imprimir(doble(buscar(4)));
