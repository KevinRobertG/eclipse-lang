// Un minijuego que también usa las utilidades.
importar "utilidades.ec";

sea mut total = 0;
para ronda en 1..=3 {
    total = total + doble(ronda);
    imprimir("Ronda " + a_texto(ronda) + ": total " + a_texto(total));
}
imprimir("Total final: " + a_texto(total));
