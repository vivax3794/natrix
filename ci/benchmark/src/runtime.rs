#![warn(missing_docs)]
use std::hint::black_box;

#[derive(Default)]
pub struct Bencher {
    results: Vec<String>,
}

impl Bencher {
    pub fn new() -> Self {
        Self {
            results: Vec::new(),
        }
    }

    pub async fn bench<A: Clone, R>(&mut self, name: &str, arg: A, func: impl Fn(A) -> R) {
        let performance = web_sys::window().unwrap().performance().unwrap();

        let start = performance.now();
        let _ = func(black_box(arg.clone()));
        natrix::async_utils::next_animation_frame().await;
        natrix::async_utils::next_animation_frame().await;
        let mut end = performance.now();

        if start == end {
            end += 1.0;
        }
        let target_iterations = 5000.0_f64.div_euclid(end - start) as u128;

        let mut total_rust = 0.0;
        let mut total_ms = 0.0;
        for _ in 0..target_iterations {
            let start = performance.now();
            let _ = func(black_box(arg.clone()));
            let end_rust = performance.now();
            natrix::async_utils::next_animation_frame().await;
            natrix::async_utils::next_animation_frame().await;
            let end = performance.now();

            total_ms += end - start;
            total_rust += end_rust - start;
        }

        let average = total_ms / target_iterations as f64;
        let average_rust = total_rust / target_iterations as f64;

        self.results.push(
            format!(
                "{name} (iters: {target_iterations}): {average_rust:.4}ms (with reflow/repaint: {average}ms)"
            ),
        );
    }

    pub fn done(self) {
        let results = self.results.join("\n");
        panic!("---NATRIX_BENCHMARK_START\n{results}\n---NATRIX_BENCHMARK_END");
    }
}
