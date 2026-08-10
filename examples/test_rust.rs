use std::time::Instant;

fn main() {
    let start = Instant::now();

    let mut i: i32 = 0;
    while i < 10000000 {
        // Rust'ta volatile işlemi için ptr::read/write veya core::hint::black_box kullanılır.
        // Optimizasyonu bozmamak ama loopu silmesini engellemek için:
        std::hint::black_box(&mut i);
        i = i + 1;
    }
    std::hint::black_box(i);

    let elapsed = start.elapsed().as_secs_f64();
    println!("Rust Suresi: {:.9}", elapsed);
}
