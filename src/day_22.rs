use crate::data::{Data, ParseBytes, TryFromBytes};
use crate::iter_utils::IterUtils;
use crate::parser::Parser;
use std::cmp::Ordering;
use std::collections::BTreeSet;
use std::fmt;

const NUM_CARDS: usize = 50;
type Card = u8;
const NO_CARD: Card = Card::MAX;

#[derive(Clone, Copy)]
struct Deck {
    cards: [Card; NUM_CARDS],
    /// The index of the card on the top.
    /// 0 <= top <= NUM_CARDS
    top: usize,
    /// The index of the card **after** the bottom.
    /// top <= bottom <= NUM_CARDS
    bottom: usize,
}

#[derive(Clone, Copy)]
struct Game {
    level: u8,
    player_1: Deck,
    player_2: Deck,
}

#[derive(Debug, Copy, Clone)]
enum Player {
    Player1,
    Player2,
}

#[derive(Copy, Clone)]
struct State([Card; NUM_CARDS + 1]);

pub fn solve() -> (i64, i64) {
    let data = Data::read(22);

    let mut game_p1: Game = data.bytes().parse_bytes();
    let mut game_p2 = game_p1;

    let winner = game_p1.play_game_p1();
    let part_1 = game_p1.deck(winner).score();

    let winner = game_p2.play_game_p2();
    let part_2 = game_p2.deck(winner).score();

    (part_1, part_2)
}

impl TryFromBytes for Deck {
    fn try_from_bytes(bytes: &[u8]) -> Option<Self> {
        let mut deck = Deck::new();
        for card in bytes.lines().skip(1).parsed() {
            deck.add_one_bottom(card);
        }
        Some(deck)
    }
}

impl TryFromBytes for Game {
    fn try_from_bytes(bytes: &[u8]) -> Option<Self> {
        let mut paragraphs = bytes.paragraphs();
        let player_1 = paragraphs.next().unwrap().parse_bytes();
        let player_2 = paragraphs.next().unwrap().parse_bytes();
        Some(Game {
            level: 1,
            player_1,
            player_2,
        })
    }
}

impl Deck {
    fn new() -> Self {
        Deck {
            cards: [NO_CARD; NUM_CARDS],
            top: 0,
            bottom: 0,
        }
    }

    fn draw_top(&mut self) -> Card {
        let card = self.cards[self.top];
        debug_assert_ne!(card, NO_CARD);
        self.top += 1;
        card
    }

    fn add_one_bottom(&mut self, card: Card) {
        self.cards[self.bottom] = card;
        self.bottom += 1;
    }

    fn add_two_bottom(&mut self, card_a: Card, card_b: Card) {
        if self.bottom > NUM_CARDS - 2 {
            // No more tail space: remove head space
            self.cards.copy_within(self.top..self.bottom, 0);
            self.bottom -= self.top;
            self.top = 0;
        }

        self.cards[self.bottom] = card_a;
        self.cards[self.bottom + 1] = card_b;
        self.bottom += 2;
    }

    fn is_empty(&self) -> bool {
        self.top == self.bottom
    }

    fn len(&self) -> usize {
        self.bottom - self.top
    }

    fn score(&self) -> i64 {
        self.as_slice()
            .iter()
            .rev()
            .enumerate()
            .map(|(index, &card)| card as i64 * (index + 1) as i64)
            .sum()
    }

    fn sub_deck(&self, size: usize) -> Deck {
        let mut deck = Deck::new();
        deck.bottom = size;
        deck.cards[..size].copy_from_slice(&self.cards[self.top..self.top + size]);
        deck
    }

    fn as_slice(&self) -> &[Card] {
        &self.cards[self.top..self.bottom]
    }
}

impl Game {
    fn deck(&self, player: Player) -> &Deck {
        match player {
            Player::Player1 => &self.player_1,
            Player::Player2 => &self.player_2,
        }
    }

    fn play_round_p1(&mut self) {
        let card_1 = self.player_1.draw_top();
        let card_2 = self.player_2.draw_top();

        if card_1 > card_2 {
            self.player_1.add_two_bottom(card_1, card_2);
        } else {
            self.player_2.add_two_bottom(card_2, card_1);
        }
    }

    fn play_game_p1(&mut self) -> Player {
        loop {
            if let Some(winner) = self.winner() {
                return winner;
            }

            self.play_round_p1();
        }
    }

    fn winner(&self) -> Option<Player> {
        if self.player_1.is_empty() {
            Some(Player::Player2)
        } else if self.player_2.is_empty() {
            Some(Player::Player1)
        } else {
            None
        }
    }

    fn play_game_p2(&mut self) -> Player {
        let mut visited_states = BTreeSet::new();

        loop {
            if let Some(winner) = self.winner() {
                return winner;
            }

            let new_state = visited_states.insert(State::new(self));
            if !new_state {
                return Player::Player1;
            }

            let card_1 = self.player_1.draw_top();
            let card_2 = self.player_2.draw_top();

            match self.play_root_round_p2(card_1, card_2) {
                Player::Player1 => {
                    self.player_1.add_two_bottom(card_1, card_2);
                }
                Player::Player2 => {
                    self.player_2.add_two_bottom(card_2, card_1);
                }
            }
        }
    }

    fn play_root_round_p2(&mut self, card_1: Card, card_2: Card) -> Player {
        if self.player_1.len() < card_1 as usize || self.player_2.len() < card_2 as usize {
            if card_1 > card_2 {
                Player::Player1
            } else {
                Player::Player2
            }
        } else {
            let mut sub_game = Game {
                level: self.level + 1,
                player_1: self.player_1.sub_deck(card_1 as usize),
                player_2: self.player_2.sub_deck(card_2 as usize),
            };

            sub_game.play_game_p2()
        }
    }
}

impl State {
    fn new(game: &Game) -> Self {
        let mut state = State([NO_CARD; NUM_CARDS + 1]);

        let deck_1 = game.player_1.as_slice();
        let deck_2 = game.player_2.as_slice();

        state.0[..deck_1.len()].copy_from_slice(deck_1);
        // Offset by 1 to leave a "NO_CARD" between the decks.
        state.0[deck_1.len() + 1..deck_1.len() + 1 + deck_2.len()].copy_from_slice(deck_2);

        state
    }
}

impl Ord for State {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0[..].cmp(&other.0[..])
    }
}
impl PartialOrd for State {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for State {
    fn eq(&self, other: &Self) -> bool {
        self.0[..] == other.0[..]
    }
}

impl Eq for State {}

impl fmt::Debug for Game {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Player 1: {:?}", self.player_1.as_slice())?;
        writeln!(f, "Player 2: {:?}", self.player_2.as_slice())
    }
}
