
use serde_json::{
    json::{Value,Json}
};

pub mod vigenere{
const ALPHABET:&str = "abcdefghijklmnopqrstuvwxyz";
pub fn gen_random (len:i32)-> &'a str{
    let mut range = rand::thread_rng();
    let mut res = String::new();
    for _ in 0..=len {
        let index = range.gen_range(0..ALPHABET.len());
        res.push(ALPHABET.chars().nth(index).unwrap());
    }
    res.as_str()
}

}

fn stringify(arr: &Vec<i32>) -> String {
    arr.iter()
        .map(|&c| {
            if c >= 0 && (c as usize) < ALPH.len() {
                ALPH.chars().nth(c as usize).unwrap().to_string()
            } else {
                " ".to_string()
            }
        })
        .collect()
}

fn arrayify(q: &str) -> Vec<i32> {
    q.to_lowercase()
    .chars()
        .map(|c| {
            if let Some(index) = ALPH.find(c) {
                index as i32
            } else {
                -1
            }
        })
        .collect()
}

pub fn encode(data: &str, key: &str) -> String {
    let d_1 = arrayify(&data);
    let k_1 = arrayify(&key);
    let k_2: Vec<_> = (0..data.len()).map(|i| k_1[i % k_1.len()]).collect();

    let res: Vec<_> = data
        .chars()
        .enumerate()
        .map(|(i, c)| {
            if c == ' ' {
                return -1;
            }
            ((c as i32 - 'a' as i32 + k_2[i % k_1.len()]) % ALPH.len() as i32)
        })
        .collect();

    stringify(&res)
}

pub fn decode(data: &str, key: &str) -> String {
    let d_1 = arrayify(&data);
    let k_1 = arrayify(&key);
    let k_2: Vec<_> = (0..data.len()).map(|i| k_1[i % k_1.len()]).collect();

    let res: Vec<_> = data
        .chars()
        .enumerate()
        .map(|(i, c)| {
            if c == ' ' {
                return -1;
            }
            ((c as i32 - 'a' as i32 - k_2[i % k_1.len()] + ALPH.len() as i32) % ALPH.len() as i32)
        })
        .collect();

    stringify(&res)
}
};