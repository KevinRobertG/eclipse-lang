// intentar: si algo falla dentro, no se cae el programa:
// el si_falla te dice qué pasó y el al_final siempre corre.
sea mut cero = 0;
intentar {
    imprimir(10 / cero);
} si_falla e {
    imprimir("No se pudo dividir:");
    imprimir(e);
} al_final {
    imprimir("Esto siempre se imprime.");
}
intentar {
    imprimir("Todo salió bien.");
} al_final {
    imprimir("Y el final también.");
}
