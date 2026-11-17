sea nombres = ["Kevin", "Fatima", "Dylan", "Ana"];
imprimir(unir(nombres, ", "));
imprimir(ordenar(nombres));
imprimir(largo(nombres));
sea mut total = 0;
para_cada nombre en nombres {
    total = total + largo(nombre);
}
imprimir(total);
