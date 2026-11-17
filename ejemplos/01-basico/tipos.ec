// Variables con tipo en Eclipse
// Para correrlo:  cargo run -- ejemplos/tipos.ec

sea nombre: texto = "Kevin";
sea edad: numero = 13;
sea juega: logico = verdadero;

imprimir(nombre, "tiene", edad, "años. ¿Juega?", juega);

// Una variable fija no se puede cambiar.
// Si quitas los // de la siguiente línea, Eclipse te avisa:
// nombre = "Otro";
