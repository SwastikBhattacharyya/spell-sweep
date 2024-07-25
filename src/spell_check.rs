use std::path::Path;

use crate::{bk_tree::BKTree, dictionary::Dictionary};

#[readonly::make]
pub struct SpellCheck {
    pub bk_tree: BKTree,
}

impl SpellCheck {
    pub fn new() -> Self {
       Self { bk_tree: SpellCheck::get_bk_tree() } 
    }

    fn get_bk_tree() -> BKTree {
        let mut bk_tree: BKTree;
        if Path::new("bk_tree.bin").exists() {
            bk_tree = match BKTree::from_file("bk_tree.bin") {
                Ok(t) => t,
                Err(e) => panic!("Failed to load BKTree from file: {}", e)
            };
        }
        else {
            let dictionary = SpellCheck::get_dictionary();
            bk_tree = BKTree::new(
                dictionary.max_word_length as u16,
                dictionary.alphabet_length as u16,
                dictionary.words.len()
            );
            for word in dictionary.words.iter() {
                bk_tree.add(word.clone());
            }
        }
        match bk_tree.to_file("bk_tree.bin") {
            Ok(_) => {},
            Err(_) => panic!("Could not save BK Tree to file.")
        }
        bk_tree
    }

    fn get_dictionary() -> Dictionary {
        let dictionary: Dictionary = match Dictionary::from_file("dictionary.txt", 255) {
            Some(dictionary) => dictionary,
            None => panic!("Failed to load dictionary from file")
        };

        dictionary
    }
}
