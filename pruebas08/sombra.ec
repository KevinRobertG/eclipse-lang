sea x = 1;
{
    sea x = 2;
    imprimir(x);
    {
        sea x = 3;
        imprimir(x);
    }
    imprimir(x);
}
imprimir(x);
sea mut z = 10;
si verdadero {
    sea z = 99;
    imprimir(z);
}
imprimir(z);
