use std::time::Instant;

fn fib(n: f64) -> f64 {
    if n < 2.0 {
        return n;
    }
    fib(n - 1.0) + fib(n - 2.0)
}

fn main() {
    let start = Instant::now();

    let result = fib(35.0);
    println!("{}", result);

    let elapsed = start.elapsed().as_secs_f64();
    println!("Rust Suresi: {:.6}", elapsed);
}
