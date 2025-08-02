// Simple benchmark - in real projects you'd use criterion or similar
use std::time::Instant;

fn fibonacci(n: u64) -> u64 {
    match n {
        0 => 1,
        1 => 1,
        n => fibonacci(n - 1) + fibonacci(n - 2),
    }
}

fn main() {
    let start = Instant::now();
    let result = fibonacci(20);
    let duration = start.elapsed();
    println!("fibonacci(20) = {} (took: {:?})", result, duration);
}
