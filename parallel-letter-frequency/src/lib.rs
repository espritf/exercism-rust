use std::{
    collections::HashMap,
    str::FromStr,
    sync::{mpsc, Arc, Mutex},
    thread,
};

pub fn frequency(input: &[&str], worker_count: usize) -> HashMap<char, usize> {
    assert!(worker_count > 0);

    let mut result: HashMap<char, usize> = HashMap::new();
    if input.is_empty() {
        return result;
    }

    let (pool_tx, pool_rx) = mpsc::channel::<String>();
    let (res_tx, res_rx) = mpsc::channel();

    let pool_rx = Arc::new(Mutex::new(pool_rx));
    for _ in 1..=worker_count {
        let pool_rx = pool_rx.clone();
        let res_tx = res_tx.clone();
        thread::spawn(move || {
            loop {
                let str = match pool_rx.lock().unwrap().recv() {
                    Ok(v) => v,
                    Err(_) => break,
                };

                let mut cc: HashMap<char, usize> = HashMap::new();
                for c in str.chars().filter(|c| c.is_alphabetic()) {
                    *cc.entry(c.to_ascii_lowercase()).or_insert(0) += 1;
                }

                res_tx.send(cc).unwrap();
            }
        });
    }

    for line in input {
        pool_tx.send(String::from_str(line).unwrap()).unwrap();
    }

    drop(pool_tx);
    drop(res_tx);

    for received in res_rx {
        for (k, v) in received {
            *result.entry(k).or_insert(0) += v;
        }
    }

    result
}
