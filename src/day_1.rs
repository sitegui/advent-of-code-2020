use std::collections::BTreeSet;
use std::fs;

const TARGET: usize = 2020;

pub fn solve() -> (usize, usize) {
    let values: BTreeSet<usize> = fs::read_to_string("data/input-1")
        .unwrap()
        .split("\n")
        .filter_map(|line| line.parse().ok())
        .collect();

    // Part 1
    let (smaller_term, larger_term) = find_pair(&values, 0, TARGET).unwrap();
    let part_1 = smaller_term * larger_term;

    // Part 2
    for &smaller_term in values.iter() {
        if let Some((medium_term, larger_term)) =
            find_pair(&values, smaller_term, TARGET - smaller_term)
        {
            let part_2 = smaller_term * medium_term * larger_term;
            return (part_1, part_2);
        }
    }

    unreachable!()
}

/// If possible, return `a` and `b`, two elements of `values` that respect:
/// 1. `a + b = target`
/// 2. `min_value <= a <= b`
fn find_pair(values: &BTreeSet<usize>, min_value: usize, target: usize) -> Option<(usize, usize)> {
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
