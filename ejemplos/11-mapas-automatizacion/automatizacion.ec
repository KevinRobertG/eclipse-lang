// Automatización: revisar archivos y carpetas, y la fecha.
// (La fecha y la hora son en UTC: Rust solo no sabe tu zona horaria,
// así que aquí solo revisamos que tengan el largo correcto.)
imprimir("¿Existe hola.ec?", existe("ejemplos/01-basico/hola.ec"));
imprimir("¿Existe un archivo que no existe?", existe("no_existe.txt"));
imprimir("Archivos de 01-basico:", listar_carpeta("ejemplos/01-basico"));
imprimir("La fecha tiene 10 letras:", largo(fecha()) == 10);
imprimir("La hora tiene 8 letras:", largo(hora()) == 8);
imprimir(reemplazar("hola mundo", "mundo", "Kevin"));
dormir(0);
imprimir("Automatización lista.");
