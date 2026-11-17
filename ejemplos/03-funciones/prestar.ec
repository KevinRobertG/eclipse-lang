// Prestar con & en Eclipse (ownership, como Rust)
// Para correrlo:  cargo run -- ejemplos/prestar.ec
// O en la máquina virtual:  cargo run -- --vm ejemplos/prestar.ec

// Cuando le pasas un texto a una función, la función se lo
// queda (se MUEVE) y tu variable ya no se puede usar.
// Para prestárselo sin que se lo quede, escribe & delante:
// saludar(&nombre) le presta el texto, no lo mueve.

funcion saludar(n) {
    imprimir("Hola, " + n + "!");
}

funcion gritar(t) {
    imprimir(t + "!!!");
}

sea nombre = "Kevin";

// Sin &: nombre se mueve a saludar y ya no se puede usar.
// saludar(nombre);
// imprimir(nombre);  <- Esto daría error: nombre se movió.

// Con &: nombre sigue vivo después de la llamada.
saludar(&nombre);
gritar(&nombre);
imprimir("Mi nombre sigue vivo: {nombre}");

// Si quieres que la función se quede con el texto,
// pásalo sin & (eso es mover, como siempre).
sea mensaje = "Hola";
saludar(mensaje);
// imprimir(mensaje);  <- Esto daría error: mensaje se movió
// a la función 'saludar'.

// Y si quieres quedarte con una copia, usa copiar():
sea original = "Kevin";
saludar(copiar(original));
imprimir("El original sigue vivo: {original}");
