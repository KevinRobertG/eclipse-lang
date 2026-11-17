// Eclipse habla con archivos
// Para correrlo:  cargo run -- ejemplos/archivos.ec

escribir_archivo("saludo.txt", "Hola desde Eclipse");
sea contenido = leer_archivo("saludo.txt");
imprimir("El archivo dice:", contenido);
