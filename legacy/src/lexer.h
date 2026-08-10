#ifndef LEXER_H
#define LEXER_H

typedef enum {
    TOKEN_ANAHTAR, // islev, degisken, dongu, ise, degilse, son, yazdir vs.
    TOKEN_ID,      // değişken/fonksiyon adları vb.
    TOKEN_SAYI,    // 123
    TOKEN_METIN,   // "merhaba"
    TOKEN_ESIT,    // =
    TOKEN_ARTI,    // +
    TOKEN_EKS,     // -
    TOKEN_CARP,    // *
    TOKEN_BOL,     // /
    TOKEN_MOD,     // %
    TOKEN_KUCUK,   // <
    TOKEN_BUYUK,   // >
    TOKEN_KUCUK_ESIT, // <=
    TOKEN_BUYUK_ESIT, // >=
    TOKEN_ESIT_ESIT,  // ==
    TOKEN_ESIT_DEGIL, // !=
    TOKEN_VE,         // &&
    TOKEN_VEYA,       // ||
    TOKEN_IKINOKTA,   // :
    TOKEN_PARANTEZ_AC,// (
    TOKEN_PARANTEZ_KAPA, // )
    TOKEN_KOSELI_AC,  // [
    TOKEN_KOSELI_KAPA,// ]
    TOKEN_NOKTA,      // .
    TOKEN_VIRGUL,     // ,
    TOKEN_EOF         // Dosya sonu
} TokenType;

typedef struct {
    TokenType tip;
    char* deger;
    int satir;
} Token;

typedef struct {
    Token* tokens;
    int count;
    int capacity;
} TokenArray;

TokenArray tokenize(const char* kaynak_kod);
void free_token_array(TokenArray* array);

#endif
