#![allow(clippy::comparison_chain)]

use crate::data::{Data, ParseBytes};
use itertools::Itertools;

const PREAMBLE: usize = 25;

pub fn solve() -> (usize, usize) {
    let data: Vec<u64> = Data::read(9)
        .lines()
        .map(|line| line.parse_bytes())
        .collect();

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

    (part_1 as usize, part_2.unwrap() as usize)
}

fn is_valid(previous: &[u64], value: u64) -> bool {
    for (a, b) in previous.iter().tuple_combinations() {
        if a + b == value {
            return true;
        }
    }

    false
}

fn contiguous_sum(values: &[u64], target: u64) -> Option<(u64, u64)> {
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
