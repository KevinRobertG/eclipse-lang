intentar {
    intentar {
        sea mut cero = 0;
        imprimir(1 / cero);
    } si_falla e {
        imprimir("adentro atrapó:");
        imprimir(e);
    }
    imprimir("sigo adentro");
} si_falla e {
    imprimir("afuera atrapó:");
    imprimir(e);
} al_final {
    imprimir("final afuera");
}
imprimir("listo");
