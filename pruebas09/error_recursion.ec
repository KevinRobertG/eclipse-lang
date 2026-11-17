funcion baja(n) {
    si n == 0 {
        devolver 0;
    }
    devolver baja(n - 1) + 1;
}
imprimir(baja(1001));
