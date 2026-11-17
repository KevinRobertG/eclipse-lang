// Eclipse 0.6 - Prestar una lista para cambiar (&mut).
// La función recibe la lista, le agrega un valor y cuando
// termina, la lista nueva vuelve a tu variable.
funcion meter(lista) {
    lista = agregar(lista, 99);
}
sea mut numeros = [1, 2];
meter(&mut numeros);
imprimir(numeros);
// Prestar solo para mirar (&): la función la lee pero no
// se la puede quedar (no se puede mover).
funcion ver(lista) {
    imprimir(largo(lista));
}
ver(&numeros);
imprimir(numeros);
