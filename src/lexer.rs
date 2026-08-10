// OZ+ lexer'ı — kaynak kodu token'lara ayırır.
// legacy/src/lexer.c mantığı birebir taşındı, güvenli Rust implementasyonu.

use crate::tokens::{Token, TokenDizisi, TokenTip};

// Türkçe anahtar kelimeler
fn anahtar_mi(kelime: &str) -> bool {
    matches!(
        kelime,
        "islev" | "degisken" | "dongu" | "ise" | "eger" | "degilse"
            | "degilse_ise" | "son" | "yazdir" | "zaman_baslat"
            | "zaman_bitir" | "don" | "metin" | "ondalik" | "girdi"
            | "dizi" | "sozluk" | "dene" | "yakala" | "hata_firlat"
            | "sinif" | "yeni" | "kir" | "devam_et" | "sabit" | "Dogru"
            | "Yanlis" | "hic" | "dahil_et" | "her" | "icinde"
    )
}

pub fn tokenize(kaynak_kod: &str) -> TokenDizisi {
    let chars: Vec<char> = kaynak_kod.chars().collect();
    let mut dizi = TokenDizisi::default();
    let mut i = 0usize;
    let mut satir: u32 = 1;

    while i < chars.len() {
        let c = chars[i];

        if c == '\n' {
            satir += 1;
            i += 1;
            continue;
        }
        if c.is_whitespace() {
            i += 1;
            continue;
        }

        // Yorum satırları: //
        if c == '/' && i + 1 < chars.len() && chars[i + 1] == '/' {
            while i < chars.len() && chars[i] != '\n' {
                i += 1;
            }
            continue;
        }

        // String: "..."
        if c == '"' {
            i += 1;
            let mut buf = String::new();
            while i < chars.len() && chars[i] != '"' {
                buf.push(chars[i]);
                i += 1;
            }
            if i < chars.len() && chars[i] == '"' {
                i += 1;
            }
            dizi.ekle(Token::yeni(TokenTip::METIN, buf, satir));
            continue;
        }

        // Tanımlayıcı / anahtar kelime
        if c.is_alphabetic() || c == '_' {
            let mut buf = String::new();
            while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                buf.push(chars[i]);
                i += 1;
            }

            if anahtar_mi(&buf) {
                dizi.ekle(Token::yeni(TokenTip::ANAHTAR, buf, satir));
            } else if buf == "ve" {
                dizi.ekle(Token::yeni(TokenTip::VE, "&&".into(), satir));
            } else if buf == "veya" {
                dizi.ekle(Token::yeni(TokenTip::VEYA, "||".into(), satir));
            } else if buf == "degil" {
                // Şimdilik ID ama "!" basacak (C sürümüyle uyumlu)
                dizi.ekle(Token::yeni(TokenTip::ID, "!".into(), satir));
            } else {
                dizi.ekle(Token::yeni(TokenTip::ID, buf, satir));
            }
            continue;
        }

        // Sayı (ondalık dahil)
        if c.is_ascii_digit() {
            let mut buf = String::new();
            while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.') {
                buf.push(chars[i]);
                i += 1;
            }
            dizi.ekle(Token::yeni(TokenTip::SAYI, buf, satir));
            continue;
        }

        // İki karakterli semboller
        if c == '=' && i + 1 < chars.len() && chars[i + 1] == '=' {
            dizi.ekle(Token::yeni(TokenTip::ESIT_ESIT, "==".into(), satir));
            i += 2;
            continue;
        }
        if (c == '<' || c == '>') && i + 1 < chars.len() && chars[i + 1] == '=' {
            let tip = if c == '<' { TokenTip::KUCUK_ESIT } else { TokenTip::BUYUK_ESIT };
            let deger: String = chars[i..=i + 1].iter().collect();
            dizi.ekle(Token::yeni(tip, deger, satir));
            i += 2;
            continue;
        }
        if c == '=' {
            dizi.ekle(Token::yeni(TokenTip::ESIT, "=".into(), satir));
            i += 1;
            continue;
        }
        if c == '!' && i + 1 < chars.len() && chars[i + 1] == '=' {
            dizi.ekle(Token::yeni(TokenTip::ESIT_DEGIL, "!=".into(), satir));
            i += 2;
            continue;
        }

        // Tek karakterli semboller ve ok
        if c == '-' && i + 1 < chars.len() && chars[i + 1] == '>' {
            dizi.ekle(Token::yeni(TokenTip::OK, "->".into(), satir));
            i += 2;
            continue;
        }

        let (tip, deger) = match c {
            '+' => (TokenTip::ARTI, "+".to_string()),
            '-' => (TokenTip::EKS, "-".to_string()),
            '*' => (TokenTip::CARP, "*".to_string()),
            '/' => (TokenTip::BOL, "/".to_string()),
            '%' => (TokenTip::MOD, "%".to_string()),
            '<' => (TokenTip::KUCUK, "<".to_string()),
            '>' => (TokenTip::BUYUK, ">".to_string()),
            ':' => (TokenTip::IKINOKTA, ":".to_string()),
            '(' => (TokenTip::PARANTEZ_AC, "(".to_string()),
            ')' => (TokenTip::PARANTEZ_KAPA, ")".to_string()),
            '[' => (TokenTip::KOSELI_AC, "[".to_string()),
            ']' => (TokenTip::KOSELI_KAPA, "]".to_string()),
            '{' => (TokenTip::SUSLU_AC, "{".to_string()),
            '}' => (TokenTip::SUSLU_KAPA, "}".to_string()),
            '.' => (TokenTip::NOKTA, ".".to_string()),
            ',' => (TokenTip::VIRGUL, ",".to_string()),
            _ => {
                i += 1; // Bilinmeyen karakteri atla
                continue;
            }
        };
        dizi.ekle(Token::yeni(tip, deger, satir));
        i += 1;
    }

    dizi.ekle(Token::eof(satir));
    dizi
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basit_degisken_tanima() {
        let t = tokenize("degisken n = 2");
        assert_eq!(t.tokens[0].anahtar_mi("degisken"), true);
        assert_eq!(t.tokens[1].tip, TokenTip::ID);
        assert_eq!(t.tokens[1].deger, "n");
        assert_eq!(t.tokens[2].tip, TokenTip::ESIT);
        assert_eq!(t.tokens[3].tip, TokenTip::SAYI);
        assert_eq!(t.tokens[3].deger, "2");
        assert_eq!(t.tokens[4].tip, TokenTip::EOF);
    }

    #[test]
    fn yorum_yok_sayilir() {
        let t = tokenize("degisken x = 1 // yorum\ndegisken y = 2");
        assert_eq!(t.tokens[0].anahtar_mi("degisken"), true);
        // iki degisken + sayilar var, yorum token üretmez
        assert_eq!(t.tokens.iter().filter(|x| x.tip == TokenTip::ID).count(), 2);
    }

    #[test]
    fn satir_numaralari() {
        let t = tokenize("degisken a = 1\ndegisken b = 2");
        let ikinci_degisken = t.tokens.iter().filter(|x| x.anahtar_mi("degisken")).collect::<Vec<_>>();
        assert_eq!(ikinci_degisken[1].satir, 2);
    }
}