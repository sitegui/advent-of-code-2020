use crate::data::Data;
use crate::dense_line::{Coordinates, DenseLine, Insert3};
use std::num::NonZeroU32;

type Cup = NonZeroU32;
const NUM_CUPS_P1: u32 = 9;
const NUM_CUPS_P2: u32 = 1_000_000;
const NUM_MOVES_P1: usize = 100;
const NUM_MOVES_P2: usize = 10_000_000;
const LOG_EVERY: usize = 1_000_000;

#[derive(Debug)]
struct Cups {
    cups: DenseLine<Cup>,
    max_cup: Cup,
    current: Coordinates,
    coordinates_by_cup: Vec<Option<Coordinates>>,
}

pub fn solve() -> (i64, i64) {
    let data = Data::read(23);

    let base_cups: Vec<Cup> = data
        .lines()
        .next()
        .unwrap()
        .iter()
        .map(|&b| Cup::new((b - b'0') as u32).unwrap())
        .collect();

    let mut cups_p1 = Cups::new(&base_cups, NUM_CUPS_P1);
    let mut cups_p2 = Cups::new(&base_cups, NUM_CUPS_P2);

    for _ in 0..NUM_MOVES_P1 {
        cups_p1.apply_move(false);
    }
    let part_1 = cups_p1.labels();

    for i in 0..NUM_MOVES_P2 {
        if i % LOG_EVERY == 0 && cfg!(debug_asserts) {
            println!("Move {}", i);
        }
        cups_p2.apply_move(i % LOG_EVERY == 0);
    }
    let cup_1 = Cup::new(1).unwrap();
    let mut pos_1 = cups_p2.find(cup_1).clone();
    let after_1 = cups_p2.cups.next(&mut pos_1);
    let after_after_1 = cups_p2.cups.next(&mut pos_1);
    let part_2 = after_1.get() as i64 * after_after_1.get() as i64;

    (part_1, part_2)
}

impl Cups {
    fn new(base_cups: &[Cup], num_cups: u32) -> Self {
        let more_cups = (base_cups.len() as u32 + 1..=num_cups).map(|cup| Cup::new(cup).unwrap());

        let cups = DenseLine::new(base_cups.iter().copied().chain(more_cups));

        let mut coordinates_by_cup = vec![None; num_cups as usize + 1];
        for (coordinate, cup) in cups.iter() {
            coordinates_by_cup[cup.get() as usize] = Some(coordinate);
        }

        Cups {
            current: cups.iter().next().unwrap().0,
            cups,
            max_cup: Cup::new(num_cups).unwrap(),
            coordinates_by_cup,
        }
    }

    fn iter(&self) -> impl Iterator<Item = Cup> + '_ {
        self.cups.iter().map(|(_, cup)| cup)
    }

    fn find(&self, cup: Cup) -> &Coordinates {
        self.coordinates_by_cup[cup.get() as usize]
            .as_ref()
            .unwrap()
    }

    fn take_coordinates(&mut self, cup: Cup) -> Coordinates {
        self.coordinates_by_cup[cup.get() as usize].take().unwrap()
    }

    fn put_coordinates(&mut self, cup: Cup, coordinates: Coordinates) {
        let old = self.coordinates_by_cup[cup.get() as usize].replace(coordinates);
        debug_assert!(old.is_none());
    }

    fn apply_move(&mut self, log: bool) {
        let result = self.cups.get_1_and_remove_3(&self.current);

        // Determine the destination cup
        let mut destination_cup = result.gotten().get();
        let [x, y, z] = result.removed();
        loop {
            destination_cup -= 1;
            if destination_cup == 0 {
                destination_cup = self.max_cup.get();
            }
            if destination_cup != x.get()
                && destination_cup != y.get()
                && destination_cup != z.get()
            {
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
        let should_rebuild = insert_after.len() >= 16;
        if log && cfg!(debug_asserts) {
            self.cups.print_stats();
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

        if should_rebuild {
            self.rebuild();
        }
    }

    fn labels(&self) -> i64 {
        let cup_1 = self.iter().position(|cup| cup.get() == 1).unwrap();
        let mut labels = 0;
        for (i, cup) in self.iter().enumerate() {
            #[allow(clippy::comparison_chain)]
            if i > cup_1 {
                let exp = self.max_cup.get() as usize - 1 + cup_1 - i;
                labels += cup.get() as i64 * 10i64.pow(exp as u32);
            } else if i < cup_1 {
                let exp = cup_1 - i - 1;
                labels += cup.get() as i64 * 10i64.pow(exp as u32);
            }
        }
        labels
    }

    fn rebuild(&mut self) {
        let current_cup = self.cups.get(&self.current);
        self.cups = DenseLine::new(self.cups.iter().map(|(_, value)| value));

        for (coordinate, cup) in self.cups.iter() {
            self.coordinates_by_cup[cup.get() as usize] = Some(coordinate);
        }
        self.current = self.find(current_cup).clone();
    }
}
