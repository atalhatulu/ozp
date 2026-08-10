fn main() {
    let mut i: i64 = 0;
    let sinir: i64 = 2000000;
    let mut toplam: i64 = 0;
    while i < sinir {
        toplam = toplam + i;
        i = i + 1;
    }
    println!("Bitti");
}
