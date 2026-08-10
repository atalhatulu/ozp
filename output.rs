#[allow(warnings)]
fn main() {
    let baslangic = std::time::Instant::now();
    iterasyon_testi();
    println!("Sure (saniye): {:?}", baslangic.elapsed().as_secs_f64());
}

fn iterasyon_testi() -> f64 {
    let mut idx = 0.0;
    while idx < 10000000.0 {
        idx = idx + 1.0;
    }
    return 0.0;
}
