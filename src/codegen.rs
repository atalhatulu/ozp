use crate::ast::{Ifade, Komut, Program, Op};

pub struct Transpiler {
    rust_kodu: String,
}

impl Transpiler {
    pub fn yeni() -> Self {
        Self { rust_kodu: String::new() }
    }

    pub fn transpile(&mut self, program: &Program) -> Result<String, String> {
        self.rust_kodu.push_str("#[allow(warnings)]\n");
        self.rust_kodu.push_str("use std::ops::{Add, Sub, Mul, Div};\n\n");
        
// Dinamik Tip (OzDeger) Motoru - Aşama 1: Diziler ve Dinamik Değişkenler
        self.rust_kodu.push_str(r#"
use std::fmt;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub enum OzDeger {
    Sayi(f64),
    Metin(String),
    Dizi(Arc<Mutex<Vec<OzDeger>>>),
    Sozluk(Arc<Mutex<HashMap<String, OzDeger>>>),
    Islev(Arc<dyn Fn(Vec<OzDeger>) -> OzDeger + Send + Sync>),
    Dogru,
    Yanlis,
    Hic,
}

impl fmt::Debug for OzDeger {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OzDeger::Sayi(n) => write!(f, "Sayi({})", n),
            OzDeger::Metin(s) => write!(f, "Metin({:?})", s),
            OzDeger::Dizi(d) => write!(f, "Dizi({:?})", d.lock().unwrap()),
            OzDeger::Sozluk(d) => write!(f, "Sozluk({:?})", d.lock().unwrap()),
            OzDeger::Islev(_) => write!(f, "Islev(<fn>)"),
            OzDeger::Dogru => write!(f, "Dogru"),
            OzDeger::Yanlis => write!(f, "Yanlis"),
            OzDeger::Hic => write!(f, "Hic"),
        }
    }
}

impl PartialEq for OzDeger {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (OzDeger::Sayi(a), OzDeger::Sayi(b)) => a == b,
            (OzDeger::Metin(a), OzDeger::Metin(b)) => a == b,
            (OzDeger::Dogru, OzDeger::Dogru) => true,
            (OzDeger::Yanlis, OzDeger::Yanlis) => true,
            (OzDeger::Hic, OzDeger::Hic) => true,
            // Dizi, Sozluk ve Islev referans karsilastirmasi su anlik basite indirgendi
            _ => false,
        }
    }
}

impl OzDeger {
    pub fn is_truthy(&self) -> bool {
        match self {
            OzDeger::Yanlis | OzDeger::Hic => false,
            OzDeger::Sayi(n) if *n == 0.0 => false,
            OzDeger::Metin(s) if s.is_empty() => false,
            _ => true,
        }
    }

    pub fn get_idx(&self, idx: &OzDeger) -> OzDeger {
        match (self, idx) {
            (OzDeger::Dizi(arr), OzDeger::Sayi(i)) => {
                let vec = arr.lock().unwrap();
                let index = *i as usize;
                if index < vec.len() { vec[index].clone() } else { OzDeger::Hic }
            }
            (OzDeger::Sozluk(dict), OzDeger::Metin(key)) => {
                let map = dict.lock().unwrap();
                map.get(key).cloned().unwrap_or(OzDeger::Hic)
            }
            _ => OzDeger::Hic,
        }
    }

    pub fn set_idx(&self, idx: &OzDeger, val: OzDeger) {
        match (self, idx) {
            (OzDeger::Dizi(arr), OzDeger::Sayi(i)) => {
                let mut vec = arr.lock().unwrap();
                let index = *i as usize;
                if index < vec.len() { vec[index] = val; }
            }
            (OzDeger::Sozluk(dict), OzDeger::Metin(key)) => {
                let mut map = dict.lock().unwrap();
                map.insert(key.clone(), val);
            }
            _ => {}
        }
    }

    pub fn cagir_metot(&self, metot: &str, mut args: Vec<OzDeger>) -> OzDeger {
        // Objenin kendi dinamik fonksiyonunu (Islev) sozlukten ceker
        if let OzDeger::Sozluk(dict) = self {
            let func_val = {
                let map = dict.lock().unwrap();
                map.get(metot).cloned()
            };
            if let Some(OzDeger::Islev(func)) = func_val {
                // Sınıf metodu ise (kendisi referansını ilk argüman olarak ekleriz - implicit self)
                args.insert(0, self.clone());
                return func(args);
            }
        }
        
        // Native Gömülü Metotlar
        match (self, metot) {
            (OzDeger::Dizi(arr), "ekle") => {
                if let Some(val) = args.get(0) {
                    arr.lock().unwrap().push(val.clone());
                }
                OzDeger::Hic
            }
            (OzDeger::Dizi(arr), "uzunluk") => {
                OzDeger::Sayi(arr.lock().unwrap().len() as f64)
            }
            (OzDeger::Sozluk(map), "uzunluk") => {
                OzDeger::Sayi(map.lock().unwrap().len() as f64)
            }
            (OzDeger::Metin(s), "uzunluk") => {
                OzDeger::Sayi(s.len() as f64)
            }
            (OzDeger::Metin(s), "parcala") => {
                let mut v = Vec::new();
                if let Some(OzDeger::Metin(ayrac)) = args.get(0) {
                    for parca in s.split(ayrac) {
                        v.push(OzDeger::Metin(parca.to_string()));
                    }
                }
                OzDeger::Dizi(std::sync::Arc::new(std::sync::Mutex::new(v)))
            }
            (OzDeger::Metin(s), "buyut") => OzDeger::Metin(s.to_uppercase()),
            (OzDeger::Metin(s), "kucult") => OzDeger::Metin(s.to_lowercase()),
            (OzDeger::Metin(s), "iceriyor_mu") => {
                let aranan = match args.get(0) {
                    Some(OzDeger::Metin(m)) => m.clone(),
                    _ => String::new(),
                };
                if s.contains(&aranan) { OzDeger::Sayi(1.0) } else { OzDeger::Sayi(0.0) }
            },
            (OzDeger::Dizi(d), "ekle") => {
                d.lock().unwrap().push(args.get(0).cloned().unwrap_or(OzDeger::Hic));
                OzDeger::Hic
            },
            (OzDeger::Dizi(d), "al") => {
                let idx = match args.get(0) {
                    Some(OzDeger::Sayi(n)) => *n as usize,
                    _ => 0,
                };
                d.lock().unwrap().get(idx).cloned().unwrap_or(OzDeger::Hic)
            },
            (OzDeger::Dizi(d), "degistir") => {
                let idx = match args.get(0) {
                    Some(OzDeger::Sayi(n)) => *n as usize,
                    _ => 0,
                };
                let val = args.get(1).cloned().unwrap_or(OzDeger::Hic);
                if let Some(elem) = d.lock().unwrap().get_mut(idx) {
                    *elem = val;
                }
                OzDeger::Hic
            },
            (OzDeger::Dizi(d), "uzunluk") => {
                OzDeger::Sayi(d.lock().unwrap().len() as f64)
            },
            _ => std::panic::panic_any(OzDeger::Metin(format!("Desteklenmeyen metot: {}", metot))),
        }
    }
}

impl fmt::Display for OzDeger {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OzDeger::Sayi(n) => write!(f, "{}", n),
            OzDeger::Metin(s) => write!(f, "{}", s),
            OzDeger::Dogru => write!(f, "Dogru"),
            OzDeger::Yanlis => write!(f, "Yanlis"),
            OzDeger::Hic => write!(f, "hic"),
            OzDeger::Islev(_) => write!(f, "<islev>"),
            OzDeger::Dizi(d) => {
                write!(f, "[")?;
                let vec = d.lock().unwrap();
                for (i, v) in vec.iter().enumerate() {
                    write!(f, "{}", v)?;
                    if i < vec.len() - 1 { write!(f, ", ")?; }
                }
                write!(f, "]")
            }
            OzDeger::Sozluk(d) => {
                write!(f, "{{")?;
                let map = d.lock().unwrap();
                let mut count = 0;
                for (k, v) in map.iter() {
                    write!(f, "\"{}\": {}", k, v)?;
                    if count < map.len() - 1 { write!(f, ", ")?; }
                    count += 1;
                }
                write!(f, "}}")
            }
        }
    }
}

impl Add for OzDeger {
    type Output = OzDeger;
    fn add(self, other: OzDeger) -> OzDeger {
        match (self, other) {
            (OzDeger::Sayi(a), OzDeger::Sayi(b)) => OzDeger::Sayi(a + b),
            (OzDeger::Metin(a), OzDeger::Metin(b)) => OzDeger::Metin(a + &b),
            (OzDeger::Metin(a), b) => OzDeger::Metin(format!("{}{}", a, b)),
            (a, OzDeger::Metin(b)) => OzDeger::Metin(format!("{}{}", a, b)),
            (OzDeger::Dizi(a), OzDeger::Dizi(b)) => {
                let mut new_vec = a.lock().unwrap().clone();
                new_vec.extend(b.lock().unwrap().clone());
                OzDeger::Dizi(Arc::new(Mutex::new(new_vec)))
            }
            _ => OzDeger::Hic,
        }
    }
}

impl Sub for OzDeger {
    type Output = OzDeger;
    fn sub(self, other: OzDeger) -> OzDeger {
        match (self, other) {
            (OzDeger::Sayi(a), OzDeger::Sayi(b)) => OzDeger::Sayi(a - b),
            _ => OzDeger::Hic,
        }
    }
}

impl Mul for OzDeger {
    type Output = OzDeger;
    fn mul(self, other: OzDeger) -> OzDeger {
        match (self, other) {
            (OzDeger::Sayi(a), OzDeger::Sayi(b)) => OzDeger::Sayi(a * b),
            _ => OzDeger::Hic,
        }
    }
}

impl Div for OzDeger {
    type Output = OzDeger;
    fn div(self, other: OzDeger) -> OzDeger {
        match (self, other) {
            (OzDeger::Sayi(a), OzDeger::Sayi(b)) if b != 0.0 => OzDeger::Sayi(a / b),
            _ => OzDeger::Hic,
        }
    }
}
"#);
        
        self.rust_kodu.push_str(r#"
fn dosya_oku(args: Vec<OzDeger>) -> OzDeger {
    if let Some(OzDeger::Metin(yol)) = args.get(0) {
        if let Ok(icerik) = std::fs::read_to_string(yol) {
            return OzDeger::Metin(icerik);
        }
    }
    OzDeger::Hic
}

fn dosya_yaz(args: Vec<OzDeger>) -> OzDeger {
    if let (Some(OzDeger::Metin(yol)), Some(OzDeger::Metin(veri))) = (args.get(0), args.get(1)) {
        if std::fs::write(yol, veri).is_ok() {
            return OzDeger::Dogru;
        }
    }
    OzDeger::Yanlis
}

fn rastgele(_args: Vec<OzDeger>) -> OzDeger {
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let val = (now % 1000) as f64 / 1000.0;
    OzDeger::Sayi(val)
}

fn tip(args: Vec<OzDeger>) -> OzDeger {
    if let Some(deger) = args.get(0) {
        match deger {
            OzDeger::Sayi(_) => OzDeger::Metin("sayi".to_string()),
            OzDeger::Metin(_) => OzDeger::Metin("metin".to_string()),
            OzDeger::Dizi(_) => OzDeger::Metin("dizi".to_string()),
            OzDeger::Sozluk(_) => OzDeger::Metin("sozluk".to_string()),
            OzDeger::Islev(_) => OzDeger::Metin("islev".to_string()),
            OzDeger::Dogru | OzDeger::Yanlis => OzDeger::Metin("mantiksal".to_string()),
            OzDeger::Hic => OzDeger::Metin("hic".to_string()),
        }
    } else {
        OzDeger::Hic
    }
}
"#);
        
        self.rust_kodu.push_str("\nfn main() {\n");
        self.rust_kodu.push_str("    std::panic::set_hook(Box::new(|_info| { /* Panik mesajlarini gizle */ }));\n");
        self.rust_kodu.push_str("    let _baslangic = std::time::Instant::now();\n");
        
        for komut in &program.komutlar {
            if !matches!(komut, Komut::Islev { .. }) {
                self.komut_yaz(komut, 1)?;
            }
        }
        
        self.rust_kodu.push_str("    println!(\"Oz Suresi: {:?}\", _baslangic.elapsed().as_secs_f64());\n");
        self.rust_kodu.push_str("}\n\n");

        for komut in &program.komutlar {
            if let Komut::Islev { isim, parametreler, donus_tipi, govde } = komut {
                self.rust_kodu.push_str(&format!("fn {}(", isim));
                for (i, (p, p_tip)) in parametreler.iter().enumerate() {
                    self.rust_kodu.push_str(&format!("mut {}: OzDeger", p));
                    if i < parametreler.len() - 1 {
                        self.rust_kodu.push_str(", ");
                    }
                }
                self.rust_kodu.push_str(") -> OzDeger {\n");
                for k in govde {
                    let _ = self.komut_yaz(k, 1);
                }
                self.rust_kodu.push_str("    OzDeger::Hic\n");
                self.rust_kodu.push_str("}\n");
            }
        }
        Ok(self.rust_kodu.clone())
    }

    fn komut_yaz(&mut self, komut: &Komut, girinti: usize) -> Result<(), String> {
        let sekme = "    ".repeat(girinti);
        match komut {
            Komut::Ise { kosul, then, else_ } => {
                let kosul_str = self.ifade_oku(kosul)?;
                self.rust_kodu.push_str(&format!("{}if {}.is_truthy() {{\n", sekme, kosul_str));
                for k in then {
                    self.komut_yaz(k, girinti + 1)?;
                }
                if let Some(els) = else_ {
                    self.rust_kodu.push_str(&format!("{}}} else {{\n", sekme));
                    for k in els {
                        self.komut_yaz(k, girinti + 1)?;
                    }
                }
                self.rust_kodu.push_str(&format!("{}}}\n", sekme));
            }
            Komut::Yazdir(ifade) => {
                let d_str = self.ifade_oku(ifade)?;
                self.rust_kodu.push_str(&format!("{}println!(\"{{}}\", {});\n", sekme, d_str));
            }
            Komut::YazdirSabit(metin) => {
                self.rust_kodu.push_str(&format!("{}println!(\"{}\");\n", sekme, metin));
            }
            Komut::Dongu { kosul, govde } => {
                let kosul_str = self.ifade_oku(kosul)?;
                self.rust_kodu.push_str(&format!("{}while {}.is_truthy() {{\n", sekme, kosul_str));
                for k in govde {
                    self.komut_yaz(k, girinti + 1)?;
                }
                self.rust_kodu.push_str(&format!("{}}}\n", sekme));
            }
            Komut::Her { eleman, koleksiyon, govde } => {
                let kol_str = self.ifade_oku(koleksiyon)?;
                // Klonlama yaparak iterasyon (dizi ve sozluk destekli)
                self.rust_kodu.push_str(&format!("{}if let OzDeger::Dizi(__arr) = {} {{\n", sekme, kol_str));
                self.rust_kodu.push_str(&format!("{}    let __vec = __arr.lock().unwrap().clone();\n", sekme));
                self.rust_kodu.push_str(&format!("{}    for __item in __vec {{\n", sekme));
                self.rust_kodu.push_str(&format!("{}        let mut {} = __item;\n", sekme, eleman));
                for k in govde.iter() { self.komut_yaz(k, girinti + 2)?; }
                self.rust_kodu.push_str(&format!("{}    }}\n", sekme));
                self.rust_kodu.push_str(&format!("{}}} else if let OzDeger::Sozluk(__map) = {} {{\n", sekme, kol_str));
                self.rust_kodu.push_str(&format!("{}    let __dict = __map.lock().unwrap().clone();\n", sekme));
                self.rust_kodu.push_str(&format!("{}    for (_k, __v) in __dict {{\n", sekme));
                self.rust_kodu.push_str(&format!("{}        let mut {} = __v;\n", sekme, eleman));
                for k in govde.iter() { self.komut_yaz(k, girinti + 2)?; }
                self.rust_kodu.push_str(&format!("{}    }}\n", sekme));
                self.rust_kodu.push_str(&format!("{}}}\n", sekme));
            }
            Komut::GirdiAl(isim) => {
                self.rust_kodu.push_str(&format!("{}let mut {} = {{\n", sekme, isim));
                self.rust_kodu.push_str(&format!("{}    let mut __buf = String::new();\n", sekme));
                self.rust_kodu.push_str(&format!("{}    std::io::stdin().read_line(&mut __buf).unwrap();\n", sekme));
                self.rust_kodu.push_str(&format!("{}    OzDeger::Metin(__buf.trim().to_string())\n", sekme));
                self.rust_kodu.push_str(&format!("{}}};\n", sekme));
            }
            Komut::GirdiSayi(isim) => {
                self.rust_kodu.push_str(&format!("{}let mut {} = {{\n", sekme, isim));
                self.rust_kodu.push_str(&format!("{}    let mut __buf = String::new();\n", sekme));
                self.rust_kodu.push_str(&format!("{}    std::io::stdin().read_line(&mut __buf).unwrap();\n", sekme));
                self.rust_kodu.push_str(&format!("{}    let val: f64 = __buf.trim().parse().unwrap_or(0.0);\n", sekme));
                self.rust_kodu.push_str(&format!("{}    OzDeger::Sayi(val)\n", sekme));
                self.rust_kodu.push_str(&format!("{}}};\n", sekme));
            }
            Komut::DegiskenTanimla { isim, deger, tip: _ } => {
                if let Some(ifd) = deger {
                    let d_str = self.ifade_oku(ifd)?;
                    self.rust_kodu.push_str(&format!("{}let mut {} = {};\n", sekme, isim, d_str));
                } else {
                    self.rust_kodu.push_str(&format!("{}let mut {} = OzDeger::Hic;\n", sekme, isim));
                }
            }
            Komut::Atama { hedef, deger } => {
                if (matches!(hedef, Ifade::Cagri { .. }) || matches!(hedef, Ifade::MetotCagrisi { .. })) && matches!(deger, Ifade::Dogru) {
                    let h_str = self.ifade_oku(hedef)?;
                    self.rust_kodu.push_str(&format!("{}{};\n", sekme, h_str));
                    return Ok(());
                }
                let d_str = self.ifade_oku(deger)?;
                match hedef {
                    Ifade::Degisken(isim) => {
                        self.rust_kodu.push_str(&format!("{}{} = {};\n", sekme, isim, d_str));
                    }
                    Ifade::DiziErisim { dizi, indeks } => {
                        let dizi_str = self.ifade_oku(dizi)?;
                        let idx_str = self.ifade_oku(indeks)?;
                        self.rust_kodu.push_str(&format!("{}{}.set_idx(&{}, {});\n", sekme, dizi_str, idx_str, d_str));
                    }
                    Ifade::UyeErisim { nesne, uye } => {
                        let nesne_str = self.ifade_oku(nesne)?;
                        self.rust_kodu.push_str(&format!("{}{}.set_idx(&OzDeger::Metin(\"{}\".to_string()), {});\n", sekme, nesne_str, uye, d_str));
                    }
                    _ => return Err("Atama hedeflerinde sadece degisken, dizi ve uye erisimleri destekleniyor".into()),
                }
            }
            Komut::Don(Some(ifade)) => {
                let ifade_str = self.ifade_oku(ifade)?;
                self.rust_kodu.push_str(&format!("{}return {};\n", sekme, ifade_str));
            }
            Komut::DeneYakala { dene_govde, hata_degiskeni, yakala_govde } => {
                self.rust_kodu.push_str(&format!("{}let __dene_sonuc = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {{\n", sekme));
                for k in dene_govde {
                    self.komut_yaz(k, girinti + 1)?;
                }
                self.rust_kodu.push_str(&format!("{}}}));\n", sekme));
                
                self.rust_kodu.push_str(&format!("{}if let Err(__hata) = __dene_sonuc {{\n", sekme));
                self.rust_kodu.push_str(&format!("{}    let mut {} = OzDeger::Metin(\"Bilinmeyen Hata\".to_string());\n", sekme, hata_degiskeni));
                self.rust_kodu.push_str(&format!("{}    if let Some(oz_err) = __hata.downcast_ref::<OzDeger>() {{ {} = oz_err.clone(); }}\n", sekme, hata_degiskeni));
                self.rust_kodu.push_str(&format!("{}    else if let Some(s_err) = __hata.downcast_ref::<String>() {{ {} = OzDeger::Metin(s_err.clone()); }}\n", sekme, hata_degiskeni));
                for k in yakala_govde {
                    self.komut_yaz(k, girinti + 1)?;
                }
                self.rust_kodu.push_str(&format!("{}}}\n", sekme));
            }
            Komut::HataFirlat(ifade) => {
                let ifade_str = self.ifade_oku(ifade)?;
                self.rust_kodu.push_str(&format!("{}std::panic::panic_any({});\n", sekme, ifade_str));
            }
            Komut::Sinif { isim, govde } => {
                // Sınıf inşa edici fonksiyonu (Constructor)
                self.rust_kodu.push_str(&format!("{}fn __sinif_yeni_{}() -> OzDeger {{\n", sekme, isim));
                self.rust_kodu.push_str(&format!("{}    let mut map = std::collections::HashMap::new();\n", sekme));
                
                // Gövdedeki her Islev bir Sınıf Metodudur.
                for k in govde {
                    if let Komut::Islev { isim: m_isim, parametreler, donus_tipi: _, govde: m_govde } = k {
                        self.rust_kodu.push_str(&format!("{}    map.insert(\"{}\".to_string(), OzDeger::Islev(std::sync::Arc::new(|__args| {{\n", sekme, m_isim));
                        
                        // Argümanları Rust lokal değişkenlerine ata (Sınıf metotlarında ilk arg kendisi/this olur)
                        for (i, (p_ad, _)) in parametreler.iter().enumerate() {
                            self.rust_kodu.push_str(&format!("{}        let mut {} = __args.get({}).cloned().unwrap_or(OzDeger::Hic);\n", sekme, p_ad, i));
                        }
                        
                        // Metot Gövdesi
                        for m_komut in m_govde {
                            self.komut_yaz(m_komut, girinti + 2)?;
                        }
                        
                        // Geri dönüş yoksa Hic dön
                        self.rust_kodu.push_str(&format!("{}        OzDeger::Hic\n", sekme));
                        self.rust_kodu.push_str(&format!("{}    }})));\n", sekme));
                    } else if let Komut::DegiskenTanimla { isim: p_isim, deger, .. } = k {
                        // Sınıf özelliği (Property)
                        let d_str = match deger {
                            Some(d) => self.ifade_oku(d)?,
                            None => "OzDeger::Hic".to_string(),
                        };
                        self.rust_kodu.push_str(&format!("{}    map.insert(\"{}\".to_string(), {});\n", sekme, p_isim, d_str));
                    }
                }
                
                self.rust_kodu.push_str(&format!("{}    OzDeger::Sozluk(std::sync::Arc::new(std::sync::Mutex::new(map)))\n", sekme));
                self.rust_kodu.push_str(&format!("{}}}\n", sekme));
            }
            _ => {} // Digerlerini simdilik atla
        }
        Ok(())
    }

    fn ifade_oku(&self, ifade: &Ifade) -> Result<String, String> {
        match ifade {
            Ifade::Sayi(s) => {
                if s.fract() == 0.0 {
                    Ok(format!("OzDeger::Sayi({}.0)", s))
                } else {
                    Ok(format!("OzDeger::Sayi({})", s))
                }
            }
            Ifade::Dogru => Ok("OzDeger::Dogru".into()),
            Ifade::Yanlis => Ok("OzDeger::Yanlis".into()),
            Ifade::Metin(s) => Ok(format!("OzDeger::Metin(\"{}\".to_string())", s)),
            Ifade::Dizi(elemanlar) => {
                let mut vals = Vec::new();
                for e in elemanlar {
                    vals.push(self.ifade_oku(e)?);
                }
                Ok(format!("OzDeger::Dizi(Arc::new(Mutex::new(vec![{}])))", vals.join(", ")))
            }
            Ifade::Sozluk(ciftler) => {
                let mut eklemeler = Vec::new();
                for (anahtar, deger) in ciftler {
                    let deger_str = self.ifade_oku(deger)?;
                    eklemeler.push(format!("map.insert(\"{}\".to_string(), {});", anahtar, deger_str));
                }
                Ok(format!("{{ let mut map = HashMap::new(); {} OzDeger::Sozluk(Arc::new(Mutex::new(map))) }}", eklemeler.join(" ")))
            }
            Ifade::DiziErisim { dizi, indeks } => {
                let dizi_str = self.ifade_oku(dizi)?;
                let idx_str = self.ifade_oku(indeks)?;
                Ok(format!("{}.get_idx(&{})", dizi_str, idx_str))
            }
            Ifade::UyeErisim { nesne, uye } => {
                let nesne_str = self.ifade_oku(nesne)?;
                // Uye erisimi sozluk icin metin indeksi gibi davranir (orn. kisi.isim == kisi["isim"])
                Ok(format!("{}.get_idx(&OzDeger::Metin(\"{}\".to_string()))", nesne_str, uye))
            }
            Ifade::Degisken(isim) => Ok(format!("{}.clone()", isim)),
            Ifade::Ikili { sol, op, sag } => {
                let sol_str = self.ifade_oku(sol)?;
                let sag_str = self.ifade_oku(sag)?;
                match op {
                    Op::Topla => Ok(format!("({} + {})", sol_str, sag_str)),
                    Op::Cikar => Ok(format!("({} - {})", sol_str, sag_str)),
                    Op::Carp => Ok(format!("({} * {})", sol_str, sag_str)),
                    Op::Bol => Ok(format!("({} / {})", sol_str, sag_str)),
                    Op::Esit => Ok(format!("if {} == {} {{ OzDeger::Dogru }} else {{ OzDeger::Yanlis }}", sol_str, sag_str)),
                    Op::Kucuk => {
                        // Kucuk, Buyuk vs icin simdilik pattern match ile kiyaslama
                        Ok(format!("if let (OzDeger::Sayi(a), OzDeger::Sayi(b)) = ({}, {}) {{ if a < b {{ OzDeger::Dogru }} else {{ OzDeger::Yanlis }} }} else {{ OzDeger::Yanlis }}", sol_str, sag_str))
                    }
                    Op::Buyuk => {
                        Ok(format!("if let (OzDeger::Sayi(a), OzDeger::Sayi(b)) = ({}, {}) {{ if a > b {{ OzDeger::Dogru }} else {{ OzDeger::Yanlis }} }} else {{ OzDeger::Yanlis }}", sol_str, sag_str))
                    }
                    _ => Err("Desteklenmeyen Op (Henuz)".into()),
                }
            }
            Ifade::Cagri { isim, argumanlar } => {
                if isim == "yazdir" {
                    let mut args = Vec::new();
                    for a in argumanlar {
                        args.push(self.ifade_oku(a)?);
                    }
                    Ok(format!("println!(\"{{}}\", {})", args.join(" + ")))
                } else if isim == "dosya_oku" || isim == "dosya_yaz" || isim == "rastgele" || isim == "tip" {
                    let mut args = Vec::new();
                    for a in argumanlar {
                        args.push(self.ifade_oku(a)?);
                    }
                    Ok(format!("{}(vec![{}])", isim, args.join(", ")))
                } else {
                    let mut args = Vec::new();
                    for a in argumanlar {
                        args.push(self.ifade_oku(a)?);
                    }
                    // Fonksiyonları çağırırken snake_case'den bağımsız isimler kullanılabilir
                    Ok(format!("{}({})", isim, args.join(", ")))
                }
            }
            Ifade::MetotCagrisi { nesne, metot, argumanlar } => {
                let nesne_str = self.ifade_oku(nesne)?;
                let mut args = Vec::new();
                for a in argumanlar {
                    args.push(self.ifade_oku(a)?);
                }
                Ok(format!("{}.cagir_metot(\"{}\", vec![{}])", nesne_str, metot, args.join(", ")))
            }
            Ifade::YeniNesne(sinif_adi) => {
                Ok(format!("__sinif_yeni_{}()", sinif_adi))
            }
            _ => Err(format!("Desteklenmeyen Ifade: {:?}", ifade)),
        }
    }
}
