funcion fib(n) {
    si n < 2 {
        devolver n;
    }
    devolver fib(n - 1) + fib(n - 2);
}
imprimir(fib(15));
imprimir(fib(10));
