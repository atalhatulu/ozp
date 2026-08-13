// OZ+ AOT Derleyicisi (Rust tabanli) 
// Mimari: Lexer -> Parser (AST) -> Codegen (Rust) -> rustc (Binary)

mod ast;
mod lexer;
mod parser;
mod tokens;
mod codegen;
mod semantic;

use std::fs;
use std::io::{self, Write};

const YARDIM: &str = "Kullanım: ozc [dosya.ozp]
  -h, --help    Bu yardımı göster
  --tokens      Sadece token listesini yazdır (debug)
  --ast         Sadece AST'ı yazdır (debug)
  --check       Sadece semantic analiz yap, derleme yapma (debug)";

/// Semantic analizi çalıştırır. Hata varsa listeler ve false döner.
fn semantic_kontrol(program: &ast::Program) -> bool {
    let hatalar = semantic::Semantik::yeni().analiz_et(program);
    if hatalar.is_empty() {
        println!("Semantik analiz: sorun yok.");
        return true;
    }
    eprintln!("[OZ+ SEMANTIK HATASI] {} sorun bulundu:", hatalar.len());
    for h in &hatalar {
        eprintln!("  - {}", h.mesaj);
    }
    false
}

fn dosya_oku(yol: &str) -> Result<String, String> {
    fs::read_to_string(yol).map_err(|e| format!("Dosya okunamadı '{}': {}", yol, e))
}

fn token_dok(yol: &str) -> Result<(), String> {
    let kaynak = dosya_oku(yol)?;
    let dizi = lexer::tokenize(&kaynak);
    for t in &dizi.tokens {
        println!("{:?}: {:?}", t.tip, t.deger);
    }
    Ok(())
}

fn ast_dok(yol: &str) -> Result<(), String> {
    let kaynak = dosya_oku(yol)?;
    let dizi = lexer::tokenize(&kaynak);
    let program = parser::parse(&dizi).map_err(|e| e.to_string())?;
    println!("{:#?}", program.komutlar);
    Ok(())
}

fn check_dok(yol: &str) -> Result<(), String> {
    let kaynak = dosya_oku(yol)?;
    let dizi = lexer::tokenize(&kaynak);
    let mut program = parser::parse(&dizi).map_err(|e| e.to_string())?;
    // dahil_et dosyalarını çözümle (modül fonksiyonları görünsün)
    let ana_dizin = std::path::Path::new(yol)
        .parent()
        .unwrap_or(std::path::Path::new(""));
    let mut yuklenenler = std::collections::HashSet::new();
    cozumle_dahil_et(&mut program.komutlar, ana_dizin, &mut yuklenenler)?;
    if semantic_kontrol(&program) {
        Ok(())
    } else {
        Err("Semantik analizde hatalar var; derleme adımı atlandı.".into())
    }
}

fn cozumle_dahil_et(komutlar: &mut Vec<ast::Komut>, ana_dizin: &std::path::Path, yuklenenler: &mut std::collections::HashSet<String>) -> Result<(), String> {
    let mut yeni_komutlar = Vec::new();
    let mut eklenecekler = Vec::new();

    for komut in komutlar.drain(..) {
        if let ast::Komut::DahilEt(ref dosya) = komut {
            // Cift tirnaklari temizle
            let mut temiz_dosya = dosya.replace("\"", "");
            if !temiz_dosya.ends_with(".ozp") {
                temiz_dosya.push_str(".ozp");
            }
            
            let tam_yol = ana_dizin.join(&temiz_dosya);
            let yol_str = tam_yol.to_string_lossy().to_string();
            
            if !yuklenenler.contains(&yol_str) {
                yuklenenler.insert(yol_str.clone());
                
                let kaynak = fs::read_to_string(&tam_yol)
                    .map_err(|e| format!("Dahil edilecek dosya bulunamadi ({}): {}", yol_str, e))?;
                    
                let dizi = lexer::tokenize(&kaynak);
                let mut alt_program = parser::parse(&dizi).map_err(|e| format!("{} icinde hata: {}", yol_str, e))?;
                
                // Recursive cozumleme
                let yeni_dizin = tam_yol.parent().unwrap_or(std::path::Path::new(""));
                cozumle_dahil_et(&mut alt_program.komutlar, yeni_dizin, yuklenenler)?;
                
                // Alt programin komutlarini eklenecekler listesine koy
                eklenecekler.extend(alt_program.komutlar);
            }
        } else {
            yeni_komutlar.push(komut);
        }
    }
    
    // Eklenecekleri basa, asil komutlari sona koy (once bagimliliklar tanimlansin)
    let mut nihai = Vec::new();
    nihai.extend(eklenecekler);
    nihai.extend(yeni_komutlar);
    
    *komutlar = nihai;
    Ok(())
}

fn derle_ve_calistir(yol: &str) -> Result<(), String> {
    let kaynak = dosya_oku(yol)?;

    println!("\n[1/4] '{}' okunuyor...", yol);
    let dizi = lexer::tokenize(&kaynak);

    println!("[2/4] İfade Ağacı (AST) üretiliyor...");
    let mut program = parser::parse(&dizi).map_err(|e| e.to_string())?;
    
    let ana_dizin = std::path::Path::new(yol).parent().unwrap_or(std::path::Path::new(""));
    let mut yuklenenler = std::collections::HashSet::new();
    cozumle_dahil_et(&mut program.komutlar, ana_dizin, &mut yuklenenler)?;

    println!("[2.5/4] Semantik Analiz ediliyor...");
    let sem_hatalar = semantic::Semantik::yeni().analiz_et(&program);
    if !sem_hatalar.is_empty() {
        eprintln!("[OZ+ SEMANTIK HATASI] {} sorun bulundu:", sem_hatalar.len());
        for h in &sem_hatalar {
            eprintln!("  - {}", h.mesaj);
        }
        return Err(format!("Semantik analiz {} hata ile bitti; derleme iptal edildi.", sem_hatalar.len()));
    }

    println!("[3/4] Hedef Makine Diline (AOT) Çevriliyor...");
    let mut t = codegen::Transpiler::yeni();
    let rust_kod = t.transpile(&program)?;
    fs::write("build_ozp.rs", rust_kod).map_err(|e| e.to_string())?;

    println!("[4/4] Sistem Derleyicisi (rustc) Tetikleniyor ve Yürütülüyor...\n");
    let status = std::process::Command::new("rustc")
        .arg("-O")
        .arg("build_ozp.rs")
        .arg("-o")
        .arg("ozp_run")
        .status()
        .map_err(|e| format!("Derleme hatası (Arka planda rustc kurulu olmalı): {}", e))?;
        
    if status.success() {
        let run_status = std::process::Command::new("./ozp_run")
            .status()
            .map_err(|e| format!("Çalıştırma hatası: {}", e))?;
        
        if !run_status.success() {
            println!("Program hata ile kapandı.");
        }
        
        // Temizlik (isteğe bağlı)
        let _ = fs::remove_file("build_ozp.rs");
        let _ = fs::remove_file("ozp_run");
        Ok(())
    } else {
        Err("Sistem derleyicisi kodu makine diline çevirirken hata aldı.".into())
    }
}

fn interaktif_menu() {
    let mut dosyalar = Vec::new();
    if let Ok(entries) = fs::read_dir("examples") {
        for entry in entries.filter_map(Result::ok) {
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("ozp") {
                if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
                    dosyalar.push(name.to_string());
                }
            }
        }
    }

    if dosyalar.is_empty() {
        println!("examples/ klasorunde hic .ozp dosyasi bulunamadi!");
        return;
    }

    println!("\n=== OZ+ (AOT) INTERAKTIF MENU ===");
    println!("Calistirmak (Derlemek) istediginiz dosyayi secin:");
    for (i, dosya) in dosyalar.iter().enumerate() {
        println!("{}) {}", i + 1, dosya);
    }
    println!("0) Cikis");
    print!("Seciminiz: ");
    io::stdout().flush().unwrap();

    let mut secim_str = String::new();
    if io::stdin().read_line(&mut secim_str).is_err() {
        println!("Gecersiz giris.");
        return;
    }

    let secim: usize = match secim_str.trim().parse() {
        Ok(n) => n,
        Err(_) => {
            println!("Gecersiz giris.");
            return;
        }
    };

    if secim == 0 {
        println!("Cikis yapiliyor...");
        return;
    } else if secim > 0 && secim <= dosyalar.len() {
        let tam_yol = format!("examples/{}", dosyalar[secim - 1]);
        if let Err(e) = derle_ve_calistir(&tam_yol) {
            eprintln!("{}", e);
        }
    } else {
        println!("Gecersiz secim!");
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        interaktif_menu();
        return;
    }

    let arg = &args[1];
    if arg == "-h" || arg == "--help" {
        println!("{}", YARDIM);
        return;
    }

    let sonuc = match args.get(1).map(String::as_str) {
        Some("--tokens") => args.get(2).map(|s| token_dok(s)).unwrap_or(Err("Dosya yolu gerekli".into())),
        Some("--ast") => args.get(2).map(|s| ast_dok(s)).unwrap_or(Err("Dosya yolu gerekli".into())),
        Some("--check") => args.get(2).map(|s| check_dok(s)).unwrap_or(Err("Dosya yolu gerekli".into())),
        _ => derle_ve_calistir(arg),
    };

    if let Err(e) = sonuc {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}