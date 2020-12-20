use crate::data::{Data, ParseBytes, TryFromBytes};
use crate::iter_utils::IterUtils;
use crate::parser::Parser;
use itertools::Itertools;
use std::collections::BTreeMap;

type RuleId = u8;

#[derive(Debug, Clone, Eq, PartialEq)]
enum Rule {
    Char(u8),
    Sequence(Vec<RuleId>),
    EitherSequence {
        either: Vec<RuleId>,
        or: Vec<RuleId>,
    },
}

pub fn solve() -> (i64, i64) {
    let data = Data::read(19);

    let mut paragraphs = data.paragraphs();
    let rules: BTreeMap<_, _> = paragraphs
        .next()
        .unwrap()
        .lines()
        .map(|mut line| {
            let rule_id: RuleId = line.consume_until(b':').parse_bytes();
            line.consume_prefix(b" ");
            (rule_id, line.parse_bytes::<Rule>())
        })
        .collect();

    // List all accepted values by rules 31 and 42
    let matches_31 = rules.get(&31).unwrap().list_matches(&rules);
    let matches_42 = rules.get(&42).unwrap().list_matches(&rules);

    // Assert that the format of rules 0, 8 and 11
    assert_eq!(rules.get(&0), Some(&Rule::Sequence(vec![8, 11])));
    assert_eq!(rules.get(&8), Some(&Rule::Sequence(vec![42])));
    assert_eq!(rules.get(&11), Some(&Rule::Sequence(vec![42, 31])));

    // This means that the rule 0 can be simply written as "0: 42 42 31".
    // With this simple format, validating part 1 is straight forward now:
    let texts = paragraphs.next().unwrap();
    let mut part_1 = 0;
    for text in texts.lines() {
        if is_valid_p1(&matches_31, &matches_42, text) {
            part_1 += 1;
        }
    }

    // For part 2, with the edited rules 8 and 11, the rule 0 then becomes:
    // "0: (42 | 42 8) (42 31 | 42 11 31)", that can be described as:
    // "0: x*42 y*42 y*31", where `x` and `y` are positive numbers indicating the number of
    // repetitions of each rule.

    let mut part_2 = 0;
    for text in texts.lines() {
        if is_valid_p2(&matches_31, &matches_42, text) {
            part_2 += 1;
        }
    }

    (part_1, part_2)
}

fn is_valid_p1(matches_31: &[Vec<u8>], matches_42: &[Vec<u8>], text: &[u8]) -> bool {
    let text_tail = cut_prefix(text, matches_42).next().and_then(|text| {
        cut_prefix(text, matches_42)
            .next()
            .and_then(|text| cut_prefix(text, matches_31).next())
    });

    text_tail == Some(&[])
}

fn is_valid_p2(matches_31: &[Vec<u8>], matches_42: &[Vec<u8>], text: &[u8]) -> bool {
    // It's valid only if after taking one "42" it is either:
    // - valid under rule 11
    // - valid under rule 0
    for candidate in cut_prefix(text, matches_42) {
        if is_valid_p2_rule_11(matches_31, matches_42, candidate)
            || is_valid_p2(matches_31, matches_42, candidate)
        {
            return true;
        }
    }
    false
}

fn is_valid_p2_rule_11(matches_31: &[Vec<u8>], matches_42: &[Vec<u8>], text: &[u8]) -> bool {
    // It's valid only if after taking one "42" prefix and one "31" suffix it is either:
    // - empty
    // - valid under rule 11
    for pre_candidate in cut_prefix(text, matches_42) {
        for candidate in cut_suffix(pre_candidate, matches_31) {
            if candidate.is_empty() || is_valid_p2_rule_11(matches_31, matches_42, candidate) {
                return true;
            }
        }
    }
    false
}

impl TryFromBytes for Rule {
    fn try_from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes[0] == b'"' {
            assert_eq!(bytes.len(), 3);
            assert_eq!(bytes[2], b'"');
            Some(Rule::Char(bytes[1]))
        } else if bytes.contains(&b'|') {
            let mut sequences = bytes.split_bytes(b" | ", true).map(parse_sequence);
            let either = sequences.next().unwrap();
            let or = sequences.next().unwrap();
            assert_eq!(sequences.next(), None);
            Some(Rule::EitherSequence { either, or })
        } else {
            Some(Rule::Sequence(parse_sequence(bytes)))
        }
    }
}

fn parse_sequence(bytes: &[u8]) -> Vec<RuleId> {
    bytes.split_byte(b' ', true).parsed().collect()
}

fn cut_prefix<'a>(text: &'a [u8], prefixes: &'a [Vec<u8>]) -> impl Iterator<Item = &'a [u8]> {
    prefixes
        .iter()
        .filter(move |prefix| text.starts_with(prefix))
        .map(move |prefix| &text[prefix.len()..])
}

fn cut_suffix<'a>(text: &'a [u8], suffixes: &'a [Vec<u8>]) -> impl Iterator<Item = &'a [u8]> {
    suffixes
        .iter()
        .filter(move |suffix| text.ends_with(suffix))
        .map(move |suffix| &text[..text.len() - suffix.len()])
}

impl Rule {
    fn list_matches(&self, rules: &BTreeMap<RuleId, Rule>) -> Vec<Vec<u8>> {
        match self {
            &Rule::Char(c) => vec![vec![c]],
            Rule::Sequence(sequence) => Self::list_sequence(sequence, rules),
            Rule::EitherSequence { either, or } => {
                let mut all_matches = Self::list_sequence(either, rules);
                all_matches.extend(Self::list_sequence(or, rules));
                all_matches
            }
        }
    }

    fn list_sequence(sequence: &[u8], rules: &BTreeMap<RuleId, Rule>) -> Vec<Vec<u8>> {
        let matches_per_item = sequence
            .iter()
            .map(|rule_id| rules.get(rule_id).unwrap().list_matches(rules))
            .collect_vec();

        matches_per_item
            .iter()
            .multi_cartesian_product()
            .map(|product| {
                product.into_iter().fold(vec![], |mut base, item| {
                    base.extend(item);
                    base
                })
            })
            .collect()
    }
}
