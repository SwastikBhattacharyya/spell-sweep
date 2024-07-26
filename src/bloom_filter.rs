use std::{
    error::Error,
    f32::consts::{E, LN_2},
    fs,
    io::{Read, Write},
    path::Path,
};

use rkyv::{AlignedVec, Archive, Deserialize, Serialize};

use crate::{dictionary::Dictionary, utils};

#[derive(Debug, Serialize, Deserialize, Archive, PartialEq)]
#[archive(compare(PartialEq), check_bytes)]
#[archive_attr(derive(Debug))]
#[readonly::make]
pub struct BloomFilter {
    pub fp_prob: f32,
    pub size: u64,
    pub hash_count: u32,
    pub bitarray: Vec<u8>,
}

impl BloomFilter {
    fn get_size(items_count: u32, fp_prob: f32) -> u64 {
        let a = (items_count as f32) * fp_prob.log(E);
        let b = LN_2.powi(2);

        return (-a / b).ceil() as u64;
    }

    fn get_hash_count(items_count: u32, fp_prob: f32) -> u32 {
        let a = Self::get_size(items_count, fp_prob) as f32 * LN_2;
        let b = items_count as f32;

        return (a / b).ceil() as u32;
    }

    pub fn new(items_count: u32, fp_prob: f32) -> BloomFilter {
        let size = Self::get_size(items_count, fp_prob);
        let hash_count = Self::get_hash_count(items_count, fp_prob);

        BloomFilter {
            fp_prob,
            size,
            hash_count,
            bitarray: vec![0; size as usize],
        }
    }

    pub fn insert(&mut self, target: &str) {
        for i in 0..self.hash_count {
            let digest = utils::hash_with_seed(target, i);
            let digest = digest % self.size;

            // self.bitarray.set(digest as usize, true);
            self.bitarray[digest as usize] = 1;
        }
    }

    pub fn lookup(&self, target: &str) -> bool {
        for i in 0..self.hash_count {
            let digest = utils::hash_with_seed(target, i);
            let digest = digest % self.size;

            if self.bitarray.get(digest as usize).unwrap() == &0 {
                return false;
            }
        }
        return true;
    }

    fn serialize(&self) -> Result<AlignedVec, Box<dyn Error>> {
        Ok(utils::serialize(self)?)
    }

    fn deserialize(bytes: &[u8]) -> Result<Self, Box<dyn Error>> {
        Ok(utils::deserialize::<Self>(bytes)?)
    }

    pub fn to_file(&self, path: &str) -> Result<(), Box<dyn Error>> {
        let bytes = self.serialize()?;
        let bytes = bytes.as_slice();

        let mut file = fs::File::create(Path::new(path))?;
        file.write_all(bytes)?;

        Ok(())
    }

    pub fn from_file(path: &str) -> Result<Self, Box<dyn Error>> {
        let mut buffer = Vec::<u8>::new();

        let mut file = fs::File::open(Path::new(path))?;
        file.read_to_end(&mut buffer)?;

        let bf = BloomFilter::deserialize(&buffer)?;
        Ok(bf)
    }
}

impl From<Vec<String>> for BloomFilter {
    fn from(value: Vec<String>) -> Self {
        let mut bf = Self::new(value.len() as u32, 0.01);
        for word in value {
            bf.insert(word.as_str());
        }
        bf
    }
}

impl From<&Dictionary> for BloomFilter {
    fn from(value: &Dictionary) -> Self {
        let mut bf = Self::new(value.words.len() as u32, 0.01);
        for word in value.words.iter() {
            bf.insert(word.as_str())
        }
        bf
    }
}

#[cfg(test)]
mod tests {
    use fs::File;

    use super::*;

    #[test]
    fn test_fp_prob() {
        let fp_prob = 0.01;
        let mut bf = BloomFilter::new(20, fp_prob);
        let word_present = [
            "A", "quick", "brown", "Fox", "jUmps", "over", "A", "lazy", "DOG",
        ];

        let word_absent = ["hello", "good", "what", "noooo", "truly", "never"];

        for word in word_present.iter() {
            bf.insert(word);
        }

        let test = [
            "good", "Fox", "over", "truly", "brown", "never", "what", "quick",
        ];

        for w in word_present.iter() {
            assert_eq!(bf.lookup(w), true);
        }

        let (mut true_pos, mut true_neg, mut false_pos) = (0, 0, 0);

        for w in test.iter() {
            if bf.lookup(w) {
                if word_present.contains(w) {
                    true_pos += 1;
                } else if word_absent.contains(w) {
                    false_pos += 1;
                }
            } else {
                true_neg += 1;
            }
        }

        assert_eq!(true_pos + true_neg + false_pos, test.len());
        assert!((false_pos as f32 / true_neg as f32) <= fp_prob);
    }

    #[test]
    fn test_ser_deser() {
        let mut bf = BloomFilter::new(20, 0.01);
        let word_present = vec![
            "A", "quick", "brown", "Fox", "jUmps", "over", "A", "lazy", "DOG",
        ];

        for word in word_present.iter() {
            bf.insert(word);
        }

        bf.to_file("bf.bin").unwrap();
        let new_bf = BloomFilter::from_file("bf.bin").unwrap();
        assert_eq!(bf, new_bf);
        std::fs::remove_file("bf.bin").expect("Failed to remove file");
    }

    #[test]
    fn test_from_vector() {
        let word_present = vec![
            "A", "quick", "brown", "Fox", "jUmps", "over", "A", "lazy", "DOG",
        ]
        .into_iter()
        .map(String::from)
        .collect::<Vec<String>>();

        let word_absent = ["hello", "good", "what", "noooo", "truly", "never"];

        let bf = BloomFilter::from(word_present.clone());

        let test = [
            "good", "Fox", "over", "truly", "brown", "never", "what", "quick",
        ];

        let (mut true_pos, mut true_neg, mut false_pos) = (0, 0, 0);

        for w in test.into_iter() {
            if bf.lookup(w) {
                if word_present.contains(&w.to_string()) {
                    true_pos += 1;
                } else if word_absent.contains(&w) {
                    false_pos += 1;
                }
            } else {
                true_neg += 1;
            }
        }

        assert_eq!(true_pos + true_neg + false_pos, test.len());
        assert!((false_pos as f32 / true_neg as f32) <= 0.01);
    }

    #[test]
    fn test_from_dictionary() {
        let file: File = File::open("dictionary.txt").expect("File not found");
        let dictionary: Dictionary = Dictionary::from((file, 255));
        
        let words_absent = ["clesr", "erroe", "hel;", "rivee", "jokeq", "fathep"];

        let bf = BloomFilter::from(&dictionary);

        for word in words_absent {
            assert_eq!(bf.lookup(word), false);
        }
    }
}
