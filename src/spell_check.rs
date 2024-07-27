use std::{fs::File, path::Path};

use crate::{bk_tree::BKTree, bloom_filter::BloomFilter, dictionary::Dictionary};

#[readonly::make]
pub struct SpellCheck {
    bk_tree: BKTree,
    bloom_filter: BloomFilter,
}

impl SpellCheck {
    pub fn new(bk_tree_path: &str, bloom_filter_path: &str, dictionary_path: &str, alphabet_length: u16) -> Self {
        let bk_tree: BKTree;
        let bloom_filter: BloomFilter;
        let mut dictionary: Option<Dictionary> = None;
        
        if Path::new(bk_tree_path).exists() {
            bk_tree = BKTree::from(File::open(bk_tree_path).expect("Failed to open BKTree file"));
        }
        else {
            if dictionary.is_none() {
                dictionary = Some(Dictionary::from((File::open(dictionary_path).expect("Failed to open dictionary file"), alphabet_length)));
            }
            bk_tree = BKTree::from(dictionary.as_ref().unwrap());
            bk_tree.to_file(bk_tree_path).expect("Failed to write BKTree to file");
        }

        if Path::new(bloom_filter_path).exists() {
            bloom_filter = BloomFilter::from(File::open(bloom_filter_path).expect("Failed to open BloomFilter file"));
        }
        else {
            if dictionary.is_none() {
                dictionary = Some(Dictionary::from((File::open(dictionary_path).expect("Failed to open dictionary file"), alphabet_length)));
            }
            bloom_filter = BloomFilter::from(dictionary.as_ref().unwrap());
            bloom_filter.to_file(bloom_filter_path).expect("Failed to write BloomFilter to file");
        }

        Self {
            bk_tree,
            bloom_filter,
        }
    }

    pub fn run(&self) {
        println!("Spell Sweep");
    }
}

#[cfg(test)]
mod tests {
    use super::SpellCheck;

    #[test]
    fn test_new() {
        let bk_tree_path: &str = "bk_tree_test.bin";
        let bloom_filter_path: &str = "bloom_filter_test.bin";
        let dictionary_path: &str = "dictionary.txt";
        let alphabet_length: u16 = 255;

        let spell_check: SpellCheck = SpellCheck::new(bk_tree_path, bloom_filter_path, dictionary_path, alphabet_length);

        assert_ne!(spell_check.bk_tree.tree.len(), 0);
        assert_eq!(spell_check.bk_tree.alphabet_length, alphabet_length);

        let words_absent = ["clesr", "erroe", "hel;", "rivee", "jokeq", "fathep"];
        for word in words_absent {
            assert_eq!(spell_check.bloom_filter.lookup(word), false);
        }

        std::fs::remove_file(bk_tree_path).expect("Failed to remove BKTree file");
        std::fs::remove_file(bloom_filter_path).expect("Failed to remove BloomFilter file");
    }
}
