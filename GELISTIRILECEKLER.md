# OZ+ Dili Geliştirme Yol Haritası (GELISTIRILECEKLER)

Bu dosya, OZ+ programlama dilinin gelecekteki geliştirme adımlarını takip etmek için oluşturulmuştur. Yeni bir yapay zeka asistanı (veya geliştirici) projeye dahil olduğunda, kaldığı yerden devam edebilmesi için aşağıdaki görevleri okumalıdır.

## 📌 Öncelikli Görevler (Aşama 1)

### 1. Kütüphane Sistemi (`dahil_et`)
- **Durum:** Sözlük (lexer) olarak eklendi, ancak derleyici mantığı (parser/main.c) henüz yazılmadı.
- **Görev:** `dahil_et "kutuphane.ozp"` yazıldığında, derleyicinin bu dosyanın içeriğini okuyup ana kodun içine gömmesi (Tıpkı C'deki `#include` gibi) sağlanmalı. Bu sayede modüler programlamaya geçilecek.

### 2. Gelişmiş Tip Çıkarımı (Advanced Type Inference)
- **Durum:** `a = 5` (Sayı) veya `b = "Metin"` (Metin) otomatik algılanıyor. Ancak `sonuc = eger a > 5 : "dogru" , "yanlis"` gibi Ternary (Tek satır şart) operatörlerinde eşittirden sonra `eger` kelimesi geldiği için derleyici doğrudan `long` tahmini yapıyor.
- **Görev:** `parser.c` içindeki tip tahmin algoritması iyileştirilmeli. Değişken atamalarında eşitliğin ilerisindeki kelimeler taranarak dönüş tipi (char*, double, long) daha akıllıca belirlenmeli.

## 📦 Standart Kütüphaneler (Aşama 2)
Çekirdek büyütülmeden, `dahil_et` özelliği kullanılarak OZ+ dilinin kendi standart kütüphaneleri (`.ozp` uzantılı) yazılmalıdır. C dilinin güçlü kütüphaneleri sarmalanarak (wrapper) oluşturulmalıdır:

- [ ] **matematik.ozp:** `karekok()`, `kuvvet_al()`, `yuvarla()` işlevleri. (Arka planda C'nin `math.h` fonksiyonlarını çağıracak).
- [ ] **zaman.ozp:** O anki saati alma, programı bekletme (`uyu()` / `sleep()`) işlevleri.
- [ ] **dosya.ozp:** Metin dosyası okuma (`dosya_oku()`) ve dosyaya yazma (`dosyaya_yaz()`) işlevleri.
- [ ] **ag.ozp (Network):** Basit HTTP istekleri atabilmek için ağ kütüphanesi.

## 🚀 Çekirdek (Core) Sözdizimi Geliştirmeleri (Aşama 3)

### 1. Akıllı Döngüler (`her` ve `icinde`)
- **Durum:** Şu an sadece sonsuz `dongu` (while) var.
- **Görev:** Python'daki `for x in liste:` yapısını getirmek için `her x icinde liste:` sözdizimi derleyiciye öğretilmeli. Dizilerin (Array) C tarafında Generic boyutlu hale getirilmesi gerekiyor.

### 2. Gelişmiş Hata Yakalama (Try-Catch)
- **Durum:** `dene`, `hata_yakala`, `hata_firlat` kelimeleri tanımlı ancak C backend'inde %100 güvenli çalışmıyor.
- **Görev:** C dilindeki `setjmp.h` kullanılarak veya C++ derleyicisine geçilerek try-catch bloklarının hatasız çalışması sağlanmalı.

### 3. Otomatik Bellek Yönetimi (Garbage Collection)
- **Durum:** `yeni` komutuyla oluşturulan nesneler iş bitiminde C tarafında manuel `free()` edilmek zorunda kalınabilir veya bellek sızıntısı (memory leak) yaşanabilir.
- **Görev:** Referans sayımı (Reference Counting) tabanlı basit bir çöp toplayıcı (GC) yazılmalı.

---

**Not:** OZ+ dilinin vizyonu "Python kadar okunaklı, C kadar hızlı ve Türkçe" bir dil yaratmaktır. Her yeni özellik eklenirken C derleyicisinin hızından ödün verilmemesine ve `oz-konsol` (REPL) uygulamasının bozulmamasına dikkat edilmelidir.
