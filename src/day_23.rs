use crate::data::Data;
use itertools::Itertools;
use std::ops::Index;
use std::{fmt, iter, slice};

type Cup = u32;
const NO_CUP: Cup = u32::MAX;
const NUM_CUPS_P1: u32 = 9;
const NUM_CUPS_P2: u32 = 1_000_000;
const NUM_MOVES_P1: usize = 100;
const NUM_MOVES_P2: usize = 10_000_000;
const ROPE_NODE_LEN: usize = 1_000;
const LOG_EVERY: usize = 10_000;

#[derive(Debug)]
struct Cups {
    cups: Rope,
    max_cup: Cup,
    current: Position,
}

#[derive(Debug, Clone)]
struct Rope {
    nodes: Vec<Node>,
}

#[derive(Debug, Clone)]
struct Node {
    values: Vec<Cup>,
    min_max: Option<(Cup, Cup)>,
}

#[derive(Debug, Copy, Clone)]
struct Position {
    node_index: usize,
    value_index: usize,
}

pub fn solve() -> (i64, i64) {
    let data = Data::read_example();

    let base_cups: Vec<Cup> = data
        .lines()
        .next()
        .unwrap()
        .iter()
        .map(|&b| (b - b'0') as u32)
        .collect();

    let mut cups_p1 = Cups::new(&base_cups, NUM_CUPS_P1);
    let mut cups_p2 = Cups::new(&base_cups, NUM_CUPS_P2);

    for i in 0..NUM_MOVES_P1 {
        println!("{}: [{}]", i, cups_p1.cups.iter().format(", "));
        cups_p1.apply_move(true);
    }
    let part_1 = cups_p1.labels();

    for i in 0..NUM_MOVES_P2 {
        if i % LOG_EVERY == 0 {
            println!("Move {}", i);
            debug_assert!(cups_p2.cups.iter().sorted().eq(1..=NUM_CUPS_P2));

            for node in &cups_p2.cups.nodes {
                debug_assert_eq!(
                    node.min_max,
                    node.values.iter().copied().minmax().into_option()
                );
            }
        }
        cups_p2.apply_move(i % LOG_EVERY == 0);
    }
    let part_2 = 0;

    (part_1, part_2)
}

impl Cups {
    fn new(base_cups: &[Cup], num_cups: u32) -> Self {
        let cups = Rope::new(
            base_cups
                .iter()
                .copied()
                .chain(base_cups.len() as u32 + 1..=num_cups),
            ROPE_NODE_LEN,
        );
        Cups {
            current: cups.first_position(),
            cups,
            max_cup: num_cups,
        }
    }

    fn apply_move(&mut self, log: bool) {
        let (current_cup, [x, y, z], mut current) = self.cups.get_1_and_remove_3(self.current);
        debug_assert_eq!(
            self.cups.nodes[current.node_index][current.value_index],
            current_cup
        );
        self.cups.try_merge(&mut current);
        debug_assert_eq!(
            self.cups.nodes[current.node_index][current.value_index],
            current_cup
        );

        // Determine the destination cup
        let mut destination_cup = current_cup;
        loop {
            destination_cup -= 1;
            if destination_cup == 0 {
                destination_cup = self.max_cup;
            }
            if destination_cup != x && destination_cup != y && destination_cup != z {
                break;
            }
        }

        let (insertion_point, skipped_nodes, explored_nodes) =
            self.cups.find(destination_cup, current);
        let current = self.cups.insert_after(insertion_point, [x, y, z], current);
        debug_assert_eq!(
            self.cups.nodes[current.node_index][current.value_index],
            current_cup
        );
        if log {
            let min_max_lens = self
                .cups
                .nodes
                .iter()
                .map(|node| node.len())
                .minmax()
                .into_option()
                .unwrap();
            println!(
                "current = {} @ {}, removed = {:?}, insert after = {} @ {}, skipped_nodes = {}, explored_nodes = {}, min_max_lens = {:?}",
                current_cup,
                self.current,
                [x, y, z],
                destination_cup,
                insertion_point,
                skipped_nodes,
                explored_nodes,
                min_max_lens
            );
        }

        self.current = self.cups.advance(current);
    }

    fn labels(&self) -> i64 {
        let cup_1 = self.cups.iter().position(|cup| cup == 1).unwrap();
        let mut labels = 0;
        for (i, cup) in self.cups.iter().enumerate() {
            if i > cup_1 {
                let exp = self.max_cup as usize - 1 + cup_1 - i;
                labels += cup as i64 * 10i64.pow(exp as u32);
            } else if i < cup_1 {
                let exp = cup_1 - i - 1;
                labels += cup as i64 * 10i64.pow(exp as u32);
            }
        }
        labels
    }
}

impl Rope {
    fn new(values: impl IntoIterator<Item = Cup>, node_size: usize) -> Self {
        let mut nodes = vec![];

        // Build nodes
        let mut new_node = Node::new(node_size);
        for value in values {
            new_node.push(value);

            if new_node.len() == node_size {
                nodes.push(new_node);
                new_node = Node::new(node_size);
            }
        }
        if !new_node.is_empty() {
            nodes.push(new_node);
        }

        Rope { nodes }
    }

    fn first_position(&self) -> Position {
        Position {
            node_index: 0,
            value_index: 0,
        }
    }

    #[inline(never)]
    fn get_1_and_remove_3(&mut self, mut current: Position) -> (Cup, [Cup; 3], Position) {
        let value = self.nodes[current.node_index][current.value_index];

        current = self.advance(current);
        let mut node = &mut self.nodes[current.node_index];

        let mut removed = [NO_CUP; 3];
        let mut num_removed = 0;

        loop {
            while current.value_index < node.len() {
                removed[num_removed] = node.remove(current.value_index);
                num_removed += 1;

                if num_removed == removed.len() {
                    return (value, removed, self.rewind(current));
                }
            }

            current.value_index = 0;
            current.node_index = if current.node_index == self.nodes.len() - 1 {
                0
            } else {
                current.node_index + 1
            };

            node = &mut self.nodes[current.node_index];
        }
    }

    #[inline(never)]
    fn try_merge(&mut self, position: &mut Position) {
        if self.nodes.len() == 1 {
            return;
        }

        let (base, next, index_to_remove) = if position.node_index == self.nodes.len() - 1 {
            let (base, head) = self.nodes.split_last_mut().unwrap();
            (base, head.first().unwrap(), 0)
        } else {
            let (head, tail) = self.nodes.split_at_mut(position.node_index + 1);
            (
                head.last_mut().unwrap(),
                tail.first().unwrap(),
                position.node_index + 1,
            )
        };

        if let Ok(()) = base.try_merge(next) {
            self.nodes.remove(index_to_remove);

            if index_to_remove < position.node_index {
                position.node_index -= 1;
            }
        }
    }

    #[inline(never)]
    fn find(&self, target: Cup, mut start: Position) -> (Position, usize, usize) {
        let mut max_index = start.value_index;
        let mut node = &self.nodes[start.node_index];
        let mut skipped_nodes = 0;
        let mut explored_nodes = 0;

        loop {
            match node.find(target, max_index) {
                None => skipped_nodes += 1,
                Some(None) => explored_nodes += 1,
                Some(Some(value_index)) => {
                    return (
                        Position {
                            node_index: start.node_index,
                            value_index,
                        },
                        skipped_nodes,
                        explored_nodes,
                    )
                }
            }

            start.node_index = if start.node_index == 0 {
                self.nodes.len() - 1
            } else {
                start.node_index - 1
            };
            node = &self.nodes[start.node_index];
            max_index = node.len();
        }
    }

    #[inline(never)]
    fn insert_after(
        &mut self,
        position: Position,
        cups: [Cup; 3],
        mut current: Position,
    ) -> Position {
        let node = &mut self.nodes[position.node_index];

        if position.node_index == current.node_index && position.value_index < current.value_index {
            current = Position {
                node_index: current.node_index,
                value_index: current.value_index + 3,
            };
        }

        node.insert_after(cups, position.value_index);
        if let Some(split_node) = node.try_split() {
            let node_len = node.len();
            self.nodes.insert(position.node_index + 1, split_node);

            if current.node_index > position.node_index {
                current.node_index += 1;
            } else if current.node_index == position.node_index && current.value_index >= node_len {
                current.node_index += 1;
                current.value_index -= node_len;
            }
        }

        current
    }

    fn iter(&self) -> impl Iterator<Item = Cup> + '_ {
        self.nodes.iter().flat_map(|node| node.iter())
    }

    fn advance(&self, mut position: Position) -> Position {
        let remaining_values = self.nodes[position.node_index].len() - position.value_index - 1;
        if remaining_values > 0 {
            position.value_index += 1;
            return position;
        }

        position.value_index = 0;
        loop {
            position.node_index = if position.node_index == self.nodes.len() - 1 {
                0
            } else {
                position.node_index + 1
            };

            if !self.nodes[position.node_index].is_empty() {
                return position;
            }
        }
    }

    fn rewind(&self, mut position: Position) -> Position {
        if position.value_index > 0 {
            position.value_index -= 1;
            return position;
        }

        loop {
            position.node_index = if position.node_index == 0 {
                self.nodes.len() - 1
            } else {
                position.node_index - 1
            };

            let len = self.nodes[position.node_index].len();
            if len > 0 {
                position.value_index = len - 1;
                return position;
            }
        }
    }
}

impl Node {
    fn new(capacity: usize) -> Self {
        Node {
            values: Vec::with_capacity(capacity),
            min_max: None,
        }
    }

    fn push(&mut self, value: Cup) {
        self.values.push(value);
        self.min_max = Some(match self.min_max {
            None => (value, value),
            Some((min, max)) => (min.min(value), max.max(value)),
        });
    }

    fn len(&self) -> usize {
        self.values.len()
    }

    fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    fn remove(&mut self, index: usize) -> Cup {
        let removed = self.values.remove(index);
        if let Some(min_max) = self.min_max {
            if min_max.0 == removed || min_max.1 == removed {
                self.min_max = self.values.iter().copied().minmax().into_option();
            }
        }

        removed
    }

    fn find(&self, target: Cup, max_index: usize) -> Option<Option<usize>> {
        match self.min_max {
            None => None,
            Some((min, max)) if target < min || target > max => None,
            _ => Some(
                self.values[..max_index]
                    .iter()
                    .position(|&value| value == target),
            ),
        }
    }

    fn insert_after(&mut self, values: [Cup; 3], index: usize) {
        self.values
            .splice(index + 1..index + 1, values.iter().copied());

        let inserted_min = values[0].min(values[1]).min(values[2]);
        let inserted_max = values[0].max(values[1]).max(values[2]);
        self.min_max = Some(match self.min_max {
            None => (inserted_min, inserted_max),
            Some((min, max)) => (min.min(inserted_min), max.max(inserted_max)),
        });
    }

    fn iter(&self) -> iter::Copied<slice::Iter<'_, Cup>> {
        self.values.iter().copied()
    }

    fn try_split(&mut self) -> Option<Self> {
        if self.len() < 2 * ROPE_NODE_LEN {
            return None;
        }

        let values = self.values.split_off(self.values.len() / 2);
        self.min_max = self.values.iter().copied().minmax().into_option();
        Some(Node {
            min_max: values.iter().copied().minmax().into_option(),
            values,
        })
    }

    fn try_merge(&mut self, other: &Self) -> Result<(), ()> {
        if self.len() > ROPE_NODE_LEN / 2 || self.len() + other.len() > 2 * ROPE_NODE_LEN {
            return Err(());
        }

        self.values.extend(other.values.iter().copied());
        self.min_max = match (self.min_max, other.min_max) {
            (None, None) => None,
            (Some(x), None) | (None, Some(x)) => Some(x),
            (Some((self_min, self_max)), Some((other_min, other_max))) => {
                Some((self_min.min(other_min), self_max.max(other_max)))
            }
        };

        Ok(())
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {})", self.node_index, self.value_index)
    }
}

impl Index<usize> for Node {
    type Output = Cup;

    fn index(&self, index: usize) -> &Self::Output {
        &self.values[index]
    }
}
