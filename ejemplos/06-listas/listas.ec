// Eclipse 0.6 - Listas: varios valores en orden.
// Una lista es como el `Vec` de Rust: crece con agregar
// y se recorre con para_cada.
sea mut numeros: lista = [1, 2, 3];
imprimir(numeros);
imprimir(numeros[0]);
imprimir(largo(numeros));
numeros = agregar(numeros, 4);
imprimir(numeros);
imprimir(elemento(numeros, 2));
sea mas: lista = [5, 6];
imprimir(numeros + mas);
para_cada n en numeros {
    imprimir(n * 2);
}
