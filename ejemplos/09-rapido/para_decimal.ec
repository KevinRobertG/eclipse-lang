// Los decimales ya funcionan en el `para`: antes el
// intérprete cortaba 1.5 a 1 y un paso de 0.5 no corría.
para i en 1.5..=3.5 {
    imprimir(i);
}
imprimir("---");
para j en 0..=2 paso 0.5 {
    imprimir(j);
}
