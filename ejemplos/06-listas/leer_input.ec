// Eclipse 0.6 - `leer` como Input: lee lo que escribe
// el usuario y lo guarda en la variable. La variable debe
// ser `sea mut`. Si ya era un número, convierte el texto
// a número; si era texto, guarda el texto tal cual.
sea mut nombre = "";
imprimir("¿Cómo te llamas?");
leer nombre;
imprimir("Hola, " + nombre);
sea mut edad: numero = 0;
imprimir("¿Cuántos años tienes?");
leer edad;
imprimir("El año que viene tendrás");
imprimir(edad + 1);
