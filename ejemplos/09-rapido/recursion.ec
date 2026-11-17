// La recursión profunda ya para igual en los tres motores:
// el intérprete corría en una pila chiquita y se caía con
// 250 llamadas; ahora los tres paran en 1000.
funcion baja(n) {
    si n == 0 {
        devolver 0;
    }
    devolver baja(n - 1) + 1;
}
imprimir(baja(500));
