use crate::data::Data;
use crate::iter_utils::IterUtils;
use std::collections::BTreeSet;

const TARGET: i64 = 2020;

pub fn solve() -> (i64, i64) {
    let values: BTreeSet<i64> = Data::read(1).lines().parsed().collect();

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
fn find_pair(values: &BTreeSet<i64>, min_value: i64, target: i64) -> Option<(i64, i64)> {
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
