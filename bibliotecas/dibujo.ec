// ============================================================
//  Librería dibujo (Eclipse 1.0)
//  Dibuja en la ventana de la terminal: limpia la pantalla,
//  cambia el color del texto y dibuja cajas y títulos.
//  Para usarla:
//      usar dibujo;
//
//  Con honestidad: esto dibuja en la terminal (la ventana
//  negra donde corre tu programa), con letras y colores.
//  Las ventanas de verdad (con botones y ratón) necesitan
//  una librería gráfica, eso viene después.
// ============================================================
usar calculadora;

// Limpia toda la pantalla de la terminal.
funcion limpiar_pantalla() {
    imprimir(caracter(27) + "[2J" + caracter(27) + "[H");
}

// Cambia el color del texto: "rojo", "verde", "amarillo",
// "azul" o "normal" (para volver al color de siempre).
funcion color_texto(color) {
    si color == "rojo" {
        imprimir(caracter(27) + "[31m");
    } sino si color == "verde" {
        imprimir(caracter(27) + "[32m");
    } sino si color == "amarillo" {
        imprimir(caracter(27) + "[33m");
    } sino si color == "azul" {
        imprimir(caracter(27) + "[34m");
    } sino {
        imprimir(caracter(27) + "[0m");
    }
}

// Una línea de guiones: linea(10) -> ----------
funcion linea(n) {
    sea mut s = "";
    sea mut i = 0;
    mientras i < n {
        s = s + "-";
        i = i + 1;
    }
    imprimir(s);
}

// Una caja vacía: caja(10, 3) dibuja un rectángulo.
funcion caja(ancho, alto) {
    sea mut arriba = "+";
    sea mut i = 0;
    mientras i < ancho {
        arriba = arriba + "-";
        i = i + 1;
    }
    arriba = arriba + "+";
    imprimir(arriba);
    sea mut j = 0;
    mientras j < alto {
        sea mut medio = "|";
        sea mut k = 0;
        mientras k < ancho {
            medio = medio + " ";
            k = k + 1;
        }
        medio = medio + "|";
        imprimir(medio);
        j = j + 1;
    }
    imprimir(arriba);
}

// Un título con marco: titulo("Hola") dibuja el texto
// entre dos líneas.
funcion titulo(mensaje) {
    sea n = largo(mensaje) + 4;
    linea(n);
    imprimir("| " + mensaje + " |");
    linea(n);
}
