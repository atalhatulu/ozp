# OZ+ Programlama Dili

**OZ+**, Türkçe sözdizimine sahip, C diline dönüştürülerek (transpile) derlenen modern ve hafif bir programlama dilidir.

## Proje Yapısı

Yeni ve modüler mimarisiyle proje şu şekilde organize edilmiştir:

- `src/` : Derleyicinin kaynak kodlarını barındırır.
  - `lexer.c/h`: Kaynak kodu kelimelere (token) ayırır.
  - `parser.c/h`: Token'ları okur ve geçerli bir C koduna (`gecici_oz_cikti.c`) dönüştürür.
  - `utils.c/h`: Dosya okuma gibi yardımcı fonksiyonlar.
  - `main.c`: Ana program akışını yönetir.
  - `include/`: OZ+ dili standart kütüphanelerini (`matematik.h`, `oz_standart.h`, `oz_plus.h`) barındırır.
- `examples/` : OZ+ ile yazılmış örnek (`.ozp` uzantılı) kodlar yer alır.
- `Makefile` : Projeyi derlemek için kullanılır.

## Nasıl Kurulur ve Çalıştırılır?

Derleyiciyi (ozc) kendi bilgisayarınızda oluşturmak için terminalinizde proje dizinindeyken `make` komutunu çalıştırın:

```bash
make
```

Bu işlem sonucunda `ozc` adında bir derleyici program oluşacaktır.

## Kod Derleme

Oluşan `ozc` derleyicisini kullanarak `.ozp` dosyalarınızı derleyebilirsiniz:

```bash
./ozc examples/asal.ozp
```

Bu işlem sonucunda `program` adında çalıştırılabilir bir dosya oluşacaktır. Programı başlatmak için:

```bash
./program
```

## Temizleme (Clean)

Geçici dosyaları ve oluşturulan çalıştırılabilir dosyaları silmek için:

```bash
make clean
```

## Dilin Özellikleri (Örnek Sözdizimi)

```ozp
degisken n = 2
dongu n < 100000:
    degisken asal = 1
    degisken i = 2
    
    degisken sinir = n / 2
    dongu i <= sinir:
        degisken kalan = n % i
        ise kalan == 0:
            asal = 0
            i = n
        son
        i = i + 1
    son

    ise asal == 1:
        yazdir n
    son
    n = n + 1
son
```
