// Armar una lista grande con `agregar` ya no es lento:
// antes, cada vuelta clonaba la lista entera (de 10 en 10
// se hacía eterno); ahora se mete en su sitio.
sea mut numeros = [];
para i en 1..=1000 {
    numeros = agregar(numeros, i);
}
imprimir("largo:");
imprimir(largo(numeros));
imprimir("primero:");
imprimir(elemento(numeros, 0));
imprimir("último:");
imprimir(elemento(numeros, 999));
sea mut total = 0;
para_cada n en numeros {
    total = total + n;
}
imprimir("total:");
imprimir(total);
