// Mi primer programa en Eclipse
// Para correrlo:  cargo run -- ejemplos/hola.ec

sea nombre = "Kevin";
sea mut puntos = 10;

imprimir("Hola,", nombre);
imprimir("Puntos:", puntos);

puntos = puntos + 5;
imprimir("Ahora tienes", puntos, "puntos");

// Los textos también se pueden pegar con +
imprimir("Hola, " + nombre + ", bienvenido a Eclipse");
