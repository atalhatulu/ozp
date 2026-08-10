#include <stdio.h>
#include <stdlib.h>
#include "utils.h"

char* dosya_oku(const char* yol) {
    FILE* dosya = fopen(yol, "rb");
    if (!dosya) {
        printf("Hata: '%s' dosyasi acilamadi!\n", yol);
        return NULL;
    }
    fseek(dosya, 0, SEEK_END);
    long boyut = ftell(dosya);
    fseek(dosya, 0, SEEK_SET);

    char* tampon = malloc(boyut + 1);
    if (tampon) {
        fread(tampon, 1, boyut, dosya);
        tampon[boyut] = '\0';
    }
    fclose(dosya);
    return tampon;
}
