// Fibonacci con recursión en Eclipse
// Para correrlo con la máquina virtual:
//   cargo run -- --vm ejemplos/fibonacci.ec
// Para ver el bytecode:
//   cargo run -- --bytecode ejemplos/fibonacci.ec

funcion fib(n) {
    si n <= 1 {
        devolver n;
    }
    devolver fib(n - 1) + fib(n - 2);
}

imprimir("fib(10) =", fib(10));
imprimir("fib(15) =", fib(15));
