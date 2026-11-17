// ============================================================
//  Librería juegos (Eclipse 1.0)
//  Dados, monedas y azar para tus juegos. Para usarla:
//      usar juegos;
// ============================================================

// dado() -> un número del 1 al 6
funcion dado() {
    devolver aleatorio() % 6 + 1;
}

// moneda() -> "cara" o "cruz"
funcion moneda() {
    si aleatorio() % 2 == 0 {
        devolver "cara";
    }
    devolver "cruz";
}

// elegir(["piedra", "papel", "tijera"]) -> uno al azar
funcion elegir(lista) {
    sea lugar = aleatorio() % largo(lista);
    devolver elemento(lista, lugar);
}

// tirar_dados(2) -> la suma de tirar 2 dados
funcion tirar_dados(cuantos) {
    sea mut total = 0;
    sea mut i = 0;
    mientras i < cuantos {
        total = total + dado();
        i = i + 1;
    }
    devolver total;
}
