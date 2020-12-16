use crate::data::{Data, ParseBytes, TryFromBytes};
use crate::iter_utils::IterUtils;
use crate::parser::Parser;
use itertools::Itertools;
use std::collections::BTreeSet;
use std::ops::RangeInclusive;

#[derive(Debug, Clone)]
struct Field {
    label: String,
    range_a: RangeInclusive<i32>,
    range_b: RangeInclusive<i32>,
}

#[derive(Debug, Clone)]
struct Ticket {
    values: Vec<i32>,
}

#[derive(Debug, Clone)]
struct Matching {
    /// Store column candidates by field
    columns_by_field: Vec<BTreeSet<u8>>,
    /// Store field candidates by column
    fields_by_column: Vec<BTreeSet<u8>>,
}

pub fn solve() -> (i64, i64) {
    let data = Data::read(16);

    let mut paragraphs = data.paragraphs();
    let first_paragraph = paragraphs.next().unwrap();
    let fields: Vec<Field> = first_paragraph.lines().parsed().collect();

    let second_paragraph = paragraphs.next().unwrap();
    let my_ticket: Ticket = second_paragraph.lines().nth(1).unwrap().parse_bytes();

    let mut part_1 = 0;
    let mut matching = Matching::new(fields.len());

    let third_paragraph = paragraphs.next().unwrap();
    for nearby_ticket in third_paragraph.lines().skip(1).parsed::<Ticket>() {
        let mut invalid_ticket = false;
        for &value in &nearby_ticket.values {
            let is_valid = fields.iter().any(|rule| rule.is_valid(value));
            if !is_valid {
                invalid_ticket = true;
                part_1 += value;
            }
        }

        if !invalid_ticket {
            matching.update_with_valid_ticket(&fields, &nearby_ticket);
        }
    }

    matching.propagate();

    let mut part_2: i64 = 1;
    for (field_id, field) in fields.iter().enumerate() {
        if field.label.starts_with("departure") {
            let column = matching.column_for_field(field_id as u8).unwrap();
            part_2 *= my_ticket.values[column as usize] as i64;
        }
    }

    (part_1 as i64, part_2)
}

impl TryFromBytes for Field {
    fn try_from_bytes(mut bytes: &[u8]) -> Option<Self> {
        let label = bytes.consume_until(b':');
        bytes.consume_byte();
        let min_a: i32 = bytes.consume_until(b'-').parse_bytes();
        let max_a: i32 = bytes.consume_until(b' ').parse_bytes();
        bytes.consume_prefix(b"or ");
        let min_b: i32 = bytes.consume_until(b'-').parse_bytes();
        let max_b: i32 = bytes.parse_bytes();
        Some(Field {
            label: std::str::from_utf8(label).unwrap().to_owned(),
            range_a: min_a..=max_a,
            range_b: min_b..=max_b,
        })
    }
}

impl TryFromBytes for Ticket {
    fn try_from_bytes(bytes: &[u8]) -> Option<Self> {
        Some(Ticket {
            values: bytes.split_byte(b',', true).parsed::<i32>().collect(),
        })
    }
}

impl Field {
    fn is_valid(&self, value: i32) -> bool {
        self.range_a.contains(&value) || self.range_b.contains(&value)
    }
}

impl Matching {
    fn new(num_fields: usize) -> Self {
        let all: BTreeSet<_> = (0..num_fields as u8).collect();
        Matching {
            columns_by_field: vec![all.clone(); num_fields],
            fields_by_column: vec![all; num_fields],
        }
    }

    fn update_with_valid_ticket(&mut self, fields: &[Field], ticket: &Ticket) {
        for (column, &value) in ticket.values.iter().enumerate() {
            for (field_id, field) in fields.iter().enumerate() {
                if !field.is_valid(value) {
                    self.columns_by_field[field_id].remove(&(column as u8));
                    self.fields_by_column[column].remove(&(field_id as u8));
                }
            }
        }
    }

    fn propagate(&mut self) {
        let mut open_fields = (0..self.columns_by_field.len() as u8).collect_vec();
        let mut open_columns = (0..self.fields_by_column.len() as u8).collect_vec();

        while !open_fields.is_empty() || !open_columns.is_empty() {
            open_fields.retain(|&open_field| match self.column_for_field(open_field) {
                None => true,
                Some(matched_column) => {
                    self.set_answer(matched_column, open_field);
                    false
                }
            });

            open_columns.retain(|&open_column| match self.field_for_column(open_column) {
                None => true,
                Some(matched_field) => {
                    self.set_answer(open_column, matched_field);
                    false
                }
            });
        }
    }

    fn set_answer(&mut self, matched_column: u8, matched_field: u8) {
        for (column, fields) in self.fields_by_column.iter_mut().enumerate() {
            if column != matched_column as usize {
                fields.remove(&matched_field);
            }
        }

        for (field, columns) in self.columns_by_field.iter_mut().enumerate() {
            if field != matched_field as usize {
                columns.remove(&matched_column);
            }
        }
    }

    fn column_for_field(&self, field: u8) -> Option<u8> {
        let columns = &self.columns_by_field[field as usize];
        if columns.len() == 1 {
            Some(*columns.iter().next().unwrap())
        } else {
            None
        }
    }

    fn field_for_column(&self, column: u8) -> Option<u8> {
        let fields = &self.fields_by_column[column as usize];
        if fields.len() == 1 {
            Some(*fields.iter().next().unwrap())
        } else {
            None
        }
    }
}
