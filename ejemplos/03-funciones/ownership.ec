// Ownership en Eclipse (como Rust)
// Para correrlo:  cargo run -- ejemplos/ownership.ec
// O en la máquina virtual:  cargo run -- --vm ejemplos/ownership.ec

// Regla 1: cada valor tiene UN solo dueño.
// Regla 2: los números y los lógicos son fotocopias:
//          se copian y la variable vieja sigue viva.
// Regla 3: los textos son originales: si se lo das a otra
//          variable o a una función, se MUEVE y la variable
//          vieja ya no se puede usar.

// Los números se copian: a sigue viva después de darle a b.
sea a = 10;
sea b = a;
imprimir("a sigue viva:", a);
imprimir("b también:", b);

// Los textos se mueven: nombre se va a saludo.
sea nombre = "Kevin";
sea saludo = nombre;
imprimir("saludo tiene:", saludo);
// imprimir(nombre);  <- Esto daría error: nombre ya se movió.

// copiar() hace una copia nueva: el original sigue vivo.
sea original = "Kevin";
sea copia = copiar(original);
imprimir("original:", original);
imprimir("copia:", copia);

// Las funciones que trae Eclipse solo MIRAN el texto.
sea frase = "Hola";
imprimir(frase);
imprimir(frase);

// Tus funciones SÍ se quedan con el texto que les pasas.
funcion gritar(t) {
    imprimir(t + "!!!");
}

sea mensaje = "Hola";
gritar(copiar(mensaje));
imprimir("mensaje sigue vivo:", mensaje);
