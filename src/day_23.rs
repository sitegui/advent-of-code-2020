use crate::data::Data;
use crate::dense_line::{Coordinates, DenseLine, Insert3};
use std::thread::sleep;
use std::time::Duration;

type Cup = u32;
const NUM_CUPS_P1: u32 = 9;
const NUM_CUPS_P2: u32 = 1_000_000;
const NUM_MOVES_P1: usize = 100;
const NUM_MOVES_P2: usize = 10_000_000;
const LOG_EVERY: usize = 1_000;

#[derive(Debug)]
struct Cups {
    cups: DenseLine<Cup>,
    max_cup: Cup,
    current: Coordinates,
    coordinates_by_cup: Vec<Option<Coordinates>>,
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

    for _ in 0..NUM_MOVES_P1 {
        // println!("{:#?}", cups_p1.cups);
        cups_p1.apply_move(true);
    }
    let part_1 = cups_p1.labels();

    for i in 0..NUM_MOVES_P2 {
        if i % LOG_EVERY == 0 {
            println!("Move {}", i);

            for (i, child) in cups_p2.cups.root.children.iter().enumerate() {
                if let Some(child) = child {
                    let n = child.iter_children().count();
                    if n > 0 {
                        println!("Child {}: {} children", i, n);
                    }
                }
            }
        }
        cups_p2.apply_move(i % LOG_EVERY == 0);
        // sleep(Duration::from_secs_f64(0.1));

        // if i == 40 {
        //     let mut node = &cups_p2.cups.root;
        //     for &c in &[15, 15, 15, 15, 8, 15] {
        //         node = node.children[c].as_ref().unwrap();
        //     }
        //     println!("{:#?}", node);
        // }
    }
    let mut pos_1 = cups_p2.find(1).clone();
    let after_1 = cups_p2.cups.next(&mut pos_1);
    let after_after_1 = cups_p2.cups.next(&mut pos_1);
    let part_2 = after_1 as i64 * after_after_1 as i64;

    (part_1, part_2)
}

impl Cups {
    fn new(base_cups: &[Cup], num_cups: u32) -> Self {
        let mut cups = Vec::with_capacity(num_cups as usize);
        cups.extend_from_slice(base_cups);
        cups.extend(base_cups.len() as u32 + 1..=num_cups);

        let cups = DenseLine::new(&cups);

        let mut coordinates_by_cup = vec![None; num_cups as usize + 1];
        for (coordinate, cup) in cups.iter() {
            coordinates_by_cup[cup as usize] = Some(coordinate);
        }

        Cups {
            current: cups.iter().next().unwrap().0,
            cups,
            max_cup: num_cups,
            coordinates_by_cup,
        }
    }

    fn iter(&self) -> impl Iterator<Item = Cup> + '_ {
        self.cups.iter().map(|(_, cup)| cup)
    }

    fn find(&self, cup: Cup) -> &Coordinates {
        self.coordinates_by_cup[cup as usize].as_ref().unwrap()
    }

    fn take_coordinates(&mut self, cup: Cup) -> Coordinates {
        self.coordinates_by_cup[cup as usize].take().unwrap()
    }

    fn put_coordinates(&mut self, cup: Cup, coordinates: Coordinates) {
        let old = self.coordinates_by_cup[cup as usize].replace(coordinates);
        debug_assert!(old.is_none());
    }

    fn apply_move(&mut self, log: bool) {
        let result = self.cups.get_1_and_remove_3(&self.current);

        // Determine the destination cup
        let mut destination_cup = result.gotten();
        let [x, y, z] = result.removed();
        loop {
            destination_cup -= 1;
            if destination_cup == 0 {
                destination_cup = self.max_cup;
            }
            if destination_cup != x && destination_cup != y && destination_cup != z {
                break;
            }
        }

        let insertion = Insert3::new(
            [x, y, z],
            [
                self.take_coordinates(x),
                self.take_coordinates(y),
                self.take_coordinates(z),
            ],
        );
        let insert_after = self.coordinates_by_cup[destination_cup as usize]
            .as_ref()
            .unwrap();
        if log {
            println!(
                "current = {} @ {:?}, removed = {:?}, insert after = {} @ {:?}",
                result.gotten(),
                self.current,
                result.removed(),
                destination_cup,
                insert_after,
            );
        }

        let insertion = self.cups.insert_3(insert_after, insertion);

        // Update index
        let [new_coords_x, new_coords_y, new_coords_z] = insertion.into_new_coordinates();
        self.put_coordinates(x, new_coords_x);
        self.put_coordinates(y, new_coords_y);
        self.put_coordinates(z, new_coords_z);

        self.cups.next(&mut self.current);
    }

    fn labels(&self) -> i64 {
        let cup_1 = self.iter().position(|cup| cup == 1).unwrap();
        let mut labels = 0;
        for (i, cup) in self.iter().enumerate() {
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
