use star::quanot::Quanot;
use std::time::Instant;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let reservoir_size: usize = args.get(1).map(|s| s.parse().unwrap_or(1000)).unwrap_or(1000);
    let n_iterations: usize = args.get(2).map(|s| s.parse().unwrap_or(10000)).unwrap_or(10000);

    println!("Quanot Benchmark");
    println!("================");
    println!("Reservoir size: {}", reservoir_size);
    println!("Iterations: {}", n_iterations);
    println!();

    let mut quanot = Quanot::new(128, reservoir_size);

    println!("Warming up...");
    for i in 0..100 {
        quanot.process(&format!("warmup {}", i));
    }
    quanot.reset();

    println!("Running benchmark...");
    let start = Instant::now();

    for i in 0..n_iterations {
        quanot.process(&format!("benchmark input {}", i));
    }

    let elapsed = start.elapsed();
    let elapsed_secs = elapsed.as_secs_f64();
    let latency_ms = (elapsed_secs * 1000.0) / (n_iterations as f64);
    let latency_us = latency_ms * 1000.0;
    let msgs_per_sec = (n_iterations as f64) / elapsed_secs;

    println!();
    println!("Results:");
    println!("  Total time: {:.3}s", elapsed_secs);
    println!("  Latency: {:.3}ms ({:.2}us)", latency_ms, latency_us);
    println!("  Throughput: {:.0} msg/s", msgs_per_sec);

    let result = quanot.process("final");
    println!();
    println!("Sample output:");
    println!("  consciousness_proxy: {:.6}", result.consciousness_proxy);
    println!("  novelty: {:.6}", result.novelty);
    println!("  reservoir_state len: {}", result.reservoir_state.len());
}