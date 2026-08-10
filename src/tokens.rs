// Token tipleri — OZ+ (Türkçe sözdizimi) için lexer çıktısı.
// C'deki enum TokenType karşılığı, Rust enum'una birebir taşındı.

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenTip {
    ANAHTAR,        // islev, degisken, dongu, ise, son, ...
    ID,             // değişken/fonksiyon adı
    METIN,          // "..." string
    SAYI,           // 123 veya 12.5
    ESIT,           // =
    ESIT_ESIT,      // ==
    ESIT_DEGIL,     // !=
    ARTI,           // +
    EKS,            // -
    OK,             // ->
    CARP,           // *
    BOL,            // /
    MOD,            // %
    VE,             // && (kelime: ve)
    VEYA,           // || (kelime: veya)
    IKINOKTA,       // :
    KUCUK,          // <
    BUYUK,          // >
    KUCUK_ESIT,     // <=
    BUYUK_ESIT,     // >=
    PARANTEZ_AC,    // (
    PARANTEZ_KAPA,  // )
    KOSELI_AC,      // [
    KOSELI_KAPA,    // ]
    SUSLU_AC,       // {
    SUSLU_KAPA,     // }
    NOKTA,          // .
    VIRGUL,         // ,
    EOF,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub tip: TokenTip,
    pub deger: String,
    pub satir: u32,
}

impl Token {
    pub fn yeni(tip: TokenTip, deger: String, satir: u32) -> Self {
        Self { tip, deger, satir }
    }

    pub fn eof(satir: u32) -> Self {
        Self { tip: TokenTip::EOF, deger: String::new(), satir }
    }

    pub fn anahtar_mi(&self, kelime: &str) -> bool {
        self.tip == TokenTip::ANAHTAR && self.deger == kelime
    }
}

#[derive(Debug, Default)]
pub struct TokenDizisi {
    pub tokens: Vec<Token>,
}

impl TokenDizisi {
    pub fn ekle(&mut self, tok: Token) {
        self.tokens.push(tok);
    }
}