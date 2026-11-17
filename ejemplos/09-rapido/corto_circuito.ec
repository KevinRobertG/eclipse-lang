// El atajo de `y` y `o`: si el lado izquierdo ya decide,
// el derecho ni se mira (ni siquiera se llama a la función).
sea mut llamadas = 0;
funcion lado() {
    llamadas = llamadas + 1;
    devolver verdadero;
}
imprimir(falso y lado());
imprimir("llamadas:");
imprimir(llamadas);
imprimir(verdadero o lado());
imprimir("llamadas:");
imprimir(llamadas);
imprimir(verdadero y lado());
imprimir("llamadas:");
imprimir(llamadas);
