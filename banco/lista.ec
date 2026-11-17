sea mut numeros: lista = [];
sea mut i = 1;
mientras i <= 10000 {
    numeros = agregar(numeros, i);
    i = i + 1;
}
sea mut total = 0;
para_cada n en numeros {
    total = total + n;
}
imprimir(total);
