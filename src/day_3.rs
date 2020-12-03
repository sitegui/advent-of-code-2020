use itertools::Itertools;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub fn solve() -> (usize, usize) {
    let map = BufReader::new(File::open("data/input-3").unwrap())
        .lines()
        .map(|line| line.unwrap().chars().collect_vec())
        .collect_vec();

    let trees_1_1 = trees_in_path(&map, 1, 1);
    let trees_3_1 = trees_in_path(&map, 3, 1);
    let trees_5_1 = trees_in_path(&map, 5, 1);
    let trees_7_1 = trees_in_path(&map, 7, 1);
    let trees_1_2 = trees_in_path(&map, 1, 2);

    let part_2 = trees_1_1 * trees_3_1 * trees_5_1 * trees_7_1 * trees_1_2;
    (trees_3_1, part_2)
}

/// Return the number of trees encountered following a path with the given slope from the top left
/// of the map.
fn trees_in_path(map: &[Vec<char>], slope_right: usize, slope_down: usize) -> usize {
    let mut pos_x = 0;
    let mut num_trees = 0;
    for row in map.iter().step_by(slope_down) {
        if row[pos_x % row.len()] == '#' {
            num_trees += 1;
        }
        pos_x += slope_right;
    }
    num_trees
}
