pub mod bloom_filter;
pub mod utils;

use bloom_filter::BloomFilter;

fn main() {
    let mut bf = BloomFilter::new(20, 0.05);
    let word_present = vec![
        "A", "quick", "brown", "Fox", "jUmps", "over", "A", "lazy", "DOG",
    ];

    for word in word_present.iter() {
        bf.insert(word);
    }

    let test = [
        "never", "brown", "hello", "die", "what??", "Fox", "GOD", "over",
    ];

    for w in test.iter() {
        if bf.lookup(w) {
            println!("{} is probably present", w);
        } else {
            println!("{} is definitely absent....", w);
        }
    }

    println!();

    bf.to_file("bf.bin").unwrap();
    let new_bf = BloomFilter::from_file("bf.bin").unwrap();
    assert_eq!(bf, new_bf);

    for w in test.iter() {
        if new_bf.lookup(w) {
            println!("{} is probably present", w);
        } else {
            println!("{} is definitely absent....", w);
        }
    }
}
