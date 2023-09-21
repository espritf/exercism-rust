use std::{collections::HashMap, thread, sync::mpsc};

pub fn frequency(input: &[&str], worker_count: usize) -> HashMap<char, usize> {
    let mut result: HashMap<char, usize> = HashMap::with_capacity(26);

    let input = input.join("").chars().collect::<Vec<char>>();
    if input.is_empty() {
        return result;
    }

    let chunk_size = input.len() / worker_count + 1;
    let (tx, rx) = mpsc::channel::<HashMap<char, usize>>();

    for str in input.chunks(chunk_size) {
        let str = str.to_owned();
        let tx = tx.clone();
        thread::spawn(move || {

            let mut counter = HashMap::with_capacity(26);
            let filtered = str
                .iter()
                .filter(|ch| ch.is_alphabetic())
                .map(|ch| ch.to_ascii_lowercase());
            for char in filtered {
                *counter.entry(char).or_insert(0) += 1;
            }

            tx.send(counter).unwrap();
        });
    }

    drop(tx);

    for counter in rx {
        for (&ch, &count) in counter.iter() {
            *result.entry(ch).or_insert(0) += count;
        }
    }

    result
}
