use std::{collections::HashMap, u8, cmp::Ordering};

/// Given a list of poker hands, return a list of those hands which win.
///
/// Note the type signature: this function should return _the same_ reference to
/// the winning hand(s) as were passed in, not reconstructed strings which happen to be equal.
pub fn winning_hands<'a>(hands: &[&'a str]) -> Vec<&'a str> {
    if hands.len() == 1 {
        return hands.to_vec();
    }

    let hands: Vec<Hand> = hands.iter().map(|s| Hand::from(s)).collect();

    let max = hands.iter().max().unwrap();
    hands
        .iter()
        .filter(|h| h.rank == max.rank)
        .map(|h| h.str)
        .collect()
}

#[derive(Eq, Debug)]
struct Hand<'a> {
    str: &'a str,
    rank: u64,
}

impl<'a> Hand<'a> {
    fn from(str: &'a str) -> Self {
        Self {
            str,
            rank: Cards::from_str(str).calc_rank(),
        }
    }
}

impl<'a> PartialEq for Hand<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.rank == other.rank
    }
}

impl<'a> Ord for Hand<'a> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.rank.cmp(&other.rank)
    }
}

impl<'a> PartialOrd for Hand<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[rustfmt::skip]
const RANKS: [&str; 13] = [ "2", "3", "4", "5", "6", "7", "8", "9", "10", "J", "Q", "K", "A"];
const BASE: u64 = 15;

struct Cards {
    kinds: HashMap<u8, u8>,
    flush: bool,
}

impl Cards {
    fn from_str(cards: &str) -> Self {
        let cards: Vec<_> = cards.split(' ').map(|c| c.split_at(c.len() - 1)).collect();

        let first = cards.get(0).unwrap();
        let flush: bool = cards.iter().all(|&(_, i)| i == first.1);

        let mut kinds: HashMap<u8, u8> = HashMap::new();
        for (r, _) in &cards {
            let v: u8 = RANKS
                .iter()
                .enumerate()
                .find(|(_, &v)| v == *r)
                .expect("unexpected card")
                .0 as u8;

            *kinds.entry(v + 1).or_insert(0) += 1;
        }

        Self { kinds, flush }
    }

    fn is_sequece(&self) -> bool {
        let mut kinds: Vec<&u8> = self.kinds.keys().collect();
        if kinds.len() != 5 {
            return false;
        }
        kinds.sort();
        // aces can start a straight (A 2 3 4 5)
        if kinds == vec![&1, &2, &3, &4, &13] {
            return true;
        }
        let min: u8 = **kinds.first().unwrap();
        kinds.iter().enumerate().all(|(k, &&v)| k as u8 == v - min)
    }

    fn get_kind_counts(&self) -> (u8, u8) {
        let (&fk, &fv) = self.kinds.iter().max_by_key(|i| i.1).unwrap();
        if fv == 1 {
            return (1, 1);
        }
        let (_, &sv) = self.kinds.iter().filter(|(&k, _)| k != fk).max_by_key(|i| i.1).unwrap();
        (fv, sv)
    }

    // 9 Straight flush - by suit & sequence
    // 8 Four of a kind - by kind
    // 7 Full house - by kind
    // 6 Flush - by suit
    // 5 Straight - by sequence
    // 4 Three of a kind - by kind
    // 3 Two pair - by kind
    // 2 One pair - by kind
    // 1 Highh card - by rate
    fn get_category(&self) -> u8 {
        let counts = self.get_kind_counts();
        let x = (self.flush, self.is_sequece(), counts.0, counts.1);
        match x {
            (true, true, 1, 1) => 9,
            (false, false, 4, _) => 8,
            (false, false, 3, 2) => 7,
            (true, false, _, _) => 6,
            (false, true, _, _) => 5,
            (false, false, 3, _) => 4,
            (false, false, 2, 2) => 3,
            (false, false, 2, 1) => 2,
            _ => 1,
        }
    }

    fn get_kinds_by_impact(&self) -> Vec<u8> {
        let mut kinds: Vec<(&u8, &u8)> = self.kinds.iter().collect();
        kinds.sort_by(|a, b| { 
            match b.1.cmp(a.1) {
                Ordering::Equal => b.0.cmp(a.0),
                ord @ _ => ord
            }
        });
        let kinds = kinds.iter().map(|e| *e.0).collect();
        // aces can start a straight (A 2 3 4 5)
        if kinds == vec![13, 4, 3, 2, 1] {
            return vec![4, 3, 2, 1, 0];
        }
        kinds
    }

    fn calc_rank(&self) -> u64 {
        let category = self.get_category();
        let kinds = self.get_kinds_by_impact();

        let mut rate: u64 = (category as u64) * BASE.pow(5);
        for (i, &r) in kinds.iter().enumerate() {
            let adder = (r as u64) * BASE.pow(4 - i as u32);
            rate += adder;
        }
        rate
    }
}
