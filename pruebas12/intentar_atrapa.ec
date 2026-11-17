sea mut cero = 0;
intentar {
    imprimir(10 / cero);
} si_falla e {
    imprimir("No se pudo dividir:");
    imprimir(e);
} al_final {
    imprimir("Esto siempre se imprime.");
}
imprimir("seguimos");
