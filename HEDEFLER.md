# OZ+ (ozp) — Geliştirme Hedefleri

**ozp**, Türkçe sözdizimine sahip, AOT (Ahead-of-Time) transpiler ile **Rust üzerinden native hızda** çalışan bir programlama dili derleyicisidir.

> ⚠️ Eski `HEDEFLER.md`, C tabanlı `ozplus-lang` projesine aitti (20 maddelik yol haritası). O içerik `legacy/` klasörüne taşınmıştır. Bu dosya, mevcut **Rust derleyicisi `ozc`** için güncel hedefleri içerir.

## ✅ Tamamlanmış (Beta)

- Lexer: Türkçe keyword + token sistemi
- Parser: Gerçek `Program(AST)` üretimi
- Operatör önceliği (precedence) ve tüm ikili operatörlerin codegen'i
- `degisken`, `metin`, `ondalik`, `dizi`, `sozluk`, `sabit`
- `ise` / `degilse`, `dongu`, `her ... icinde`
- `islev` + parametre tipi + `don`
- `sinif` / `yeni` / metot / özellik (OOP)
- `dene` / `hata_yakala` / `hata_firlat`
- Dizi / sözlük işlemleri, `dahil_et` modül sistemi
- Builtin stdlib: `dosya_oku`, `dosya_yaz`, `rastgele`, `tip`, metin/dizi metotları
- AOT native derleme (`rustc -O`), `--tokens` / `--ast` debug bayrakları
- Sembol tablosu + scope yönetimi
- `zaman()` builtin fonksiyonu + `zaman_baslat` / `zaman_bitir` zamanlayıcı komutları

## 🎯 Sıradaki Hedefler

### Katman 1 — Derleyici temelini sağlamlaştır (v0.2, öncelikli)
- [ ] **Gerçek hata yönetimi:** Parser'da `Sayi(0.0)` fallback kaldırılmalı; `Result<Ifade, CompileHata>` ile satır odaklı hata mesajları. `degisken x = !!!` → derlenme hatası vermeli.
- [ ] **Tip çıkarımı:** `degisken isim = "Teha"` → `Metin`, `degisken x = 42` → `TamSayi`. Tipler: `Int, Float, String, Bool, Array<T>, Sozluk<K,V>, Function, Class, Object, Void`.
- [ ] **Jenerik dizi/sözlük:** `dizi sayilar = [1,2,3]` → `Array<Int>`.
- [ ] **`eger` (ternary)** ifadesinin codegen'i tamamlanmalı.
- [ ] **Eski C stdlib örnekleri (pre-existing):** `bilgi_yarismasi`, `cop_toplayici`, `sinif_testi`, `test_kovan`, `hesap_makinesi`, `sozluk_ve_hata`, `rehber_uygulamasi` — `sozluk_olustur` / `sozluk_ekle` / `_oz_son_hata_mesaji` gibi **eski `ozplus-lang` C stdlib API'sine** dayanıyor; Rust transpiler'da bu fonksiyonlar yok → `degisken/fonksiyon bulunamadı` (E0425). Bu örneklerin Rust `OzDeger` API'sine taşınması gerekiyor.

### Katman 2 — Mimari (v0.3)
- [ ] Semantic Analyzer katmanı (AST sonrası doğrulama)
- [ ] AST'yi backend'den ayıran HIR katmanı
- [ ] Gerçek module system (`import` / `export` / `private` / `public`)
- [ ] `dahil_et` yol çözümleme düzeltmesi (göreli yol)

### Katman 3 — Ekosistem
- [ ] Rust `ozc` için güncel VS Code uzantısı (mevcut uzantı legacy C içindi)
- [ ] LSP sunucusu
- [ ] Paket yöneticisi

## Test / Doğrulama Notu

Parser'da 6 unit test vardır. Operatör değişiklikleri sonrası tüm `examples/*.ozp` regresyon koşusu önerilir:

```bash
cargo test --release --bin ozc
```

---

*Geliştirici: teha & Antigravity (Google)*
