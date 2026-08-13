// OZ+ parser — token akışını AST'ya çevirir.
// legacy/src/parser.c davranışı temel alındı ama "doğrudan fprintf" yerine
// önce gerçek bir Program(AST) üretir. codegen; C'yi buradan basar.

use crate::ast::{Ifade, Komut, Op, Program, SembolTablosu, Tip};
use crate::tokens::{Token, TokenDizisi, TokenTip};

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    pub semboller: SembolTablosu,
    in_islev_decl: bool,
    in_sinif_block: bool,
    sinif_adi: String,
}

impl Parser {
    pub fn yeni(tokens: &TokenDizisi) -> Self {
        Self {
            tokens: tokens.tokens.clone(),
            pos: 0,
            semboller: SembolTablosu::default(),
            in_islev_decl: false,
            in_sinif_block: false,
            sinif_adi: String::new(),
        }
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.pos.min(self.tokens.len() - 1)]
    }

    #[allow(dead_code)]
    fn peek_n(&self, n: usize) -> Option<&Token> {
        self.tokens.get(self.pos + n)
    }

    fn ilerle(&mut self) -> Token {
        let t = self.tokens[self.pos.min(self.tokens.len() - 1)].clone();
        if self.pos < self.tokens.len() - 1 {
            self.pos += 1;
        }
        t
    }

    fn atla(&mut self) {
        if self.pos < self.tokens.len() - 1 {
            self.pos += 1;
        }
    }

    fn anahtar(&self, k: &str) -> bool {
        self.peek().anahtar_mi(k)
    }

    fn id_deger(&self) -> Option<String> {
        if self.peek().tip == TokenTip::ID {
            Some(self.peek().deger.clone())
        } else {
            None
        }
    }

    // --- İfade ayrıştırma ---
    // OZ+ oldukça serbest bir sözdizimine sahip; en yaygın durumlar:
    //   sayı, metin, değişken, ikili işlem (satır sonuna kadar birleşik)

    fn ifade_parse(&mut self) -> Ifade {
        // Operatör önceliğine saygılı ifade ayrıştırma (precedence climbing).
        //   seviye 3: * / %
        //   seviye 2: + -
        //   seviye 1: < > <= >= == !=
        //   seviye 0: ve veya
        // `2 + 3 * 4` → `2 + (3 * 4)` üretilir (eski sürüm soldan-sağa
        // bağlıyordu: `(2 + 3) * 4`).
        self.binary_parse(0)
    }

    fn binary_parse(&mut self, min_oncesi: u8) -> Ifade {
        let mut lhs = self.terim_parse();

        loop {
            let (oncesi, op) = match self.op_gorsin() {
                Some(t) => t,
                None => break,
            };
            if oncesi < min_oncesi {
                break;
            }
            self.atla();
            let sag = self.binary_parse(oncesi + 1);
            lhs = Ifade::Ikili { sol: Box::new(lhs), op, sag: Box::new(sag) };
        }

        lhs
    }

    // Geçerli token bir ikili operatörse öncelik seviyesi + operatör döndürür.
    fn op_gorsin(&self) -> Option<(u8, Op)> {
        let t = self.peek();
        Some(match t.tip {
            TokenTip::CARP | TokenTip::BOL | TokenTip::MOD => (3, match t.tip {
                TokenTip::CARP => Op::Carp,
                TokenTip::BOL => Op::Bol,
                _ => Op::Mod,
            }),
            TokenTip::ARTI | TokenTip::EKS => (2, match t.tip {
                TokenTip::ARTI => Op::Topla,
                _ => Op::Cikar,
            }),
            TokenTip::KUCUK => (1, Op::Kucuk),
            TokenTip::BUYUK => (1, Op::Buyuk),
            TokenTip::KUCUK_ESIT => (1, Op::KucukEsit),
            TokenTip::BUYUK_ESIT => (1, Op::BuyukEsit),
            TokenTip::ESIT_ESIT => (1, Op::Esit),
            TokenTip::ESIT_DEGIL => (1, Op::EsitDegil),
            TokenTip::VE => (0, Op::Ve),
            TokenTip::VEYA => (0, Op::Veya),
            _ => return None,
        })
    }

    fn terim_parse(&mut self) -> Ifade {
        let t = self.peek();
        match t.tip {
            TokenTip::SAYI => {
                let deger = self.ilerle().deger;
                Ifade::Sayi(deger.parse().unwrap_or(0.0))
            }
            TokenTip::METIN => {
                let deger = self.ilerle().deger;
                Ifade::Metin(deger)
            }
            TokenTip::ANAHTAR if t.deger == "Dogru" => {
                self.ilerle();
                Ifade::Dogru
            }
            TokenTip::ANAHTAR if t.deger == "Yanlis" => {
                self.ilerle();
                Ifade::Yanlis
            }
            TokenTip::ANAHTAR if t.deger == "hic" => {
                self.ilerle();
                Ifade::Hic
            }
            TokenTip::ANAHTAR if t.deger == "yeni" => {
                self.ilerle();
                let sinif_adi = self.id_deger().unwrap_or_default();
                if !sinif_adi.is_empty() {
                    self.atla();
                }
                if self.peek().tip == TokenTip::PARANTEZ_AC {
                    self.atla();
                    if self.peek().tip == TokenTip::PARANTEZ_KAPA {
                        self.atla();
                    }
                }
                Ifade::YeniNesne(sinif_adi)
            }
            TokenTip::PARANTEZ_AC => {
                // İç içe ifade: (5 + 3)
                self.atla(); // (
                let ic = self.ifade_parse();
                if self.peek().tip == TokenTip::PARANTEZ_KAPA {
                    self.atla();
                }
                ic
            }
            TokenTip::KOSELI_AC => {
                // Dizi Oluşturma: [1, 2, 3]
                self.atla();
                let mut elemanlar = Vec::new();
                if self.peek().tip != TokenTip::KOSELI_KAPA {
                    elemanlar.push(self.ifade_parse());
                    while self.peek().tip == TokenTip::VIRGUL {
                        self.atla();
                        elemanlar.push(self.ifade_parse());
                    }
                }
                while self.peek().tip != TokenTip::KOSELI_KAPA && self.peek().tip != TokenTip::EOF {
                    self.atla();
                }
                if self.peek().tip == TokenTip::KOSELI_KAPA {
                    self.atla();
                }
                Ifade::Dizi(elemanlar)
            }
            TokenTip::SUSLU_AC => {
                // Sözlük Oluşturma: {"anahtar": "deger"}
                self.atla();
                let mut elemanlar = Vec::new();
                if self.peek().tip != TokenTip::SUSLU_KAPA {
                    loop {
                        if self.peek().tip == TokenTip::METIN {
                            let anahtar = self.ilerle().deger;
                            if self.peek().tip == TokenTip::IKINOKTA {
                                self.atla();
                                let deger = self.ifade_parse();
                                elemanlar.push((anahtar, deger));
                            }
                        }
                        if self.peek().tip == TokenTip::VIRGUL {
                            self.atla();
                        } else {
                            break;
                        }
                    }
                }
                while self.peek().tip != TokenTip::SUSLU_KAPA && self.peek().tip != TokenTip::EOF {
                    self.atla();
                }
                if self.peek().tip == TokenTip::SUSLU_KAPA {
                    self.atla();
                }
                Ifade::Sozluk(elemanlar)
            }
            TokenTip::ID => {
                let isim = self.ilerle().deger;
                let mut lhs = Ifade::Degisken(isim);

                loop {
                    if self.peek().tip == TokenTip::PARANTEZ_AC {
                        self.atla();
                        let mut argumanlar = Vec::new();
                        if self.peek().tip != TokenTip::PARANTEZ_KAPA {
                            argumanlar.push(self.ifade_parse());
                            while self.peek().tip == TokenTip::VIRGUL {
                                self.atla();
                                argumanlar.push(self.ifade_parse());
                            }
                        }
                        while self.peek().tip != TokenTip::PARANTEZ_KAPA && self.peek().tip != TokenTip::EOF {
                            self.atla();
                        }
                        if self.peek().tip == TokenTip::PARANTEZ_KAPA {
                            self.atla();
                        }

                        if let Ifade::UyeErisim { nesne, uye } = lhs {
                            lhs = Ifade::MetotCagrisi { nesne, metot: uye, argumanlar };
                        } else if let Ifade::Degisken(isim) = lhs {
                            lhs = Ifade::Cagri { isim, argumanlar };
                        } else {
                            lhs = Ifade::Cagri { isim: "Bilinmeyen".to_string(), argumanlar };
                        }
                    } else if self.peek().tip == TokenTip::KOSELI_AC {
                        self.atla();
                        let indeks = self.ifade_parse();
                        if self.peek().tip == TokenTip::KOSELI_KAPA {
                            self.atla();
                        }
                        lhs = Ifade::DiziErisim { dizi: Box::new(lhs), indeks: Box::new(indeks) };
                    } else if self.peek().tip == TokenTip::NOKTA {
                        self.atla();
                        let uye = self.id_deger().unwrap_or_default();
                        if !uye.is_empty() {
                            self.atla();
                        }
                        lhs = Ifade::UyeErisim { nesne: Box::new(lhs), uye };
                    } else {
                        break;
                    }
                }
                lhs
            }
            _ => {
                // Bilinmeyen — boş değer ile devam et
                let _ = self.ilerle();
                Ifade::Sayi(0.0)
            }
        }
    }

    // --- Komut ayrıştırma ---
    fn komut_parse(&mut self, program: &mut Program) -> Komut {
        let t = self.peek().clone();

        if t.tip == TokenTip::EOF {
            return Komut::Bos;
        }

        // Eski C parser'ın anahtar kelime işleyişi, AST'ya taşınıyor
        if t.tip == TokenTip::ANAHTAR {
            match t.deger.as_str() {
                "degisken" => {
                    self.ilerle(); // degisken
                    let isim = self.id_deger().unwrap_or_default();
                    if !isim.is_empty() {
                        self.atla();
                        self.semboller.tanimla(&isim, Tip::TamSayi);
                    }
                    // Opsiyonel başlangıç değeri: degisken n = 2
                    if self.peek().anahtar_mi("esit") || self.peek().tip == TokenTip::ESIT
                        || self.peek().tip == TokenTip::ESIT_ESIT
                    {
                        // '=' bekleniyor
                    }
                    if self.peek().tip == TokenTip::ESIT {
                        self.atla();
                        let deger = self.ifade_parse();
                        return Komut::DegiskenTanimla { isim, tip: Tip::TamSayi, deger: Some(deger) };
                    }
                    Komut::DegiskenTanimla { isim, tip: Tip::TamSayi, deger: None }
                }
                "metin" => {
                    self.ilerle();
                    let isim = self.id_deger().unwrap_or_default();
                    if !isim.is_empty() {
                        self.atla();
                        self.semboller.tanimla(&isim, Tip::Metin);
                    }
                    if self.peek().tip == TokenTip::ESIT {
                        self.atla();
                        let deger = self.ifade_parse();
                        return Komut::DegiskenTanimla { isim, tip: Tip::Metin, deger: Some(deger) };
                    }
                    Komut::DegiskenTanimla { isim, tip: Tip::Metin, deger: None }
                }
                "ondalik" => {
                    self.ilerle();
                    let isim = self.id_deger().unwrap_or_default();
                    if !isim.is_empty() {
                        self.atla();
                        self.semboller.tanimla(&isim, Tip::Ondalik);
                    }
                    if self.peek().tip == TokenTip::ESIT {
                        self.atla();
                        let deger = self.ifade_parse();
                        return Komut::DegiskenTanimla { isim, tip: Tip::Ondalik, deger: Some(deger) };
                    }
                    Komut::DegiskenTanimla { isim, tip: Tip::Ondalik, deger: None }
                }
                "dizi" => {
                    self.ilerle();
                    let isim = self.id_deger().unwrap_or_default();
                    if !isim.is_empty() {
                        self.atla();
                        self.semboller.tanimla(&isim, Tip::Dizi);
                    }
                    Komut::DegiskenTanimla { isim, tip: Tip::Dizi, deger: None }
                }
                "sozluk" => {
                    self.ilerle();
                    let isim = self.id_deger().unwrap_or_default();
                    if !isim.is_empty() {
                        self.atla();
                        self.semboller.tanimla(&isim, Tip::Sozluk);
                    }
                    Komut::DegiskenTanimla { isim, tip: Tip::Sozluk, deger: None }
                }
                "dongu" => {
                    self.ilerle(); // dongu
                    let kosul = self.ifade_parse();
                    // ':' atla
                    if self.peek().tip == TokenTip::IKINOKTA {
                        self.atla();
                    }
                    self.semboller.scope_artir();
                    let govde = self.govde_parse(program);
                    Komut::Dongu { kosul, govde }
                }
                "ise" => {
                    self.ilerle();
                    let kosul = self.ifade_parse();
                    if self.peek().tip == TokenTip::IKINOKTA {
                        self.atla();
                    }
                    self.semboller.scope_artir();
                    let then = self.govde_parse(program);

                    // degilse
                    let else_ = if self.anahtar("degilse") {
                        self.ilerle();
                        if self.anahtar("ise") {
                            // degilse ise — C sürümünde dokunulmamıştı
                            self.atla();
                            self.semboller.scope_artir();
                            let g = self.govde_parse(program);
                            Some(g)
                        } else {
                            if self.peek().tip == TokenTip::IKINOKTA {
                                self.atla();
                            }
                            self.semboller.scope_artir();
                            Some(self.govde_parse(program))
                        }
                    } else {
                        None
                    };

                    Komut::Ise { kosul, then, else_ }
                }
                "son" => {
                    // govde_parse tarafından yakalanır; buraya gelirse atla
                    self.ilerle();
                    Komut::Bos
                }
                "yazdir" => {
                    self.ilerle();
                    let ifade = self.ifade_parse();
                    Komut::Yazdir(ifade)
                }
                "zaman_baslat" => {
                    self.ilerle();
                    Komut::ZamanBaslat
                }
                "zaman_bitir" => {
                    self.ilerle();
                    Komut::ZamanBitir
                }
                "don" => {
                    self.ilerle();
                    if self.peek().tip != TokenTip::IKINOKTA
                        && self.peek().tip != TokenTip::EOF
                        && self.peek().satir == t.satir
                    {
                        let ifade = self.ifade_parse();
                        Komut::Don(Some(ifade))
                    } else {
                        Komut::Don(None)
                    }
                }
                "kir" => {
                    self.ilerle();
                    Komut::Kir
                }
                "devam_et" => {
                    self.ilerle();
                    Komut::DevamEt
                }
                "her" => {
                    self.ilerle(); // her
                    let eleman = self.id_deger().unwrap_or_default();
                    if !eleman.is_empty() { self.atla(); }
                    
                    if self.peek().deger == "icinde" {
                        self.atla();
                    }
                    
                    let koleksiyon = self.ifade_parse();
                    
                    if self.peek().tip == TokenTip::IKINOKTA {
                        self.atla();
                    }
                    
                    let mut govde = Vec::new();
                    self.semboller.scope_artir();
                    while self.peek().tip != TokenTip::EOF && !self.peek().anahtar_mi("son") {
                        govde.push(self.komut_parse(program));
                    }
                    if self.peek().anahtar_mi("son") {
                        self.atla();
                    }
                    // TODO: eger scope_azalt lazimsa eklenecek, simdilik yok.
                    
                    Komut::Her { eleman, koleksiyon, govde }
                }
                "girdi" => {
                    self.ilerle();
                    let isim = self.id_deger().unwrap_or_default();
                    if !isim.is_empty() {
                        self.atla();
                    }
                    // girdi degisken al  → string okuma
                    if self.peek().anahtar_mi("al") {
                        self.ilerle();
                        self.semboller.tanimla(&isim, Tip::Metin);
                        return Komut::GirdiAl(isim);
                    }
                    // girdi degisken → sayısal okuma
                    self.semboller.tanimla(&isim, Tip::TamSayi);
                    Komut::GirdiSayi(isim)
                }

                "sinif" => {
                    self.ilerle();
                    let isim = self.id_deger().unwrap_or_default();
                    if !isim.is_empty() {
                        self.atla();
                    }
                    self.sinif_adi = isim.clone();
                    self.in_sinif_block = true;
                    if self.peek().tip == TokenTip::IKINOKTA {
                        self.atla();
                    }
                    self.semboller.scope_artir();
                    let govde = self.govde_parse(program);
                    self.in_sinif_block = false;
                    program
                        .sinif_haritasi
                        .entry(isim.clone())
                        .or_default()
                        .extend(govde.clone());
                    Komut::Sinif { isim, govde }
                }
                "dene" => {
                    self.ilerle(); // dene
                    if self.peek().tip == TokenTip::IKINOKTA { self.atla(); }
                    
                    let mut dene_govde = Vec::new();
                    while self.peek().tip != TokenTip::EOF && !self.peek().anahtar_mi("yakala") {
                        dene_govde.push(self.komut_parse(program));
                    }

                    if self.peek().anahtar_mi("yakala") {
                        self.atla(); // yakala
                    }
                    if self.peek().anahtar_mi("hata") {
                        self.atla(); // hata
                    }
                    
                    let mut hata_degiskeni = self.id_deger().unwrap_or_default();
                    if !hata_degiskeni.is_empty() {
                        self.atla();
                    } else {
                        hata_degiskeni = "hata".to_string(); // Varsayilan
                    }
                    if self.peek().tip == TokenTip::IKINOKTA { self.atla(); }

                    let mut yakala_govde = Vec::new();
                    while self.peek().tip != TokenTip::EOF && !self.peek().anahtar_mi("son") {
                        yakala_govde.push(self.komut_parse(program));
                    }
                    
                    if self.peek().anahtar_mi("son") {
                        self.atla(); // son
                    }

                    Komut::DeneYakala { dene_govde, hata_degiskeni, yakala_govde }
                }
                "hata_firlat" => {
                    self.ilerle();
                    let ifade = self.ifade_parse();
                    Komut::HataFirlat(ifade)
                }
                "eger" => {
                    self.ilerle(); // eger
                    let _kosul = self.ifade_parse();
                    // eger kosul, dogru : yanlis  → üçlü operatör
                    // (şimdilik Bos; C sürümünde parse-eda in_eger bayrağı kullanılıyordu)
                    Komut::Eger
                }
                "sabit" => {
                    self.ilerle();
                    let isim = self.id_deger().unwrap_or_default();
                    if !isim.is_empty() {
                        self.atla();
                    }
                    if self.peek().tip == TokenTip::ESIT {
                        self.atla();
                        let deger = self.ifade_parse();
                        return Komut::Sabit { isim, deger };
                    }
                    Komut::Sabit {
                        isim,
                        deger: Ifade::Sayi(0.0),
                    }
                }
                "islev" => {
                    self.ilerle();
                    let isim = self.id_deger().unwrap_or_default();
                    if !isim.is_empty() {
                        self.atla();
                    }
                    self.in_islev_decl = true;
                    let mut parametreler = Vec::new();
                    if self.peek().tip == TokenTip::PARANTEZ_AC {
                        self.atla();
                        while self.peek().tip != TokenTip::PARANTEZ_KAPA
                            && self.peek().tip != TokenTip::EOF
                        {
                            if let Some(p) = self.id_deger() {
                                self.atla(); // p'yi atla
                                let mut tip = None;
                                if self.peek().tip == TokenTip::IKINOKTA {
                                    self.atla();
                                    if let Some(t) = self.id_deger() {
                                        tip = Some(t);
                                        self.atla();
                                    }
                                }
                                parametreler.push((p, tip));
                            } else {
                                self.atla();
                            }
                        }
                        if self.peek().tip == TokenTip::PARANTEZ_KAPA {
                            self.atla();
                        }
                    }
                    let mut donus_tipi = None;
                    if self.peek().tip == TokenTip::OK {
                        self.atla();
                        if let Some(t) = self.id_deger() {
                            donus_tipi = Some(t);
                            self.atla();
                        }
                    }
                    if self.peek().tip == TokenTip::IKINOKTA {
                        self.atla();
                    }
                    self.semboller.scope_artir();
                    let govde = self.govde_parse(program);
                    self.in_islev_decl = false;
                    Komut::Islev { isim, parametreler, donus_tipi, govde }
                }
                "dahil_et" => {
                    self.ilerle();
                    let dosya = if self.peek().tip == TokenTip::METIN {
                        self.peek().deger.clone()
                    } else if self.peek().tip == TokenTip::ID {
                        self.peek().deger.clone()
                    } else {
                        "".to_string()
                    };
                    if !dosya.is_empty() {
                        self.atla();
                    }
                    Komut::DahilEt(dosya)
                }
                _ => {
                    // Bilinmeyen anahtar — ifade gibi dene, değilse atla
                    self.ilerle();
                    Komut::Bos
                }
            }
        } else if t.tip == TokenTip::ID {
            let hedef = self.ifade_parse(); // Zaten ilerleyip ifadeyi çekecek!

            // Atama: hedef = deger
            if self.peek().tip == TokenTip::ESIT || self.peek().anahtar_mi("esit") {
                self.atla();
                let deger = self.ifade_parse();

                if let Ifade::Degisken(isim) = &hedef {
                    self.semboller.tanimla(isim, Tip::TamSayi);
                }
                return Komut::Atama { hedef, deger };
            }

            // Atama değilse bu bir serbest ifadedir (cagri vs)
            if matches!(hedef, Ifade::Cagri { .. } | Ifade::MetotCagrisi { .. }) {
                return Komut::Atama {
                    hedef,
                    deger: Ifade::Dogru,
                };
            }

            Komut::Bos
        } else {
            self.ilerle();
            Komut::Bos
        }
    }

    // Bir blok gövdesini 'son' anahtar kelimesine kadar ayrıştırır
    fn govde_parse(&mut self, program: &mut Program) -> Vec<Komut> {
        let mut komutlar = Vec::new();
        loop {
            let t = self.peek().clone();
            if t.tip == TokenTip::EOF {
                break;
            }
            if t.anahtar_mi("son") {
                self.ilerle(); // son
                self.semboller.scope_dusur();
                break;
            }
            // degilse — ise bloğu içinde else başlangıcı; bırak üstteki işlesin
            if t.anahtar_mi("degilse") {
                break;
            }
            // degilse_ise
            if t.anahtar_mi("degilse_ise") {
                self.ilerle();
                if self.anahtar("ise") {
                    self.ilerle();
                }
                self.semboller.scope_artir();
                komutlar.extend(self.govde_parse(program));
                continue;
            }

            let k = self.komut_parse(program);
            if !matches!(k, Komut::Bos) {
                komutlar.push(k);
            }
        }
        komutlar
    }

    pub fn parse_program(mut self) -> Program {
        let mut program = Program::default();
        loop {
            let t = self.peek();
            if t.tip == TokenTip::EOF {
                break;
            }
            if t.anahtar_mi("son") || t.anahtar_mi("degilse") {
                // Saçmalayan/eksik blokları sessizce atla
                self.atla();
                continue;
            }
            let k = self.komut_parse(&mut program);
            if !matches!(k, Komut::Bos) {
                program.komutlar.push(k);
            } else {
                self.atla(); // sonsuz döngü koruması
            }
        }
        program
    }
}

pub fn parse(tokens: &TokenDizisi) -> Result<Program, CompileHata> {
    let parser = Parser::yeni(tokens);
    Ok(parser.parse_program())
}

#[derive(Debug)]
pub struct CompileHata {
    pub satir: u32,
    pub mesaj: String,
}

impl std::fmt::Display for CompileHata {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[OZ+ DERLEME HATASI] Satir {}: {}", self.satir, self.mesaj)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer;

    fn parse_dosya(kod: &str) -> Program {
        let tokens = lexer::tokenize(kod);
        parse(&tokens).unwrap()
    }

    #[test]
    fn kod_basi_degisken() {
        let p = parse_dosya("degisken n = 2");
        assert_eq!(p.komutlar.len(), 1);
        match &p.komutlar[0] {
            Komut::DegiskenTanimla { isim, deger, .. } => {
                assert_eq!(isim, "n");
                assert!(deger.is_some());
            }
            _ => panic!("beklenen: degisken tanimi"),
        }
    }

    #[test]
    fn dongu_scopes() {
        let p = parse_dosya(
            "degisken n = 0\ndongu n < 5:\n    degisken k = 1\n    n = n + 1\nson\n",
        );
        assert_eq!(p.komutlar.len(), 2);
        match &p.komutlar[1] {
            Komut::Dongu { govde, .. } => assert_eq!(govde.len(), 2),
            _ => panic!("beklenen: dongu"),
        }
    }

    #[test]
    fn ise_degilse() {
        let p = parse_dosya(
            "ise n == 1:\n    yazdir \"bir\"\ndegilse:\n    yazdir \"diger\"\nson\n",
        );
        assert_eq!(p.komutlar.len(), 1);
        match &p.komutlar[0] {
            Komut::Ise { else_, then, .. } => {
                assert!(else_.is_some());
                assert_eq!(then.len(), 1);
            }
            _ => panic!("beklenen: ise"),
        }
    }
}