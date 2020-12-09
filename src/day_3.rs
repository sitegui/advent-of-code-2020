use crate::data::Data;
use itertools::Itertools;

pub fn solve() -> (i64, i64) {
    let data = Data::read(3);
    let map = data.lines().collect_vec();

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
fn trees_in_path(map: &[&[u8]], slope_right: usize, slope_down: usize) -> i64 {
    let mut pos_x = 0;
    let mut num_trees = 0;
    for row in map.iter().step_by(slope_down) {
        if row[pos_x % row.len()] == b'#' {
            num_trees += 1;
        }
        pos_x += slope_right;
    }
    num_trees
}
