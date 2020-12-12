#![allow(clippy::comparison_chain)]

use crate::data::Data;
use crate::iter_utils::IterUtils;
use itertools::Itertools;

const PREAMBLE: usize = 25;

pub fn solve() -> (i64, i64) {
    let data: Vec<i64> = Data::read(9).lines().parsed().collect();

    let mut part_1 = None;
    for (index, &value) in data.iter().enumerate().skip(PREAMBLE) {
        if !is_valid(&data[index - PREAMBLE..index], value) {
            part_1 = Some(value);
            break;
        }
    }
    let part_1 = part_1.unwrap();

    let mut part_2 = None;
    for start_index in 0..data.len() {
        if let Some((min, max)) = contiguous_sum(&data[start_index..], part_1) {
            part_2 = Some(max + min);
            break;
        }
    }

    (part_1, part_2.unwrap())
}

fn is_valid(previous: &[i64], value: i64) -> bool {
    for (a, b) in previous.iter().tuple_combinations() {
        if a + b == value {
            return true;
        }
    }

    false
}

fn contiguous_sum(values: &[i64], target: i64) -> Option<(i64, i64)> {
    let mut sum = values[0];

    for (index, &value) in values.iter().enumerate().skip(1) {
        sum += value;
        if sum == target {
            return values[0..=index].iter().copied().minmax().into_option();
        } else if sum > target {
            break;
        }
    }

    None
}
