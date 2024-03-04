
pub mod vigenere {
  
const ALPHABETS :&str = "abcdefghijklmnopqrstuvwxyz";

//Convert vectors to strings based on const -> ALPHABETS indexes
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
///convert string to vector of indexes mapped to const-> ALPHABETS 
fn arrayify(q: &str) -> Vec<i32> {
    q.to_lowercase()
    .chars()
        .map(|c| {
            if let Some(index) = ALPHABETS.find(c) {
                index as i32
            } else {
                -1
            }
        })
        .collect()
}

pub fn encode(data: &str, key: &str) -> String {
   //key vectors
    let _key = arrayify(&key);
    //hack to work with any key Length 
    let _key_expansion: Vec<_> = (0..data.len()).map(|i| _key[i % _key.len()]).collect();
//mapping each characters
    let res: Vec<_> = data
        .chars()
        .enumerate()
        .map(|(i, c)| {
            if c == ' ' {
                return -1;
            }
            //calculation VIGENERE ALGORITHM 
            (c as i32 - 'a' as i32 + _key_expansion[i % _key_expansion.len()]) % ALPHABETS.len() as i32
        })
        .collect();
///return result 
    stringify(&res)
}

pub fn decode(data: &str, key: &str) -> String {
   ///convert key to vector 
    let _key = arrayify(key);
    //hack to work with all key Length 
    let _key_expansion: Vec<_> = (0..data.len()).map(|i| _key[i % _key.len()]).collect();
///map each character against it's index in const-> ALPHABET
    let res: Vec<_> = data
        .chars()
        .enumerate()
        .map(|(i, c)| {
            if c == ' ' {
                return -1;
            }
            //calculation 
            (c as i32 - 'a' as i32 - _key_expansion[i % _key_expansion.len()] + ALPHABETS.len() as i32) % ALPHABETS.len() as i32
        })
        .collect();
///return the result 
    stringify(&res)
  }
};