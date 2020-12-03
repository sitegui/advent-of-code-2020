use std::collections::BTreeSet;
use std::fs;
use std::time::Instant;

const TARGET: i32 = 2020;

pub fn solve() {
    let values: BTreeSet<i32> = fs::read_to_string("data/input-1")
        .unwrap()
        .split("\n")
        .filter_map(|line| line.parse().ok())
        .collect();

    // Part 1
    let start = Instant::now();
    if let Some((smaller_term, larger_term)) = find_pair(&values, 0, TARGET) {
        println!(
            "Answer = {} * {} = {} (in {:?})",
            smaller_term,
            larger_term,
            smaller_term * larger_term,
            start.elapsed()
        );
    }

    // Part 2
    let start = Instant::now();
    for &smaller_term in values.iter() {
        if let Some((medium_term, larger_term)) =
            find_pair(&values, smaller_term, TARGET - smaller_term)
        {
            println!(
                "Answer = {} * {} * {} = {} (in {:?})",
                smaller_term,
                medium_term,
                larger_term,
                smaller_term * medium_term * larger_term,
                start.elapsed()
            );
            break;
        }
    }
}

/// If possible, return `a` and `b`, two elements of `values` that respect:
/// 1. `a + b = target`
/// 2. `min_value <= a <= b`
fn find_pair(values: &BTreeSet<i32>, min_value: i32, target: i32) -> Option<(i32, i32)> {
    for &medium_term in values.range(min_value..) {
        let larger_term = target - medium_term;
        if medium_term > larger_term {
            break;
        }

        if values.contains(&larger_term) {
            return Some((medium_term, larger_term));
        }
    }

    None
}
