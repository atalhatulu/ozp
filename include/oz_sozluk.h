#ifndef OZ_SOZLUK_H
#define OZ_SOZLUK_H

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

typedef struct {
    char* anahtar;
    char* deger;
} OzAnahtarDeger;

typedef struct {
    OzAnahtarDeger* ogeler;
    int sayi;
    int kapasite;
} OzSozluk;

OzSozluk* sozluk_olustur() {
    OzSozluk* s = (OzSozluk*)malloc(sizeof(OzSozluk));
    s->sayi = 0;
    s->kapasite = 16;
    s->ogeler = (OzAnahtarDeger*)malloc(sizeof(OzAnahtarDeger) * s->kapasite);
    return s;
}

void sozluk_ekle(OzSozluk* s, char* anahtar, char* deger) {
    for (int i = 0; i < s->sayi; i++) {
        if (strcmp(s->ogeler[i].anahtar, anahtar) == 0) {
            s->ogeler[i].deger = strdup(deger);
            return;
        }
    }
    if (s->sayi >= s->kapasite) {
        s->kapasite *= 2;
        s->ogeler = (OzAnahtarDeger*)realloc(s->ogeler, sizeof(OzAnahtarDeger) * s->kapasite);
    }
    s->ogeler[s->sayi].anahtar = strdup(anahtar);
    s->ogeler[s->sayi].deger = strdup(deger);
    s->sayi++;
}

char* sozluk_getir(OzSozluk* s, char* anahtar) {
    for (int i = 0; i < s->sayi; i++) {
        if (strcmp(s->ogeler[i].anahtar, anahtar) == 0) {
            return s->ogeler[i].deger;
        }
    }
    return "BULUNAMADI";
}

void sozluk_sil(OzSozluk* s) {
    for (int i = 0; i < s->sayi; i++) {
        free(s->ogeler[i].anahtar);
        free(s->ogeler[i].deger);
    }
    free(s->ogeler);
    free(s);
}

#endif // OZ_SOZLUK_H
