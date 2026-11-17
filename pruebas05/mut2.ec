funcion sumar_a(a, b) {
    a = a + 1;
    b = b + 2;
    devolver a + b;
}
sea x = 1;
sea z = 2;
imprimir(sumar_a(&mut x, &mut z));
imprimir(x);
imprimir(z);
