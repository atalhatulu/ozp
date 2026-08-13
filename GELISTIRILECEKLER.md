# OZ+ (ozp) Derleyici — Geliştirme Yol Haritası

Bu dosya, **Rust tabanlı `ozc` derleyicisinin** (Lexer → Parser/AST → Codegen → rustc) gelecekteki geliştirme adımlarını takip eder.

> ⚠️ Geçmişte burada duran içerik, eski **C transpiler'ının** (`ozplus-lang`, `src/*.c`, `oz`, `oz-konsol`) yol haritasıydı ve yanlışlıkla bu repoya kopyalanmıştı. O içerik artık **yalnızca `legacy/` klasöründe**, ait olduğu C projesiyle birlikte durmaktadır.

Mimari:

```text
.ozp → Lexer → Parser (AST) → Codegen (Rust) → rustc -O → native executable
```

## 🟢 Tamamlanmış

- [x] Lexer (Türkçe keyword: `islev`, `degisken`, `dongu`, `eger`, `sinif`, `her`, `icinde` …; string/sayı/yorum/id/operator/array-object/`->`/karşılaştırma)
- [x] Parser → gerçek `Program(AST)` üretimi
- [x] Operatör önceliği (precedence climbing: `2 + 3 * 4` → `2 + (3*4)`)
- [x] Tüm ikili operatörlerin codegen'i (`+ - * / % == != < > <= >= ve veya`)
- [x] `dahil_et` modül/flattening sistemi (HashSet ile tekrar yükleme önlemi)
- [x] `sinif` / `yeni` / metot çağrısı (OOP, `OzDeger` map tabanlı)
- [x] `dene` / `hata_yakala` / `hata_firlat` (try-catch, panic tabanlı)
- [x] Diziler, sözlükler, `her ... icinde` döngüsü
- [x] AOT native derleme (`rustc -O`), `--tokens` / `--ast` debug bayrakları
- [x] Sembol tablosu (scope yığını)
- [x] Unit testler (6)

## 🟡 Kısmen tamamlanmış / Bilinen eksikler

- **Hata yönetimi (parser):** `ifade_parse` bilinmeyen token'da `Ifade::Sayi(0.0)` döndürüyor → `degisken x = !!!` sessizce `0` olarak derleniyor. `Result<Ifade, CompileHata>` / gerçek hata mesajı + satır numarası hedeflenmeli.
- **Type system:** `degisken` her zaman `Tip::TamSayi` olarak kaydediliyor; tip çıkarımı yok. `degisken isim = "Teha"` → `String` çıkarımı hedeflenmeli. Tipler `{TamSayi, Metin, Ondalik, Dizi, Sozluk}` ile sınırlı.
- **`eger` (ternary) ifadesi:** parser'da `Komut::Eger` üretiyor ama codegen'de `Bos` (C sürümünden kalan yarım parça). Implementasyon bekliyor.
- **`zaman` fonksiyonu:** `zaman_baslat` / `zaman_bitir` codegen'i `zaman(...)` çağırıyor ama bu fonksiyon üretilmiyor → `zaman` kullanan örnekler (`dongu_lokal`, `dongu_test`, `fibonacci`, `bilgi_yarismasi` …) rustc derleme hatası veriyor.
- **`dizi` / `sozluk` statik tip:** `dizi sayilar = [1,2,3]` → `Array<Int>` / `Array<String>` jenerik tip çıkarımı yapılmıyor.
- **`test_dahil_et.ozp`:** `dahil_et "matematik.ozp"` yolu ana dizine göre değil `examples/examples/...` olarak çözülüyor.

## 🔴 Gelecek (v0.2 → v1.0)

ChatGPT proje incelemesinden sonra stratejik yol:

1. **Feature freeze:** Yeni keyword / syntax eklemeden önce temeli sağlamlaştır.
2. **Semantic Analyzer + Type System:** AST sonrası semantik doğrulama katmanı; tip çıkarımı; `Array<T>`, `Sozluk<K,V>`, `Function`, `Class`.
3. **Gerçek hata raporlama:** Satır/sütun bazlı, panik yerine `Result` propagasyonu.
4. **IR katmanı (opsiyonel/uzun vade):** AST bağımlılığını backend'den ayır:

```text
          ┌→ Rust (rustc)
AST → HIR ┼→ LLVM / Cranelift
          ├→ WASM
          └→ Interpreter
```

> Şimdilik `OZ+ → Rust → rustc` kalıyor; `rustc` kullanmak yanlış değil. Ama AST/semantic seviyesi `rustc`'ye bağımlı olmamalı ki bir gün kendi backend'ine geçilebilsin.

5. **Gerçek module system:** `dahil_et`'in dosya birleştirme (textual flattening) olmaktan çıkıp `module / import / export / private / public / namespace` düzenine evrilmesi.
6. **LSP tam desteği:** Mevcut VS Code uzantısı (legacy C için) Rust `ozc`'ye uyarlanmalı.

---

*Geliştirici: teha & Antigravity (Google)*
