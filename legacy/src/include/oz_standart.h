#include <stdio.h>
#include <time.h>
#include <setjmp.h>
#include "oz_sozluk.h"

clock_t _oz_baslangic_zamani;
jmp_buf _oz_hata_buf;
char* _oz_son_hata_mesaji = "Bilinmeyen hata";

void _oz_zaman_baslat() {
    _oz_baslangic_zamani = clock();
}

void _oz_zaman_bitir() {
    double sure = (double)(clock() - _oz_baslangic_zamani) / CLOCKS_PER_SEC;
    printf("\n*** ISLEM SURESI: %f saniye ***\n", sure);
}

void oz_yazdir_tam(int n) {
    printf("OZ+ Ciktisi: %d\n", n);
}

void dosya_yaz(char* dosya_adi, char* icerik) {
    FILE* f = fopen(dosya_adi, "w");
    if(f) {
        fprintf(f, "%s", icerik);
        fclose(f);
        printf("[OZ+] %s dosyasina yazildi.\n", dosya_adi);
    } else {
        printf("[OZ+ HATASI] Dosya olusturulamadi: %s\n", dosya_adi);
    }
}

void dosya_oku(char* dosya_adi) {
    FILE* f = fopen(dosya_adi, "r");
    if(f) {
        char buffer[1024];
        printf("\n--- %s ICERIGI ---\n", dosya_adi);
        while(fgets(buffer, sizeof(buffer), f)) {
            printf("%s", buffer);
        }
        printf("\n------------------\n");
        fclose(f);
    } else {
        printf("[OZ+ HATASI] Dosya okunamadi: %s\n", dosya_adi);
    }
}
