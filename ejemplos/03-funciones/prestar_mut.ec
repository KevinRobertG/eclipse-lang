// prestar_mut: prestar una variable para cambiarla (&mut)
// Para correrlo:  cargo run -- ejemplos/prestar_mut.ec

funcion subir(n) {
    n = n + 5;
    devolver n;
}

sea puntos = 10;
imprimir("Antes:", puntos);
imprimir("La función devuelve:", subir(&mut puntos));
imprimir("Después:", puntos);

// Solo se pueden prestar para cambiar números y lógicos.
// El texto no: si lo intentas, Eclipse te dice que no.
// Y una variable fija tampoco se puede prestar para cambiar.
