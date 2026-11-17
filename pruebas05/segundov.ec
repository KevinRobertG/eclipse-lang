funcion clasificar(n) {
    segun n {
        1 => { devolver "uno"; }
        2 => { devolver "dos"; }
        _ => { devolver "otro"; }
    }
}
imprimir(clasificar(1));
imprimir(clasificar(2));
imprimir(clasificar(9));
