sea x = 99;
segun x {
    1 => { imprimir("uno"); }
    2 => { imprimir("dos"); }
}
imprimir("sin coincidencia, no pasa nada");
segun "hola" {
    "hola" => { imprimir("saludo"); }
    "adios" => { imprimir("despedida"); }
}
segun nulo {
    nulo => { imprimir("es nulo"); }
}
