#![allow(clippy::naive_bytecount)]

use std::env::args;
use std::fmt;
use std::time::{Duration, Instant};

mod data;
mod iter_utils;
mod parser;

const NUM_WARMING: usize = 10;
const NUM_SAMPLES: usize = 10;

struct Day {
    label: &'static str,
    solve_fn: fn() -> (i64, i64),
    expected: (i64, i64),
}

#[derive(Clone)]
struct BenchResult {
    label: &'static str,
    samples: Vec<Duration>,
    mean: Duration,
    std: Duration,
}

impl Day {
    fn solve(&self) -> (i64, i64) {
        (self.solve_fn)()
    }

    fn bench_solve(&self) -> BenchResult {
        for _ in 0..NUM_WARMING {
            self.assert_solve();
        }

        let mut samples = Vec::with_capacity(10);
        for _ in 0..NUM_SAMPLES {
            let start = Instant::now();
            self.assert_solve();
            samples.push(start.elapsed());
        }

        BenchResult::from_samples(self.label, samples)
    }

    fn assert_solve(&self) {
        let answer = self.solve();
        assert_eq!(answer, self.expected, "when checking {}", self.label);
    }
}

impl BenchResult {
    fn from_samples(label: &'static str, samples: Vec<Duration>) -> Self {
        let mean = samples.iter().sum::<Duration>() / samples.len() as u32;
        let std = (samples
            .iter()
            .map(|&s| (s.as_secs_f64() - mean.as_secs_f64()).powi(2))
            .sum::<f64>()
            / (samples.len() as f64 - 1.))
            .sqrt();

        BenchResult {
            label,
            samples,
            mean,
            std: Duration::from_secs_f64(std),
        }
    }
}

macro_rules! days {
    ($($day:ident = ($part_1:expr, $part_2:expr)),* $(,)?) => {
        $(mod $day;)*

        const DAYS: &[Day] = &[
            $(Day {
                label: stringify!($day),
                solve_fn: $day::solve,
                expected: ($part_1, $part_2)
            }),*
        ];
    };
}

days! {
    day_1 = (788739, 178724430),
    day_2 = (424, 747),
    day_3 = (205, 3952146825),
    day_4 = (254, 184),
    day_5 = (861, 633),
    day_6 = (6170, 2947),
    day_7 = (197, 85324),
    day_8 = (1753, 733),
    day_9 = (248131121, 31580383),
    day_10 = (1980, 4628074479616),
    day_11 = (2183, 1990),
    day_12 = (1294, 20592),
    day_13 = (174, 780601154795940),
}

fn main() {
    match args().nth(1) {
        None => {
            println!("Will execute all days to time their individual and total execution times");

            let mut results = Vec::with_capacity(DAYS.len());
            for day in DAYS {
                let result = day.bench_solve();
                println!("{}", result);
                results.push(result);
            }

            let combined: Vec<Duration> = (0..NUM_SAMPLES)
                .map(|i| results.iter().map(|result| result.samples[i]).sum())
                .collect();

            let overall = BenchResult::from_samples("overall", combined);
            println!("{}", overall);
        }
        Some(day) => {
            let day: usize = day.parse().unwrap();

            let start = Instant::now();
            let (part_1, part_2) = DAYS[day - 1].solve();
            println!(
                "Part 1 = {}, part 2 = {} in {:?}",
                part_1,
                part_2,
                start.elapsed()
            );
        }
    }
}

impl fmt::Display for BenchResult {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} in {:.2} ± {:.2} ms",
            self.label,
            self.mean.as_secs_f64() * 1e3,
            self.std.as_secs_f64() * 1e3
        )
    }
}
