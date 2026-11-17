// Interpolación de textos en Eclipse
// Para correrlo:  cargo run -- ejemplos/interpolacion.ec
// O en la máquina virtual:  cargo run -- --vm ejemplos/interpolacion.ec

// Dentro de un texto, {nombre} pega el valor de la variable.
// También se pueden poner cuentas: {puntos * 2}.
// Para escribir una llave de verdad, escribe {{ o }}.

sea nombre = "Kevin";
sea puntos = 10;

imprimir("Hola, {nombre}, tienes {puntos} puntos");
imprimir("El doble de tus puntos es {puntos * 2}");
imprimir("El año que viene tendrás {puntos + 1} puntos");

sea edad = 13;
imprimir("{nombre} tiene {edad} años");

// Antes, para hacer esto había que pegar con +:
// imprimir("Hola, " + nombre + ", tienes " + a_texto(puntos) + " puntos");
// Ahora es mucho más fácil de leer.

// Las llaves literales:
imprimir("Esto es una llave: {{ y esto es otra: }}");
