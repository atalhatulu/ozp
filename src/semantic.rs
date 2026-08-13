// OZ+ Semantic Analyzer
// Parser AST ürettikten sonra, Codegen'den ÖNCE çalışan doğrulama katmanı:
//   - Sembol çözümleme (tanımsız değişken / fonksiyon kullanımı)
//   - Scope analizi (blok kapsamı)
//   - Tip çıkarımı (değişken tipini ilk atamadan çıkar)
//   - Atama tip kontrolü (degisken x = 10; x = "metin" → uyarı)
//   - Kırılma/geri-dönüş scope doğrulaması (kir/devam_et dongu içinde olmalı)
//
// Bağımsız ve güvenli: AST'yi hiç değiştirmez, sadece doğrular. Codegen'e
// sıfır regresyon riski. Hatalar CompileHata olarak raporlanır.

use crate::ast::{Ifade, Komut, Program, Tip};
use crate::parser::CompileHata;

/// Gömülü (builtin) fonksiyonlar — kullanıcı tanımlı olması beklenmez.
const BUILTIN_FONKSIYONLAR: &[&str] = &[
    "dosya_oku", "dosya_yaz", "rastgele", "tip", "zaman", "yazdir",
    "sozluk_olustur", "sozluk_ekle", "sozluk_getir", "sozluk_sil", "sozluk_var_mi",
    "dizi_ekle", "dizi_uzunluk", "dizi_getir", "metin_uzunluk", "metin_birlestir",
    "sayiya_cevir", "metne_cevir", "ust", "kok", "mutlak", "sinirla", "yuvarla",
    "min", "max", "kullanici_girdisi",
];

/// Scope yığını: her öğe o kapsamdaki değişken → tip haritası.
#[derive(Default)]
struct ScopeYigini {
    yigin: Vec<Vec<(String, Tip)>>,
}

impl ScopeYigini {
    fn acik_scope(&mut self) {
        self.yigin.push(Vec::new());
    }
    fn kapat(&mut self) {
        self.yigin.pop();
    }
    fn tanimla(&mut self, isim: &str, tip: Tip) {
        if let Some(scope) = self.yigin.last_mut() {
            if let Some(mevcut) = scope.iter_mut().find(|(n, _)| n == isim) {
                mevcut.1 = tip;
                return;
            }
            scope.push((isim.to_string(), tip));
        }
    }
    /// Güncel tüm kapsamlarda ara (en içten dışa).
    fn tip_getir(&self, isim: &str) -> Option<Tip> {
        for scope in self.yigin.iter().rev() {
            if let Some((_, t)) = scope.iter().find(|(n, _)| n == isim) {
                return Some(*t);
            }
        }
        None
    }
}

pub struct Semantik {
    scopes: ScopeYigini,
    fonksiyonlar: Vec<(String, usize)>, // (isim, parametre sayısı) — kullanıcı tanımlı
    siniflar: Vec<String>,
    hatalar: Vec<CompileHata>,
    derin_dongu: usize,   // aktif döngü derinliği (kir/devam_et kontrolü)
    derin_islev: usize,   // aktif fonksiyon derinliği (don kontrolü)
}

impl Semantik {
    pub fn yeni() -> Self {
        let mut s = Semantik {
            scopes: ScopeYigini::default(),
            fonksiyonlar: Vec::new(),
            siniflar: Vec::new(),
            hatalar: Vec::new(),
            derin_dongu: 0,
            derin_islev: 0,
        };
        s.scopes.acik_scope(); // global scope
        s
    }

    fn hata(&mut self, mesaj: String) {
        self.hatalar.push(CompileHata { satir: 0, mesaj });
    }

    // ---- İfade tipi çıkarımı ----
    fn ifade_tipi(&mut self, ifade: &Ifade) -> Tip {
        match ifade {
            Ifade::Sayi(_) => {
                // Ondalıklı mı tamsayı mı olduğu değerden anlaşılır ama
                // AST'de f64 tutuluyor; Ondalik olarak işaretle. Tip sadece
                // tutarlılık için kullanılır, fark kritik değil.
                Tip::Ondalik
            }
            Ifade::Metin(_) => Tip::Metin,
            Ifade::Dogru | Ifade::Yanlis => Tip::TamSayi, // mantıksal → sayı olarak işle
            Ifade::Hic => Tip::TamSayi,
            Ifade::Degisken(isim) => {
                if let Some(t) = self.scopes.tip_getir(isim) {
                    t
                } else {
                    self.hata(format!("Tanımsız değişken kullanimi: '{}'. Once 'degisken' ile tanimlayin.", isim));
                    Tip::TamSayi // tahmini; akışı sürdürmek için
                }
            }
            Ifade::Ikili { sol, sag, op } => {
                // İkili işlem: kıyas/ve/veya mantıksaldır, diğerleri sayısal.
                self.ifade_tipi(sol);
                self.ifade_tipi(sag);
                match op {
                    crate::ast::Op::Esit | crate::ast::Op::EsitDegil
                    | crate::ast::Op::Kucuk | crate::ast::Op::Buyuk
                    | crate::ast::Op::KucukEsit | crate::ast::Op::BuyukEsit
                    | crate::ast::Op::Ve | crate::ast::Op::Veya => Tip::TamSayi,
                    _ => self.ifade_tipi(sol),
                }
            }
            Ifade::Cagri { isim, argumanlar } => {
                for a in argumanlar {
                    self.ifade_tipi(a);
                }
                self.fonksiyon_return_tipi(isim, argumanlar.len())
            }
            Ifade::MetotCagrisi { nesne, argumanlar, .. } => {
                self.ifade_tipi(nesne);
                for a in argumanlar {
                    self.ifade_tipi(a);
                }
                Tip::TamSayi // metot dönüş tipi bilinmez; esnek davran
            }
            Ifade::YeniNesne(_) => Tip::Sozluk,
            Ifade::Dizi(_) => Tip::Dizi,
            Ifade::Sozluk(_) => Tip::Sozluk,
            Ifade::DiziErisim { dizi, .. } => {
                let t = self.ifade_tipi(dizi);
                if t != Tip::Dizi && t != Tip::Sozluk {
                    self.hata(format!("Dizi/sozluk erisimi bekleniyordu. Gercek tip: {:?}.", t));
                }
                Tip::TamSayi
            }
            Ifade::UyeErisim { nesne, .. } => {
                self.ifade_tipi(nesne);
                Tip::TamSayi
            }
        }
    }

    fn fonksiyon_return_tipi(&mut self, isim: &str, arg_sayisi: usize) -> Tip {
        if BUILTIN_FONKSIYONLAR.contains(&isim) {
            // Builtin'lerin çoğu OzDeger döner; tip bilinmezse esnek davran.
            return Tip::TamSayi;
        }
        if let Some((_, tanimli_arg)) = self.fonksiyonlar.iter().find(|(n, _)| n == isim) {
            if *tanimli_arg != arg_sayisi {
                self.hata(format!(
                    "Fonksiyon '{}' cagrisi: beklenen parametre sayisi {}, verilen {}.",
                    isim, tanimli_arg, arg_sayisi
                ));
            }
            Tip::TamSayi
        } else {
            self.hata(format!("Tanimsiz fonksiyon cagrisi: '{}'.", isim));
            Tip::TamSayi
        }
    }

    // ---- Komut doğrulama ----
    fn komut_dogrula(&mut self, komut: &Komut, sinif_govde: bool) {
        match komut {
            Komut::DegiskenTanimla { isim, tip, deger } => {
                // Tip çıkarımı: değer varsa değerden, yoksa deklare tipten.
                let cikarilan_tip = deger.as_ref().map(|d| self.ifade_tipi(d)).unwrap_or(*tip);
                self.scopes.tanimla(isim, cikarilan_tip);
            }
            Komut::Atama { hedef, deger } => {
                let deger_tip = self.ifade_tipi(deger);
                match hedef {
                    Ifade::Cagri { isim, argumanlar } => {
                        // Void fonksiyon çağrısı: fonksiyon() = Dogru şeklinde
                        // parse edilir. Argümanları doğrula, çağrıyı kontrol et.
                        for a in argumanlar {
                            self.ifade_tipi(a);
                        }
                        self.fonksiyon_return_tipi(isim, argumanlar.len());
                    }
                    Ifade::MetotCagrisi { nesne, argumanlar, .. } => {
                        self.ifade_tipi(nesne);
                        for a in argumanlar {
                            self.ifade_tipi(a);
                        }
                    }
                    Ifade::Degisken(isim) => {
                        if let Some(mevcut_tip) = self.scopes.tip_getir(isim) {
                            if mevcut_tip != deger_tip {
                                self.hata(format!(
                                    "Atama tip uyumsuzlugu: '{}' tipi {:?} ama {:?} atanmaya calisildi.",
                                    isim, mevcut_tip, deger_tip
                                ));
                            }
                        } else {
                            self.hata(format!(
                                "Atama hedefi tanimsiz degisken: '{}'. Once 'degisken' ile tanimlayin.",
                                isim
                            ));
                        }
                    }
                    Ifade::DiziErisim { dizi, .. } => {
                        self.ifade_tipi(dizi);
                    }
                    Ifade::UyeErisim { nesne, .. } => {
                        self.ifade_tipi(nesne);
                    }
                    _ => {
                        self.hata("Atama hedefi desteklenmeyen ifade.".into());
                    }
                }
            }
            Komut::Ise { kosul, then, else_ } => {
                self.ifade_tipi(kosul);
                self.scopes.acik_scope();
                for k in then {
                    self.komut_dogrula(k, sinif_govde);
                }
                self.scopes.kapat();
                if let Some(e) = else_ {
                    self.scopes.acik_scope();
                    for k in e {
                        self.komut_dogrula(k, sinif_govde);
                    }
                    self.scopes.kapat();
                }
            }
            Komut::Dongu { kosul, govde } => {
                self.ifade_tipi(kosul);
                self.scopes.acik_scope();
                self.derin_dongu += 1;
                for k in govde {
                    self.komut_dogrula(k, sinif_govde);
                }
                self.derin_dongu -= 1;
                self.scopes.kapat();
            }
            Komut::Her { eleman, koleksiyon, govde } => {
                self.ifade_tipi(koleksiyon);
                self.scopes.acik_scope();
                self.scopes.tanimla(eleman, Tip::TamSayi); // döngü elemanı
                self.derin_dongu += 1;
                for k in govde {
                    self.komut_dogrula(k, sinif_govde);
                }
                self.derin_dongu -= 1;
                self.scopes.kapat();
            }
            Komut::Yazdir(ifade) | Komut::HataFirlat(ifade) => {
                self.ifade_tipi(ifade);
            }
            Komut::YazdirSabit(_) => {}
            Komut::GirdiAl(isim) | Komut::GirdiSayi(isim) => {
                let tip = if matches!(komut, Komut::GirdiAl(_)) { Tip::Metin } else { Tip::TamSayi };
                self.scopes.tanimla(isim, tip);
            }
            Komut::Don(Some(ifade)) => {
                if self.derin_islev == 0 {
                    self.hata("'don' yalnizca bir 'islev' icerisinde kullanilabilir.".into());
                }
                self.ifade_tipi(ifade);
            }
            Komut::Don(None) => {
                if self.derin_islev == 0 {
                    self.hata("'don' yalnizca bir 'islev' icerisinde kullanilabilir.".into());
                }
            }
            Komut::Kir | Komut::DevamEt => {
                if self.derin_dongu == 0 {
                    let hangi = if matches!(komut, Komut::Kir) { "kir" } else { "devam_et" };
                    self.hata(format!("'{}' yalnizca bir dongu icerisinde kullanilabilir.", hangi));
                }
            }
            Komut::Islev { isim, parametreler, govde, .. } => {
                self.fonksiyonlar.push((isim.clone(), parametreler.len()));
                self.scopes.acik_scope();
                for (p_ad, p_tip) in parametreler {
                    let tip = p_tip.as_deref()
                        .map(tip_cevir)
                        .unwrap_or(Tip::TamSayi);
                    self.scopes.tanimla(p_ad, tip);
                }
                self.derin_islev += 1;
                for k in govde {
                    self.komut_dogrula(k, sinif_govde);
                }
                self.derin_islev -= 1;
                self.scopes.kapat();
            }
            Komut::Sinif { isim, govde } => {
                self.siniflar.push(isim.clone());
                self.scopes.acik_scope();
                for k in govde {
                    self.komut_dogrula(k, true);
                }
                self.scopes.kapat();
            }
            Komut::DeneYakala { dene_govde, hata_degiskeni, yakala_govde } => {
                self.scopes.acik_scope();
                for k in dene_govde {
                    self.komut_dogrula(k, sinif_govde);
                }
                self.scopes.kapat();
                self.scopes.acik_scope();
                self.scopes.tanimla(hata_degiskeni, Tip::Metin);
                for k in yakala_govde {
                    self.komut_dogrula(k, sinif_govde);
                }
                self.scopes.kapat();
            }
            Komut::Sabit { isim, deger } => {
                self.ifade_tipi(deger);
                self.scopes.tanimla(isim, Tip::Metin);
            }
            Komut::Eger | Komut::Bos | Komut::ZamanBaslat | Komut::ZamanBitir
            | Komut::DahilEt(_) => {}
        }
    }

    // ---- Giriş noktası ----
    pub fn analiz_et(mut self, program: &Program) -> Vec<CompileHata> {
        // Global sabitleri ve fonksiyonları önce kaydet (ileri referanslar için ilk geçiş).
        for k in &program.komutlar {
            if let Komut::Islev { isim, parametreler, .. } = k {
                self.fonksiyonlar.push((isim.clone(), parametreler.len()));
            }
            if let Komut::Sinif { isim, .. } = k {
                self.siniflar.push(isim.clone());
            }
        }
        // İkinci geçiş: gövdeyi en baştan (global scope) doğrula.
        self.scopes.acik_scope();
        for k in &program.komutlar {
            self.komut_dogrula(k, false);
        }
        self.scopes.kapat();
        self.hatalar
    }
}

/// Metin tip adını Tip enum'una çevirir (islev parametre/donus tipleri için).
fn tip_cevir(ad: &str) -> Tip {
    match ad {
        "metin" | "string" | "char" => Tip::Metin,
        "ondalik" | "double" | "float" => Tip::Ondalik,
        "dizi" | "array" => Tip::Dizi,
        "sozluk" | "map" => Tip::Sozluk,
        _ => Tip::TamSayi, // tam sayı / diğer
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer;
    use crate::parser;

    fn analiz(kod: &str) -> Vec<CompileHata> {
        let tokens = lexer::tokenize(kod);
        let program = parser::parse(&tokens).unwrap();
        Semantik::yeni().analiz_et(&program)
    }

    fn hata_var_mi(hatalar: &[CompileHata], anahtar: &str) -> bool {
        hatalar.iter().any(|h| h.mesaj.contains(anahtar))
    }

    #[test]
    fn tanimsiz_degisken() {
        let h = analiz("yazdir olmayan_degisken");
        assert!(hata_var_mi(&h, "olmayan_degisken"), "mesajlar: {:?}", h);
    }

    #[test]
    fn tanimli_degisken_sorunsuz() {
        let h = analiz("degisken x = 10\nyazdir x");
        assert!(h.is_empty(), "beklenmedik hata: {:?}", h);
    }

    #[test]
    fn tanimsiz_atama() {
        let h = analiz("degisken x = 10\natama_hedefi = 5");
        // atama_hedefi tanımsız olmalı
        assert!(hata_var_mi(&h, "atama_hedefi"), "mesajlar: {:?}", h);
    }

    #[test]
    fn tip_uyusmazligi() {
        let h = analiz("degisken x = 10\nx = \"merhaba\"");
        assert!(hata_var_mi(&h, "tip uyumsuzlugu"), "mesajlar: {:?}", h);
    }

    #[test]
    fn tanimsiz_fonksiyon() {
        let h = analiz("degisken s = baska_fonksiyon()");
        assert!(hata_var_mi(&h, "Tanimsiz fonksiyon"), "mesajlar: {:?}", h);
    }

    #[test]
    fn don_dongu_disinda() {
        let h = analiz("don");
        assert!(hata_var_mi(&h, "don"), "mesajlar: {:?}", h);
    }

    #[test]
    fn kir_dongu_disinda() {
        let h = analiz("kir");
        assert!(hata_var_mi(&h, "kir"), "mesajlar: {:?}", h);
    }
}
