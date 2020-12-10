use crate::data::{Data, ParseBytes};
use itertools::Itertools;

pub fn solve() -> (i64, i64) {
    // Read the data from the file and detect the maximum value at the same time
    let mut highest_adapter = i32::MIN;
    let mut adapters: Vec<i32> = Data::read(10)
        .lines()
        .map(|line| line.parse_bytes())
        .inspect(|&value| highest_adapter = highest_adapter.max(value))
        .collect_vec();

    // Add outlet and device and sort the values in ascending order
    adapters.push(0);
    adapters.push(highest_adapter + 3);
    adapters.sort();

    // Go through the list and count the number of joltage jumps of size 1 and 3.
    // Also save the size of the runs of jumps of size 1.
    let mut num_diff_1 = 0;
    let mut num_diff_3 = 0;
    let mut run_sizes = Vec::with_capacity(adapters.len());
    let mut current_run_size = 1;
    let mut max_run_size = i32::MIN;
    for (output, input) in adapters.iter().copied().tuple_windows() {
        match input - output {
            1 => {
                num_diff_1 += 1;
                current_run_size += 1;
            }
            3 => {
                num_diff_3 += 1;
                run_sizes.push(current_run_size);
                max_run_size = max_run_size.max(current_run_size);
                current_run_size = 1;
            }
            _ => unreachable!(),
        }
    }

    let part_1 = num_diff_1 * num_diff_3;

    // Jumps of size 3 force us to pick one specific adapter. On the other hand, jumps of size 1
    // gives us more freedom of choice. In a run of size `N`, we have `C(N)` ways of connecting the
    // adaptors.
    // Since a run is surrounded by jumps of size 3, we know the first and last adaptor must be
    // chosen (otherwise we could not make the jump of size 3 to or from this run).
    // After taking the first adaptor, we can either connect the second, third or fourth. For
    // whichever choice we make a similar, but smaller, problem is presented: how to connect `N-1`,
    // `N-2` or `N-3` adapters, respectively.
    // This gives us the recursion: `C(N) = C(N-3) + C(N-2) + C(N-1)`.
    let mut num_combinations = vec![0, 1, 1];
    for i in 3..=max_run_size as usize {
        num_combinations
            .push(num_combinations[i - 3] + num_combinations[i - 2] + num_combinations[i - 1]);
    }

    // Each run can be formed independently. So the total combination is simply their product.
    let mut total: i64 = 1;
    for run_size in run_sizes {
        total *= num_combinations[run_size as usize];
    }

    (part_1, total)
}
