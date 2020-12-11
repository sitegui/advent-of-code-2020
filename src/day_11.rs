use crate::data::Data;
use itertools::Itertools;
use std::fmt;
use std::fmt::{Debug, Formatter};

#[derive(Copy, Clone, Eq, PartialEq)]
enum Tile {
    Floor,
    Empty,
    Occupied,
}

#[derive(Clone, Eq, PartialEq)]
struct Layout {
    height: usize,
    width: usize,
    tiles: Vec<Tile>,
}

pub fn solve() -> (i64, i64) {
    let mut layout_p1 = Layout::from_tiles(
        Data::read(11)
            .lines()
            .map(|line| line.iter().map(|&b| Tile::from_byte(b))),
    );
    let mut layout_p2 = layout_p1.clone();

    loop {
        let evolved = layout_p1.evolved_p1();
        if evolved == layout_p1 {
            break;
        }
        layout_p1 = evolved;
    }

    loop {
        let evolved = layout_p2.evolved_p2();
        if evolved == layout_p2 {
            break;
        }
        layout_p2 = evolved;
    }

    (
        layout_p1.total_occupied() as i64,
        layout_p2.total_occupied() as i64,
    )
}

impl Tile {
    fn from_byte(b: u8) -> Self {
        match b {
            b'.' => Tile::Floor,
            b'L' => Tile::Empty,
            b'#' => Tile::Occupied,
            _ => unreachable!(),
        }
    }
}

impl Layout {
    fn from_tiles(tiles: impl Iterator<Item = impl Iterator<Item = Tile>>) -> Self {
        let mut all_tiles = vec![];
        let mut height = 0;
        for line in tiles {
            all_tiles.extend(line);
            height += 1;
        }
        Layout {
            height,
            width: all_tiles.len() / height,
            tiles: all_tiles,
        }
    }

    fn get(&self, x: isize, y: isize) -> Option<Tile> {
        if x < 0 || x >= self.width as isize || y < 0 || y >= self.height as isize {
            None
        } else {
            Some(self.tiles[(x + y * self.width as isize) as usize])
        }
    }

    fn occupied_adjacent(&self, x: usize, y: usize) -> u8 {
        let mut occupied = 0;
        let x = x as isize;
        let y = y as isize;

        macro_rules! check_cell {
            ($dx:expr, $dy:expr) => {
                if self.get(x + $dx, y + $dy) == Some(Tile::Occupied) {
                    occupied += 1;
                }
            };
        }

        check_cell!(-1, -1);
        check_cell!(0, -1);
        check_cell!(1, -1);
        check_cell!(-1, 0);
        check_cell!(1, 0);
        check_cell!(-1, 1);
        check_cell!(0, 1);
        check_cell!(1, 1);

        occupied
    }

    fn occupied_visible(&self, x: usize, y: usize) -> u8 {
        let mut occupied = 0;
        let x = x as isize;
        let y = y as isize;

        macro_rules! check_direction {
            ($dx:expr, $dy:expr) => {
                if self.occupied_in_line(x, y, $dx, $dy) {
                    occupied += 1;
                }
            };
        }

        check_direction!(-1, -1);
        check_direction!(0, -1);
        check_direction!(1, -1);
        check_direction!(-1, 0);
        check_direction!(1, 0);
        check_direction!(-1, 1);
        check_direction!(0, 1);
        check_direction!(1, 1);

        occupied
    }

    fn occupied_in_line(&self, mut x: isize, mut y: isize, dx: isize, dy: isize) -> bool {
        loop {
            x += dx;
            y += dy;

            match self.get(x, y) {
                None | Some(Tile::Empty) => return false,
                Some(Tile::Occupied) => return true,
                _ => {}
            }
        }
    }

    fn evolved_p1(&self) -> Self {
        let mut new_tiles = Vec::with_capacity(self.tiles.len());
        for (y, row) in self.tiles.chunks(self.width).enumerate() {
            for (x, &tile) in row.iter().enumerate() {
                if tile == Tile::Empty && self.occupied_adjacent(x, y) == 0 {
                    new_tiles.push(Tile::Occupied);
                } else if tile == Tile::Occupied && self.occupied_adjacent(x, y) >= 4 {
                    new_tiles.push(Tile::Empty);
                } else {
                    new_tiles.push(tile);
                }
            }
        }
        Layout {
            height: self.height,
            width: self.width,
            tiles: new_tiles,
        }
    }

    fn evolved_p2(&self) -> Self {
        let mut new_tiles = Vec::with_capacity(self.tiles.len());
        for (y, row) in self.tiles.chunks(self.width).enumerate() {
            for (x, &tile) in row.iter().enumerate() {
                if tile == Tile::Empty && self.occupied_visible(x, y) == 0 {
                    new_tiles.push(Tile::Occupied);
                } else if tile == Tile::Occupied && self.occupied_visible(x, y) >= 5 {
                    new_tiles.push(Tile::Empty);
                } else {
                    new_tiles.push(tile);
                }
            }
        }
        Layout {
            height: self.height,
            width: self.width,
            tiles: new_tiles,
        }
    }

    fn total_occupied(&self) -> usize {
        self.tiles
            .iter()
            .filter(|&&tile| tile == Tile::Occupied)
            .count()
    }
}

impl Debug for Tile {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let c = match *self {
            Tile::Floor => '.',
            Tile::Empty => 'L',
            Tile::Occupied => '#',
        };
        write!(f, "{}", c)
    }
}

impl Debug for Layout {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f)?;
        for line in self.tiles.chunks(self.width) {
            writeln!(f, "{:?}", line.iter().format(""))?;
        }

        Ok(())
    }
}
