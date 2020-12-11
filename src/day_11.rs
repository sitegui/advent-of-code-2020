use crate::data::Data;
use itertools::Itertools;
use std::fmt::{Debug, Formatter};
use std::ops::{Add, AddAssign};
use std::{fmt, mem};

#[derive(Copy, Clone, Eq, PartialEq)]
enum Tile {
    Floor,
    Empty,
    Occupied,
}

#[derive(Clone, Eq, PartialEq)]
struct Layout {
    height: i8,
    width: i8,
    tiles: Vec<Tile>,
}

#[derive(Copy, Clone)]
struct Index {
    x: i8,
    y: i8,
    index: i16,
}

#[derive(Debug)]
struct Influencers {
    width: i8,
    influencers: Vec<Index>,
    num_influencers: Vec<(Index, i8)>,
}

pub fn solve() -> (i64, i64) {
    let original_layout = Layout::from_tiles(
        Data::read(11)
            .lines()
            .map(|line| line.iter().map(|&b| Tile::from_byte(b))),
    );

    let directions = &[
        Index::new_with_layout(&original_layout, 1, 1),
        Index::new_with_layout(&original_layout, 1, 0),
        Index::new_with_layout(&original_layout, 1, -1),
        Index::new_with_layout(&original_layout, 0, 1),
        Index::new_with_layout(&original_layout, 0, -1),
        Index::new_with_layout(&original_layout, -1, 1),
        Index::new_with_layout(&original_layout, -1, 0),
        Index::new_with_layout(&original_layout, -1, -1),
    ];

    let mut layout_p1 = original_layout.clone();
    let influencers_p1 = Influencers::new(&layout_p1, |index, influencers| {
        for &direction in directions {
            let influencer = index + direction;
            if layout_p1.try_get(influencer) == Some(Tile::Empty) {
                influencers.push(influencer);
            }
        }
    });

    let variable_occupancy = layout_p1.mark_constantly_occupied(&influencers_p1, 4);

    let mut buffer = layout_p1.clone();
    while layout_p1.evolve(&variable_occupancy, 4, &mut buffer) {
        mem::swap(&mut layout_p1, &mut buffer);
    }

    let mut layout_p2 = original_layout;
    let influencers_p2 = Influencers::new(&layout_p2, |index, influencers| {
        for &dir in directions {
            let mut influencer = index;
            loop {
                influencer += dir;
                match layout_p2.try_get(influencer) {
                    None => break,
                    Some(Tile::Empty) => {
                        influencers.push(influencer);
                        break;
                    }
                    _ => {}
                }
            }
        }
    });

    let variable_occupancy = layout_p2.mark_constantly_occupied(&influencers_p2, 5);

    let mut buffer = layout_p2.clone();
    while layout_p2.evolve(&variable_occupancy, 5, &mut buffer) {
        mem::swap(&mut layout_p2, &mut buffer);
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
            width: (all_tiles.len() / height as usize) as i8,
            tiles: all_tiles,
        }
    }

    fn set(&mut self, index: Index, tile: Tile) {
        self.tiles[index.index as usize] = tile;
    }

    fn get(&self, index: Index) -> Tile {
        self.tiles[index.index as usize]
    }

    fn try_get(&self, index: Index) -> Option<Tile> {
        if index.x < 0 || index.x >= self.width || index.y < 0 || index.y >= self.height {
            None
        } else {
            Some(self.get(index))
        }
    }

    /// Mark the seats with less than `max_influencers` as occupied.
    /// Also, return the seat indexes that have `max_influencers` or more.
    fn mark_constantly_occupied<'a>(
        &mut self,
        influencers: &'a Influencers,
        max_influencers: i8,
    ) -> Vec<(Index, &'a [Index])> {
        let mut variable_occupancy = Vec::with_capacity(self.tiles.len());

        for (index, influencers) in influencers.iter() {
            if (influencers.len() as i8) < max_influencers {
                self.set(index, Tile::Occupied);
            } else {
                variable_occupancy.push((index, influencers));
            }
        }

        variable_occupancy
    }

    fn evolve(
        &self,
        variable_occupancy: &[(Index, &[Index])],
        max_occupied: i8,
        buffer: &mut Layout,
    ) -> bool {
        let mut changed = false;

        for &(index, influencers) in variable_occupancy {
            if self.evolve_tile(index, influencers, max_occupied, buffer) {
                changed = true;
            }
        }

        changed
    }

    fn evolve_tile(
        &self,
        index: Index,
        influencers: &[Index],
        max_occupied: i8,
        buffer: &mut Layout,
    ) -> bool {
        let tile = self.get(index);

        match tile {
            Tile::Empty => {
                for &influencer in influencers {
                    if self.get(influencer) == Tile::Occupied {
                        buffer.set(index, Tile::Empty);
                        return false;
                    }
                }
                buffer.set(index, Tile::Occupied);
                true
            }
            Tile::Occupied => {
                let mut occupied = 0;
                for &influencer in influencers {
                    if self.get(influencer) == Tile::Occupied {
                        occupied += 1;
                        if occupied == max_occupied {
                            buffer.set(index, Tile::Empty);
                            return true;
                        }
                    }
                }
                buffer.set(index, Tile::Occupied);
                false
            }
            _ => unreachable!(),
        }
    }

    fn total_occupied(&self) -> usize {
        self.tiles
            .iter()
            .filter(|&&tile| tile == Tile::Occupied)
            .count()
    }

    fn iter(&self) -> impl Iterator<Item = (Index, Tile)> + '_ {
        self.tiles
            .chunks(self.width as usize)
            .enumerate()
            .flat_map(move |(y, row)| {
                row.iter()
                    .enumerate()
                    .map(move |(x, &tile)| (Index::new_with_layout(self, x as i8, y as i8), tile))
            })
    }
}

impl Index {
    fn new_with_layout(layout: &Layout, x: i8, y: i8) -> Self {
        let index = x as i16 + y as i16 * layout.width as i16;
        Self::new(x, y, index)
    }

    fn new(x: i8, y: i8, index: i16) -> Self {
        Index { x, y, index }
    }
}

impl Influencers {
    fn new<F>(layout: &Layout, mut extend_for_tile: F) -> Self
    where
        F: FnMut(Index, &mut Vec<Index>),
    {
        let mut influencers = Influencers {
            width: layout.width,
            influencers: Vec::with_capacity(layout.tiles.len() * 8),
            num_influencers: Vec::with_capacity(layout.tiles.len()),
        };
        for (index, _) in layout.iter() {
            if layout.get(index) != Tile::Floor {
                let len_before = influencers.influencers.len();
                extend_for_tile(index, &mut influencers.influencers);
                let num_influencers = (influencers.influencers.len() - len_before) as i8;
                influencers.num_influencers.push((index, num_influencers));
            };
        }
        influencers
    }

    fn iter(&self) -> impl Iterator<Item = (Index, &[Index])> {
        let mut offset: usize = 0;
        self.num_influencers
            .iter()
            .map(move |&(index, num_influencers)| {
                let prev_offset = offset;
                offset += num_influencers as usize;
                let influencers = &self.influencers[prev_offset..offset];
                (index, influencers)
            })
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
        for line in self.tiles.chunks(self.width as usize) {
            writeln!(f, "{:?}", line.iter().format(""))?;
        }

        Ok(())
    }
}

impl Debug for Index {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "({},{})", self.x, self.y)
    }
}

impl Add for Index {
    type Output = Index;

    fn add(self, rhs: Self) -> Self::Output {
        Index {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            index: self.index + rhs.index,
        }
    }
}

impl AddAssign for Index {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.index += rhs.index;
    }
}
