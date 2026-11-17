// segun: como el `match` de Rust
// Para correrlo:  cargo run -- ejemplos/segun.ec
// O en la máquina virtual:  cargo run -- --vm ejemplos/segun.ec

sea puntos = 10;

segun puntos {
    10 => { imprimir("¡diez puntos, premio!"); }
    5 => { imprimir("cinco puntos"); }
    _ => { imprimir("otro puntaje"); }
}

sea nombre = "Ana";

segun nombre {
    "Kevin" => { imprimir("Hola, Kevin"); }
    "Ana" => { imprimir("Hola, Ana"); }
    _ => { imprimir("Hola, ¿quién eres?"); }
}

// segun también sirve para devolver valores desde funciones.
funcion clasificar(n) {
    segun n {
        0 => { devolver "cero"; }
        1 => { devolver "uno"; }
        _ => { devolver "muchos"; }
    }
}

imprimir(clasificar(0));
imprimir(clasificar(1));
imprimir(clasificar(7));
