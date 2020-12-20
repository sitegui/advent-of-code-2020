use crate::data::{Data, ParseBytes, TryFromBytes};
use crate::iter_utils::IterUtils;
use crate::matrix::Matrix;
use crate::parser::Parser;
use itertools::Itertools;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::mem;

const TILE_PIXELS: usize = 10;
const IMAGE_TILES: usize = 12;
const IMAGE_PIXELS: usize = IMAGE_TILES * (TILE_PIXELS - 2);
const MONSTER: &[&[u8]] = &[
    b"                  # ",
    b"#    ##    ##    ###",
    b" #  #  #  #  #  #   ",
];

#[derive(Debug, Clone, Copy)]
struct Border {
    pixels: [u8; TILE_PIXELS],
    neighbor: Option<i64>,
}

#[derive(Debug, Clone)]
struct Tile {
    id: i64,
    pixels: Matrix<u8>,
    border_top: Border,
    border_bottom: Border,
    border_right: Border,
    border_left: Border,
    transform: Transform,
}

struct Tiles {
    tiles: BTreeMap<i64, RefCell<Tile>>,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
struct Transform {
    vertical_flip: bool,
    cw_rotations: u8,
}

pub fn solve() -> (i64, i64) {
    let data = Data::read(20);

    let mut tiles = Tiles::parse(data.bytes());
    assert_eq!(tiles.tiles.len(), IMAGE_TILES * IMAGE_TILES);

    // Find which tiles are neighbors
    for (tile, other_tile) in tiles.tiles.values().tuple_combinations::<(_, _)>() {
        tile.borrow_mut()
            .mark_shared_borders(&mut other_tile.borrow_mut());
    }

    // Find a corner
    let corner_id = tiles.find_a_corner();
    let mut corner = tiles.remove(corner_id);

    // Rotate until it fits the top-left corner and start the image from it
    while corner.border_top.neighbor.is_some() || corner.border_left.neighbor.is_some() {
        corner.rotate_cw();
    }
    let mut image_tiles = Matrix::new(vec![None; IMAGE_TILES * IMAGE_TILES], IMAGE_TILES);
    image_tiles[(0, 0)] = Some(corner);

    // Build the image
    for y in 0..IMAGE_TILES {
        for x in 0..IMAGE_TILES {
            if x == 0 && y == 0 {
                // Top left corner is determined outside this loop
                continue;
            }

            // Lookup the tiles that are immediately above and on the left
            let top_neighbor = if y == 0 {
                None
            } else {
                image_tiles[(x, y - 1)].as_ref()
            };
            let left_neighbor = if x == 0 {
                None
            } else {
                image_tiles[(x - 1, y)].as_ref()
            };

            // They will tell which tile to pick, so do it
            let new_tile_id = top_neighbor
                .map(|tile| tile.border_bottom.neighbor.unwrap())
                .unwrap_or_else(|| left_neighbor.unwrap().border_right.neighbor.unwrap());
            let mut new_tile = tiles.remove(new_tile_id);

            // Find the right transform to apply
            new_tile.transform_until(
                top_neighbor.map(|tile| tile.id),
                left_neighbor.map(|tile| tile.id),
            );

            // Add to the image
            image_tiles[(x, y)] = Some(new_tile);
        }
    }

    let tile_id_at = |x, y| image_tiles[(x, y)].as_ref().unwrap().id;
    let part_1 = tile_id_at(0, 0)
        * tile_id_at(IMAGE_TILES - 1, 0)
        * tile_id_at(0, IMAGE_TILES - 1)
        * tile_id_at(IMAGE_TILES - 1, IMAGE_TILES - 1);

    // Form final image buffer
    let mut image = Vec::with_capacity(IMAGE_PIXELS * IMAGE_PIXELS);
    for tile_row in image_tiles.rows() {
        let tile_row_pixels = tile_row
            .iter()
            .map(|tile| {
                let tile = tile.as_ref().unwrap();
                tile.transform.apply(&tile.pixels)
            })
            .collect_vec();
        for y in 1..TILE_PIXELS - 1 {
            for tile_pixels in &tile_row_pixels {
                image.extend_from_slice(&tile_pixels.row(y)[1..TILE_PIXELS - 1]);
            }
        }
    }
    let image = Matrix::new(image, IMAGE_PIXELS);

    // Find all possible monsters
    let monster = Matrix::new(
        MONSTER.iter().copied().flatten().copied().collect_vec(),
        MONSTER[0].len(),
    );
    let num_monsters = count_monsters_all_transformations(&image, &monster);

    let monster_pixels = monster
        .values()
        .iter()
        .filter(|&&byte| byte == b'#')
        .count();
    let part_2 =
        image.values().iter().filter(|&&byte| byte == b'#').count() - num_monsters * monster_pixels;

    (part_1, part_2 as i64)
}

impl TryFromBytes for Tile {
    fn try_from_bytes(bytes: &[u8]) -> Option<Self> {
        let mut lines = bytes.lines();

        let mut header = lines.next().unwrap();
        header.consume_prefix(b"Tile ");
        let id = header.consume_until(b':').parse_bytes();

        let mut border_top = Border::new();
        let mut border_bottom = Border::new();
        let mut border_right = Border::new();
        let mut border_left = Border::new();

        let pixels = Matrix::new(lines.flatten().copied().collect(), TILE_PIXELS);
        assert_eq!(pixels.width(), pixels.height());

        for x in 0..TILE_PIXELS {
            border_top.pixels[x] = pixels[(x, 0)];
            border_bottom.pixels[x] = pixels[(x, TILE_PIXELS - 1)];
        }
        for y in 0..TILE_PIXELS {
            border_left.pixels[y] = pixels[(0, y)];
            border_right.pixels[y] = pixels[(TILE_PIXELS - 1, y)];
        }

        Some(Tile {
            id,
            pixels,
            border_top,
            border_bottom,
            border_right,
            border_left,
            transform: Transform::IDENTITY,
        })
    }
}

impl Border {
    fn new() -> Self {
        Border {
            pixels: [u8::MAX; TILE_PIXELS],
            neighbor: None,
        }
    }

    fn mark_shared_border(&mut self, self_id: i64, other_tile: &mut Tile) {
        if self.pixels == other_tile.border_top.pixels
            || rev_eq(self.pixels, other_tile.border_top.pixels)
        {
            self.neighbor = Some(other_tile.id);
            other_tile.border_top.neighbor = Some(self_id);
        } else if self.pixels == other_tile.border_bottom.pixels
            || rev_eq(self.pixels, other_tile.border_bottom.pixels)
        {
            self.neighbor = Some(other_tile.id);
            other_tile.border_bottom.neighbor = Some(self_id);
        } else if self.pixels == other_tile.border_right.pixels
            || rev_eq(self.pixels, other_tile.border_right.pixels)
        {
            self.neighbor = Some(other_tile.id);
            other_tile.border_right.neighbor = Some(self_id);
        } else if self.pixels == other_tile.border_left.pixels
            || rev_eq(self.pixels, other_tile.border_left.pixels)
        {
            self.neighbor = Some(other_tile.id);
            other_tile.border_left.neighbor = Some(self_id);
        }
    }
}

impl Tile {
    fn mark_shared_borders(&mut self, other: &mut Self) {
        self.border_top.mark_shared_border(self.id, other);
        self.border_bottom.mark_shared_border(self.id, other);
        self.border_right.mark_shared_border(self.id, other);
        self.border_left.mark_shared_border(self.id, other);
    }

    fn num_neighbors(&self) -> u8 {
        self.border_top.neighbor.is_some() as u8
            + self.border_bottom.neighbor.is_some() as u8
            + self.border_right.neighbor.is_some() as u8
            + self.border_left.neighbor.is_some() as u8
    }

    fn rotate_cw(&mut self) {
        self.transform.rotate_cw();
        let old_top = mem::replace(&mut self.border_top, self.border_left);
        let old_right = mem::replace(&mut self.border_right, old_top);
        let old_bottom = mem::replace(&mut self.border_bottom, old_right);
        self.border_left = old_bottom;
    }

    fn vertical_flip(&mut self) {
        self.transform.vertical_flip();
        let old_right = mem::replace(&mut self.border_right, self.border_left);
        self.border_left = old_right;
    }

    fn transform_until(&mut self, top_neighbor: Option<i64>, left_neighbor: Option<i64>) {
        for i in 0..8 {
            if self.border_top.neighbor == top_neighbor
                && self.border_left.neighbor == left_neighbor
            {
                return;
            }

            self.rotate_cw();
            if i == 3 {
                self.vertical_flip();
            }
        }

        unreachable!()
    }
}

impl Tiles {
    fn parse(data: &[u8]) -> Self {
        Tiles {
            tiles: data
                .paragraphs()
                .parsed::<Tile>()
                .map(|tile| (tile.id, RefCell::new(tile)))
                .collect(),
        }
    }

    fn find_a_corner(&self) -> i64 {
        for tile in self.tiles.values() {
            let tile = tile.borrow();
            if tile.num_neighbors() == 2 {
                return tile.id;
            }
        }
        unreachable!()
    }

    fn remove(&mut self, id: i64) -> Tile {
        self.tiles.remove(&id).unwrap().into_inner()
    }
}

fn rev_eq(a: [u8; TILE_PIXELS], b: [u8; TILE_PIXELS]) -> bool {
    for i in 0..TILE_PIXELS {
        if a[i] != b[TILE_PIXELS - i - 1] {
            return false;
        }
    }
    true
}

impl Transform {
    const IDENTITY: Transform = Transform {
        vertical_flip: false,
        cw_rotations: 0,
    };

    fn rotate_cw(&mut self) {
        self.cw_rotations = (self.cw_rotations + 1) % 4;
    }

    fn vertical_flip(&mut self) {
        self.vertical_flip = !self.vertical_flip;
    }

    fn apply<T: Clone>(self, matrix: &Matrix<T>) -> Matrix<T> {
        if self == Transform::IDENTITY {
            return matrix.clone();
        }

        let (width, height) = if self.cw_rotations % 2 == 0 {
            (matrix.width(), matrix.height())
        } else {
            (matrix.height(), matrix.width())
        };
        Matrix::new_with_fn(
            |x, y| {
                // println!("For {} {}", x, y);
                let (x, y) = match self.cw_rotations {
                    0 => (x, y),
                    1 => (y, matrix.height() - x - 1),
                    2 => (matrix.width() - x - 1, matrix.height() - y - 1),
                    3 => (matrix.width() - y - 1, x),
                    _ => unreachable!(),
                };

                let (x, y) = if self.vertical_flip {
                    (matrix.width() - x - 1, y)
                } else {
                    (x, y)
                };

                // println!("Get {} {}", x, y);
                matrix[(x, y)].clone()
            },
            width,
            height,
        )
    }
}

fn count_monsters_all_transformations(image: &Matrix<u8>, monster: &Matrix<u8>) -> usize {
    let mut transform = Transform::IDENTITY;
    for i in 0..8 {
        let monster = transform.apply(monster);

        let num_monsters = count_monsters(image, &monster);
        if num_monsters != 0 {
            return num_monsters;
        }

        transform.rotate_cw();
        if i == 3 {
            transform.vertical_flip();
        }
    }
    unreachable!()
}

fn count_monsters(image: &Matrix<u8>, monster: &Matrix<u8>) -> usize {
    let mut num_monsters = 0;

    // First, find some (x, y) in which the first row of the monster matches
    for y in 0..image.height() - monster.height() + 1 {
        for x in 0..image.width() - monster.width() + 1 {
            if match_monster(image, monster, x, y) {
                num_monsters += 1;
            }
        }
    }

    num_monsters
}

fn match_monster(image: &Matrix<u8>, monster: &Matrix<u8>, x: usize, y: usize) -> bool {
    for (dy, monster_row) in monster.rows().enumerate() {
        if !match_start_row(&image.row(y + dy)[x..], monster_row) {
            return false;
        }
    }
    true
}

fn match_start_row(image_row: &[u8], monster_row: &[u8]) -> bool {
    for (x, &monster_pixel) in monster_row.iter().enumerate() {
        if monster_pixel == b'#' && image_row[x] != b'#' {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transforms() {
        let show = |m: &Matrix<usize>| {
            println!();
            for row in m.rows() {
                println!("{:?}", row);
            }
        };

        let m = Matrix::new_with_fn(|x, y| 10 * y + x, 2, 3);
        show(&m);

        let mut t = Transform::IDENTITY;
        t.vertical_flip();
        for _ in 0..4 {
            show(&t.apply(&m));
            t.rotate_cw();
        }
    }
}
