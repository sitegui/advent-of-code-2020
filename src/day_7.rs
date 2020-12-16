use crate::data::{Data, ParseBytes};
use crate::parser::Parser;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Default)]
struct Rules<'a> {
    /// Map inside color -> "is contained in" outside colors
    contained_in: BTreeMap<&'a [u8], Vec<&'a [u8]>>,
    /// Map outside color -> "contains" `N` of each inside colors
    contains: BTreeMap<&'a [u8], Vec<(u8, &'a [u8])>>,
}

impl<'a> Rules<'a> {
    fn insert_rule(&mut self, outside_color: &'a [u8], num: u8, inside_color: &'a [u8]) {
        self.contained_in
            .entry(inside_color)
            .or_default()
            .push(outside_color);

        self.contains
            .entry(outside_color)
            .or_default()
            .push((num, inside_color));
    }

    fn direct_containers(&self, inside_color: &'a [u8]) -> &[&'a [u8]] {
        self.contained_in
            .get(&inside_color)
            .map(|v| v.as_slice())
            .unwrap_or_default()
    }

    fn total_bags_inside(&self, outside_color: &'a [u8]) -> i64 {
        self.contains
            .get(&outside_color)
            .map(|contained| {
                contained
                    .iter()
                    .map(|&(num, inside_color)| {
                        num as i64 * (1 + self.total_bags_inside(inside_color))
                    })
                    .sum()
            })
            .unwrap_or(0)
    }
}

pub fn solve() -> (i64, i64) {
    let data = Data::read(7);

    let mut rules = Rules::default();

    // Parse the rules
    for mut line in data.lines() {
        let outside_color = line.consume_words(2);
        assert_eq!(line.consume_words(2), b"bags contain");

        while let Some(mut contained) = line.try_consume_words(4) {
            let num: u8 = contained.consume_words(1).parse_bytes();
            let inside_color = contained.consume_words(2);
            assert!(matches!(
                contained.consume_words(1),
                b"bags," | b"bags." | b"bag," | b"bag."
            ));
            assert!(contained.is_empty());

            rules.insert_rule(outside_color, num, inside_color);
        }

        assert!(matches!(
            line.try_consume_words(3),
            None | Some(b"no other bags.")
        ));

        assert!(line.is_empty());
    }

    // Detect the possible outside colors
    const INITIAL_COLOR: &[u8] = b"shiny gold";
    let mut pending_insides = vec![INITIAL_COLOR];
    let mut considered_outside = BTreeSet::new();
    while let Some(inside) = pending_insides.pop() {
        for &container in rules.direct_containers(inside) {
            if considered_outside.insert(container) {
                pending_insides.push(container);
            }
        }
    }

    (
        considered_outside.len() as i64,
        rules.total_bags_inside(INITIAL_COLOR),
    )
}
