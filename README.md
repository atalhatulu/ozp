# Oz Dili (Oz+) - Beta Sürümü 🚀

**Oz+**, Türkçe sözdizimine sahip, okunabilirliği Python kadar kolay ancak arkasındaki AOT (Ahead of Time) Transpiler mimarisi sayesinde **Native Rust hızında** çalışan dinamik tipli, açık kaynaklı bir programlama dilidir.

## 🌟 Neden Oz+ ?

- **Türkçe Sözdizimi:** `ise`, `degilse`, `dongu`, `sinif`, `her` gibi tamamen Türkçe, kolay anlaşılır anahtar kelimeler.
- **Performans Canavarı:** Yavaş Sanal Makine (VM) veya Yorumlayıcı (Interpreter) mimarisi yerine AOT (Ahead of Time) Derleyici kullanır. Yazdığınız kod saniyeler içinde Native makine diline (Rust üzerinden) çevrilerek çalıştırılır. C++ ve Rust ile aynı kulvarda koşar.
- **Nesne Yönelimli (OOP):** Sınıflar, metotlar ve nesne oluşturma özellikleri tam desteklenir.
- **Dinamik Tip Sistemi:** Tip belirlemeye gerek kalmadan esnek kodlama yapabilirsiniz.

---

## 🛠️ Kurulum ve Kullanım

Oz+ dilini derlemek ve çalıştırmak için bilgisayarınızda **Rust (cargo)** yüklü olmalıdır.

```bash
# Repo'yu indirin
git clone https://github.com/teha/ozp.git
cd ozp

# Oz kodunuzu calistirin
cargo run --bin ozc --release -- examples/bubble_sort.ozp
```

---

## 📖 Dilin Temelleri

### 1. Değişkenler ve Yazdırma
```ozp
degisken isim = "Dunya"
degisken sayi = 42
yazdir "Merhaba " + isim
```

### 2. Şart Blokları
```ozp
degisken yas = 18
ise yas > 17:
    yazdir "Yetiskin"
degilse:
    yazdir "Cocuk"
son
```

### 3. Döngüler
**While Döngüsü:**
```ozp
degisken i = 0
dongu i < 5:
    yazdir i
    i = i + 1
son
```
**İleri Seviye For Döngüsü (Her - İçinde):**
```ozp
her sayi icinde [1, 2, 3]:
    yazdir sayi
son
```

### 4. Fonksiyonlar (İşlevler)
```ozp
islev topla(a, b):
    don a + b
son

degisken sonuc = topla(5, 10)
yazdir sonuc
```

### 5. Sınıflar ve Nesneler (OOP)
```ozp
sinif Araba:
    degisken marka = "Belirsiz"
    
    islev calistir(kendisi):
        yazdir kendisi.marka + " calisti!"
    son
son

degisken bmw = yeni Araba()
bmw.marka = "BMW"
bmw.calistir()
```

### 6. Modül Sistemi (Dahil Et)
Projelerinizi parçalara bölebilir ve Native Hızında birleştirebilirsiniz.
```ozp
dahil_et "matematik.ozp"
yazdir topla(3, 5)
```

---

## 📚 Standart Kütüphane (Gömülü Özellikler)
Oz+, dilin çekirdeğine gömülü güçlü metotlar sunar.

**Dosya İşlemleri (I/O)**
- `dosya_oku("veri.txt")`
- `dosya_yaz("veri.txt", "icerik")`

**Metin (String) İşlemleri**
- `metin.buyut()`
- `metin.kucult()`
- `metin.parcala(" ")`
- `metin.iceriyor_mu("kelime")`

**Dizi (Liste) İşlemleri**
- `liste.ekle(veri)`
- `liste.al(indeks)`
- `liste.degistir(indeks, yeni_veri)`
- `liste.uzunluk()`

**Diğer Araçlar**
- `rastgele()`: 0-1 arası rastgele sayı üretir.
- `tip(degisken)`: Değişkenin tipini verir.

---

## 🏗️ Mimari ve Nasıl Çalışıyor?

Oz+ projesi geleneksel yöntemleri reddeder.
1. **Lexer:** Kodunuzu Türkçe Token'lara ayırır.
2. **Parser:** Token'lardan soyut bir İfade Ağacı (AST) oluşturur. Gerekli modülleri (`dahil_et`) okuyup tek bir dev ağaç yaratır.
3. **Transpiler (Codegen):** Oluşturulan AST'yi, memory-safe (bellek güvenli) Rust koduna çevirir (`Arc<Mutex>` kilitleriyle).
4. **Binary (rustc):** Çevrilen Rust kodunu işletim sisteminize özel Makine Diline derler ve milisaniyeler içinde başlatır.

## 🚧 Eksikler (V1.0 Prodüksiyon İçin Hedefler)
Şu an **Beta** sürümündeyiz. Tam prodüksiyon sürümü için hedefler:
- [ ] Gelişmiş Hata Takibi (Satır numarası bildiren Stack Trace)
- [ ] Dairesel referanslar (Circular references) için Garbage Collector mimarisi
- [ ] Ağ (HTTP, TCP) paketleri

---
*Geliştirici: teha & Antigravity (Google)*
