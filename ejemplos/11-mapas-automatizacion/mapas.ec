// Un mapa es como un diccionario: cada clave tiene su valor.
// Aquí guardamos servicios con su puerto y su descripción.
// (Es solo una lista de datos, no escanea nada.)
sea mut servicios = {21: "FTP", 22: "SSH", 80: "HTTP", 443: "HTTPS"};
imprimir("El puerto 22 es:", servicios[22]);
imprimir("El puerto 80 es:", servicios[80]);
imprimir("¿Hay un 3306?", tiene(servicios, 3306));
servicios[3306] = "MySQL";
imprimir("Ahora sí hay 3306:", tiene(servicios, 3306));
imprimir("Puertos:", claves(servicios));
imprimir("Servicios:", valores(servicios));
imprimir("Cuántos hay:", largo(servicios));
imprimir(servicios);
