#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <ctype.h>
#include "lexer.h"

static void ekle_token(TokenArray* dizi, TokenType tip, const char* deger, int satir) {
    if (dizi->count >= dizi->capacity) {
        dizi->capacity = (dizi->capacity == 0) ? 128 : dizi->capacity * 2;
        dizi->tokens = realloc(dizi->tokens, sizeof(Token) * dizi->capacity);
    }
    Token t;
    t.tip = tip;
    t.deger = deger ? strdup(deger) : NULL;
    t.satir = satir;
    dizi->tokens[dizi->count++] = t;
}

TokenArray tokenize(const char* kaynak_kod) {
    TokenArray dizi = {NULL, 0, 0};
    int i = 0;
    int satir = 1;

    while (kaynak_kod && kaynak_kod[i] != '\0') {
        char c = kaynak_kod[i];

        if (c == '\n') { satir++; i++; continue; }
        if (isspace(c)) { i++; continue; }

        // Yorum satırlarını yoksay
        if (c == '/' && kaynak_kod[i+1] == '/') {
            while (kaynak_kod[i] != '\0' && kaynak_kod[i] != '\n') {
                i++;
            }
            continue;
        }

        // Metin (String) tanıma
        if (c == '"') {
            i++;
            char tampon[512];
            int j = 0;
            while (kaynak_kod[i] != '\0' && kaynak_kod[i] != '"') {
                tampon[j++] = kaynak_kod[i++];
            }
            tampon[j] = '\0';
            if (kaynak_kod[i] == '"') i++; // Tırnağı kapat
            ekle_token(&dizi, TOKEN_METIN, tampon, satir);
            continue;
        }

        if (isalpha(c) || c == '_') {
            char tampon[256];
            int j = 0;
            while (isalnum(kaynak_kod[i]) || kaynak_kod[i] == '_') {
                tampon[j++] = kaynak_kod[i++];
            }
            tampon[j] = '\0';

            if (strcmp(tampon, "islev") == 0 || strcmp(tampon, "degisken") == 0 ||
                strcmp(tampon, "dongu") == 0 || strcmp(tampon, "ise") == 0 ||
                strcmp(tampon, "eger") == 0 ||
                strcmp(tampon, "degilse") == 0 || strcmp(tampon, "degilse_ise") == 0 ||
                strcmp(tampon, "son") == 0 || strcmp(tampon, "yazdir") == 0 || 
                strcmp(tampon, "zaman_baslat") == 0 || strcmp(tampon, "zaman_bitir") == 0 || 
                strcmp(tampon, "don") == 0 || strcmp(tampon, "metin") == 0 || 
                strcmp(tampon, "ondalik") == 0 || strcmp(tampon, "girdi") == 0 || 
                strcmp(tampon, "dizi") == 0 || strcmp(tampon, "sozluk") == 0 || 
                strcmp(tampon, "dene") == 0 || strcmp(tampon, "hata_yakala") == 0 || 
                strcmp(tampon, "hata_firlat") == 0 || strcmp(tampon, "sinif") == 0 || 
                strcmp(tampon, "yeni") == 0 || strcmp(tampon, "kir") == 0 || 
                strcmp(tampon, "devam_et") == 0 || strcmp(tampon, "sabit") == 0 ||
                strcmp(tampon, "Dogru") == 0 || strcmp(tampon, "Yanlis") == 0 ||
                strcmp(tampon, "hic") == 0 || strcmp(tampon, "dahil_et") == 0) {
                ekle_token(&dizi, TOKEN_ANAHTAR, tampon, satir);
            } else if (strcmp(tampon, "ve") == 0) {
                ekle_token(&dizi, TOKEN_VE, "&&", satir);
            } else if (strcmp(tampon, "veya") == 0) {
                ekle_token(&dizi, TOKEN_VEYA, "||", satir);
            } else if (strcmp(tampon, "degil") == 0) {
                ekle_token(&dizi, TOKEN_ID, "!", satir); // Şimdilik id ama ! basacak
            } else {
                ekle_token(&dizi, TOKEN_ID, tampon, satir);
            }
            continue;
        }

        if (isdigit(c)) {
            char tampon[256];
            int j = 0;
            // Noktayı da dahil ettik (Ondalıklı sayılar için)
            while (isdigit(kaynak_kod[i]) || kaynak_kod[i] == '.') {
                tampon[j++] = kaynak_kod[i++];
            }
            tampon[j] = '\0';
            ekle_token(&dizi, TOKEN_SAYI, tampon, satir);
            continue;
        }

        // Semboller
        if (c == '=' && kaynak_kod[i+1] == '=') { ekle_token(&dizi, TOKEN_ESIT_ESIT, "==", satir); i+=2; continue; }
        if (c == '<' && kaynak_kod[i+1] == '=') { ekle_token(&dizi, TOKEN_KUCUK_ESIT, "<=", satir); i+=2; continue; }
        if (c == '>' && kaynak_kod[i+1] == '=') { ekle_token(&dizi, TOKEN_BUYUK_ESIT, ">=", satir); i+=2; continue; }
        
        if (c == '=') { ekle_token(&dizi, TOKEN_ESIT, "=", satir); i++; continue; }
        if (c == '+') { ekle_token(&dizi, TOKEN_ARTI, "+", satir); i++; continue; }
        if (c == '-') { ekle_token(&dizi, TOKEN_EKS, "-", satir); i++; continue; }
        if (c == '*') { ekle_token(&dizi, TOKEN_CARP, "*", satir); i++; continue; }
        if (c == '/') { ekle_token(&dizi, TOKEN_BOL, "/", satir); i++; continue; }
        if (c == '%') { ekle_token(&dizi, TOKEN_MOD, "%", satir); i++; continue; }
        if (c == '<') { ekle_token(&dizi, TOKEN_KUCUK, "<", satir); i++; continue; }
        if (c == '>') { ekle_token(&dizi, TOKEN_BUYUK, ">", satir); i++; continue; }
        if (c == ':') { ekle_token(&dizi, TOKEN_IKINOKTA, ":", satir); i++; continue; }
        if (c == '(') { ekle_token(&dizi, TOKEN_PARANTEZ_AC, "(", satir); i++; continue; }
        if (c == ')') {
            ekle_token(&dizi, TOKEN_PARANTEZ_KAPA, ")", satir);
            i++;
            continue;
        } else if (c == '[') {
            ekle_token(&dizi, TOKEN_KOSELI_AC, "[", satir);
            i++;
            continue;
        } else if (c == ']') {
            ekle_token(&dizi, TOKEN_KOSELI_KAPA, "]", satir);
            i++;
            continue;
        } else if (c == '.') {
            ekle_token(&dizi, TOKEN_NOKTA, ".", satir);
            i++;
            continue;
        } else if (c == ',') { ekle_token(&dizi, TOKEN_VIRGUL, ",", satir); i++; continue; }

        i++; // Bilinmeyen karakteri atla
    }

    ekle_token(&dizi, TOKEN_EOF, NULL, satir);
    return dizi;
}

void free_token_array(TokenArray* array) {
    for (int i = 0; i < array->count; i++) {
        if (array->tokens[i].deger) {
            free(array->tokens[i].deger);
        }
    }
    if (array->tokens) {
        free(array->tokens);
    }
    array->count = 0;
    array->capacity = 0;
}
