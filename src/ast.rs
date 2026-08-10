// OZ+ AST — kod ağacı. parser bu yapıyı üretir, codegen C diline basar.
// Bu, legacy/parser.c'deki "doğrudan fprintf" yaklaşımından ayrılıştır:
// token'lar önce gerçek bir AST'ye kurulur, sonra kod üretilir.

use std::collections::HashMap;

// --- İfade düğümleri ---
#[derive(Debug, Clone, PartialEq)]
pub enum Ifade {
    Sayi(f64),
    Metin(String),
    Degisken(String),
    Ikili {
        sol: Box<Ifade>,
        op: Op,
        sag: Box<Ifade>,
    },
    Cagri {
        isim: String,
        argumanlar: Vec<Ifade>,
    },
    MetotCagrisi {
        nesne: Box<Ifade>,
        metot: String,
        argumanlar: Vec<Ifade>,
    },
    YeniNesne(String),
    Dizi(Vec<Ifade>),
    Sozluk(Vec<(String, Ifade)>),
    DiziErisim {
        dizi: Box<Ifade>,
        indeks: Box<Ifade>,
    },
    UyeErisim {
        nesne: Box<Ifade>,
        uye: String,
    },
    Dogru,
    Yanlis,
    Hic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Op {
    Topla,
    Cikar,
    Carp,
    Bol,
    Mod,
    Esit,
    EsitDegil,
    Kucuk,
    Buyuk,
    KucukEsit,
    BuyukEsit,
    Ve,
    Veya,
}

// --- Komut düğümleri (sözdizimi) ---
#[derive(Debug, Clone, PartialEq)]
pub enum Komut {
    DegiskenTanimla {
        isim: String,
        tip: Tip,
        deger: Option<Ifade>,
    },
    Atama {
        hedef: Ifade,
        deger: Ifade,
    },
    Dongu {
        kosul: Ifade,
        govde: Vec<Komut>,
    },
    Her {
        eleman: String,
        koleksiyon: Ifade,
        govde: Vec<Komut>,
    },
    Ise {
        kosul: Ifade,
        then: Vec<Komut>,
        else_: Option<Vec<Komut>>,
    },
    Eger, // üçlü operatör: eger kosul, dogru : yanlis (implementasyon: Bos'dur)
    Yazdir(Ifade),
    YazdirSabit(String),
    GirdiAl(String),       // girdi degisken al
    GirdiSayi(String),     // girdi degisken  (sayısal)
    ZamanBaslat,
    ZamanBitir,
    Don(Option<Ifade>),
    Kir,
    DevamEt,
    Islev {
        isim: String,
        parametreler: Vec<(String, Option<String>)>, // (Parametre Adı, Tipi)
        donus_tipi: Option<String>,
        govde: Vec<Komut>,
    },
    Sinif {
        isim: String,
        govde: Vec<Komut>,
    },
    DeneYakala {
        dene_govde: Vec<Komut>,
        hata_degiskeni: String,
        yakala_govde: Vec<Komut>,
    },
    HataFirlat(Ifade),
    Sabit {
        isim: String,
        deger: Ifade,
    },
    DahilEt(String),
    Bos,
}

// --- Tipler ---
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tip {
    TamSayi,    // long / degisken
    Metin,      // char* / metin
    Ondalik,    // double / ondalik
    Dizi,       // long[] / dizi
    Sozluk,     // OzSozluk* / sozluk
}

// --- Sembol tablosu (scope yığını ile) ---
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Sembol {
    pub isim: String,
    pub tip: Tip,
    pub scope: u32,
}

#[derive(Debug, Default)]
pub struct SembolTablosu {
    pub semboller: Vec<Sembol>,
    pub current_scope: u32,
}

#[allow(dead_code)]
impl SembolTablosu {
    pub fn tanimli_mi(&self, isim: &str) -> bool {
        self.semboller.iter().any(|s| s.isim == isim)
    }

    pub fn tip_getir(&self, isim: &str) -> Option<Tip> {
        self.semboller.iter().rev().find(|s| s.isim == isim).map(|s| s.tip)
    }

    pub fn tanimla(&mut self, isim: &str, tip: Tip) -> bool {
        if self.tanimli_mi(isim) {
            return false;
        }
        self.semboller.push(Sembol {
            isim: isim.to_string(),
            tip,
            scope: self.current_scope,
        });
        true
    }

    pub fn scope_artir(&mut self) {
        self.current_scope += 1;
    }

    pub fn scope_dusur(&mut self) -> Vec<Sembol> {
        self.current_scope = self.current_scope.saturating_sub(1);
        let onceki = self.current_scope;
        let durumlar: Vec<Sembol> = self
            .semboller
            .iter()
            .filter(|s| s.scope > onceki)
            .cloned()
            .collect();
        self.semboller.retain(|s| s.scope <= onceki);
        durumlar // kapsamdışı kalan sembolleri döndür (varsa clean-up için)
    }

    pub fn scope_sigari(&self, isim: &str) -> bool {
        self.semboller
            .iter()
            .filter(|s| s.scope == self.current_scope)
            .any(|s| s.isim == isim)
    }
}

// --- AST kökü ---
#[derive(Debug, Default)]
pub struct Program {
    pub komutlar: Vec<Komut>,
    pub sinif_haritasi: HashMap<String, Vec<Komut>>,
}