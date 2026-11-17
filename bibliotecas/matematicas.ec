// ============================================================
//  Librería matematicas (Eclipse 1.0)
//  Más matemáticas para tus programas. Para usarla:
//      usar matematicas;
// ============================================================

// factorial(5) -> 120
funcion factorial(n) {
    si n <= 1 {
        devolver 1;
    }
    devolver n * factorial(n - 1);
}

// es_par(4) -> verdadero ; es_par(7) -> falso
funcion es_par(n) {
    devolver n % 2 == 0;
}

// es_primo(7) -> verdadero ; es_primo(8) -> falso
funcion es_primo(n) {
    si n < 2 {
        devolver falso;
    }
    para i en 2..n {
        si n % i == 0 {
            devolver falso;
        }
    }
    devolver verdadero;
}

// mcd(12, 18) -> 6 (el máximo común divisor)
funcion mcd(a, b) {
    sea mut x = a;
    sea mut z = b;
    mientras z != 0 {
        sea resto = x % z;
        x = z;
        z = resto;
    }
    devolver x;
}

// mcm(4, 6) -> 12 (el mínimo común múltiplo)
funcion mcm(a, b) {
    devolver a * b / mcd(a, b);
}

// fibonacci(10) -> 55
funcion fibonacci(n) {
    si n <= 1 {
        devolver n;
    }
    devolver fibonacci(n - 1) + fibonacci(n - 2);
}

// promedio([10, 20, 30]) -> 20
funcion promedio(lista) {
    sea mut total = 0;
    para_cada valor en lista {
        total = total + valor;
    }
    devolver total / largo(lista);
}
