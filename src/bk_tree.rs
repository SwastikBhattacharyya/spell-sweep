use std::{error::Error, fs::File, io::{BufReader, BufWriter, Read, Write}, rc::Rc};
use rkyv::{AlignedVec, Archive, Deserialize, Serialize};

use crate::dictionary::Dictionary;

#[derive(Clone, Debug, Archive, Serialize, Deserialize, PartialEq)]
#[archive(compare(PartialEq), check_bytes)]
#[archive_attr(derive(Debug))]
#[readonly::make]
pub struct Node {
    pub word: NodeString,
    pub next: Vec<Option<u32>>
}

type NodeString = Option<Rc<String>>;

impl Node {
    pub fn new(word: NodeString, max_word_length: usize) -> Self {
        Self {
            word,
            next: vec![None; max_word_length + 1]
        }
    }
}

#[derive(Archive, Serialize, Deserialize, Debug, PartialEq)]
#[archive(compare(PartialEq), check_bytes)]
#[archive_attr(derive(Debug))]
#[readonly::make]
pub struct BKTree {
    pub max_word_length: u16,
    pub alphabet_length: u16,
    pub tree: Vec<Node>,
    pub size: u32
}

impl BKTree {
    pub fn new(max_word_length: u16, alphabet_length: u16, max_words: usize) -> Self {
        Self {
            max_word_length,
            alphabet_length,
            tree: vec![Node::new(None, max_word_length as usize); max_words],
            size: 0
        }
    }

    fn get_damerau_levenshtein_distance(&self, a: &str, b: &str) -> Result<u8, Box<dyn Error>> {
        let m: usize = a.len();
        let n: usize = b.len();

        let infinity: usize = m + n;
        let mut dp: Vec<Vec<usize>> = vec![vec![0; n + 2]; m + 2];
        dp[0][0] = infinity;

        for i in 0..=m {
            dp[i + 1][1] = i;
            dp[i + 1][0] = infinity;
        }

        for j in 0..=n {
            dp[1][j + 1] = j;
            dp[0][j + 1] = infinity;
        }

        let mut da: Vec<usize> = vec![0; self.alphabet_length as usize];

        for i in 1..=m {
            let mut db: usize = 0;
            for j in 1..=n {
                let k: usize = da[b.chars().nth(j - 1).ok_or("Couldn't get the (j - 1)th character of b")? as usize];
                let l: usize = db;

                let a_char: char = a.chars().nth(i - 1).ok_or("Couldn't get the (i - 1)th character of a")?;
                let b_char: char = b.chars().nth(j - 1).ok_or("Couldn't get the (j - 1)th character of b")?;
                let cost: usize = if a_char == b_char { 0 } else { 1 };
                db = if cost == 0 { j } else { db };

                dp[i + 1][j + 1] = std::cmp::min(
                    std::cmp::min(
                        dp[i][j] + cost,
                        dp[i + 1][j] + 1
                    ),
                    std::cmp::min(
                        dp[i][j + 1] + 1,
                        dp[k][l] + (i - k - 1) + 1 + (j - l - 1)
                    )
                );
            }
            da[a.chars().nth(i - 1).ok_or("Couldn't get the (i - 1)th character of a")? as usize] = i;
        }

        Ok(dp[m + 1][n + 1] as u8)
    }

    pub fn add(&mut self, word: Rc<String>) -> Result<(), Box<dyn Error>> {
        let mut current: usize = 0;
        let mut distance: u8;

        loop {
            let current_word: &str = match &self.tree[current].word {
                Some(w) => w,
                None => ""
            };
            distance = self.get_damerau_levenshtein_distance(&current_word, &word)?;

            match distance {
                0 => break,
                d => {
                    match self.tree[current].next[d as usize] {
                        Some(n) => current = n as usize,
                        None => {
                            if !self.tree[current].word.is_none() { self.tree[current].next[d as usize] = Some(self.size); }
                            self.tree[self.size as usize].word = Some(word);
                            self.size += 1;
                            break;
                        },
                    }
                } 
            }
        }

        Ok(())
    }

    pub fn does_contain(&self, word: &str) -> Result<bool, Box<dyn Error>> {
        let mut current: usize = 0;
        let mut distance: u8;

        loop {
            let current_word: &str = match &self.tree[current].word {
                Some(w) => w,
                None => ""
            };
            distance = self.get_damerau_levenshtein_distance(&current_word, &word)?;

            match distance {
                0 => return Ok(true),
                d if self.tree[current].next[d as usize].is_none() => return Ok(false),
                d => {
                    match self.tree[current].next[d as usize] {
                        Some(n) => current = n as usize,
                        None => return Ok(false)
                    }
                },
            }
        }
    }

    pub fn get_similar_words(&self, word: &str, tolerance: u8) -> Result<Vec<&str>, Box<dyn Error>> {
        let mut result: Vec<&str> = Vec::new();
        let mut stack: Vec<usize> = vec![0];

        while !stack.is_empty() {
            let current: usize = stack.pop()
                .ok_or("Couldn't get current element from stack")?;
            let current_word: &str = match &self.tree[current].word {
                Some(w) => w,
                None => ""
            };
            let distance: u8 = self.get_damerau_levenshtein_distance(&word, &current_word)?;

            if distance <= tolerance {
                result.push(current_word);
            }

            let tolerance_start: u8 = if distance > tolerance { distance - tolerance } else { 1 };
            let tolerance_end: u8 = distance + tolerance;

            for i in tolerance_start..=tolerance_end {
                if self.tree[current].next[i as usize].is_some() {
                    stack.push(self.tree[current].next[i as usize]
                            .ok_or("Couldn't push element to stack")? as usize);
                }
            }
        }

        Ok(result)
    }

    pub fn to_file(&self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let bytes: AlignedVec = rkyv::to_bytes::<_, 256>(self)?;
        let file: File = std::fs::File::create(file_path)?; 
        let mut writer: BufWriter<File> = BufWriter::new(file);
        writer.write_all(&bytes)?;

        Ok(())
    }
}

impl From<&Dictionary> for BKTree {
    fn from(value: &Dictionary) -> Self {
        let mut tree: BKTree = BKTree::new(value.max_word_length, value.alphabet_length, value.words.len());
    
        for word in value.words.iter() {
            tree.add(Rc::clone(&word)).expect("Failed to add word to tree");
        }
        
        tree
    }
}

impl From<File> for BKTree {
    fn from(mut value: File) -> Self {
        let mut reader: BufReader<&mut File> = BufReader::new(&mut value);        
        
        let mut bytes: Vec<u8> = Vec::new();
        reader.read_to_end(&mut bytes).expect("Failed to read to bytes");

        let tree: BKTree = rkyv::from_bytes::<BKTree>(&bytes).expect("Failed to deserialize BKTree");
        tree
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::fs::File;
    use std::rc::Rc;
    use super::BKTree;
    use super::super::dictionary::Dictionary;

    #[test]
    #[ignore = "Computationally expensive since it loads the entire dictionary"]
    fn test_from_dictionary() {
        let file: File = File::open("dictionary.txt").expect("Failed to open file"); 

        let dictionary: Dictionary = Dictionary::from((file, 255));
        let tree = BKTree::from(&dictionary);

        assert_eq!(tree.alphabet_length, dictionary.alphabet_length);
        assert_eq!(tree.max_word_length, dictionary.max_word_length);
        assert_eq!(tree.size, dictionary.words.len() as u32);

        for word in dictionary.words.iter() {
            assert!(tree.does_contain(&word).unwrap());
            assert_eq!(Rc::strong_count(&word), 2);
        }
    }

    #[test]
    fn test_similar_words() -> Result<(), Box<dyn Error>> {
        let mut tree: BKTree = BKTree::new(5, 255, 5);

        tree.add(Rc::new("hello".to_string()))?;
        tree.add(Rc::new("world".to_string()))?;
        tree.add(Rc::new("hella".to_string()))?;
        tree.add(Rc::new("hell".to_string()))?;
        tree.add(Rc::new("help".to_string()))?;

        let similar_words: Vec<&str> = tree.get_similar_words("hell", 1).expect("Failed to get similar words");
        assert_eq!(similar_words.len(), 4);
        assert!(similar_words.contains(&"hello"));
        assert!(similar_words.contains(&"hella"));
        assert!(similar_words.contains(&"hell"));
        assert!(similar_words.contains(&"help"));

        Ok(())
    }

    #[test]
    fn test_file_serialization() -> Result<(), Box<dyn Error>> {
        let mut tree: BKTree = BKTree::new(5, 255, 5);

        tree.add(Rc::new("hello".to_string()))?;
        tree.add(Rc::new("world".to_string()))?;
        tree.add(Rc::new("hella".to_string()))?;
        tree.add(Rc::new("hell".to_string()))?;
        tree.add(Rc::new("help".to_string()))?;

        tree.to_file("bk_tree_test.bin")?;

        let file: File = File::open("bk_tree_test.bin").expect("Failed to open BKTree file");
        let new_tree: BKTree = BKTree::from(file);
        
        assert_eq!(tree, new_tree);
        std::fs::remove_file("bk_tree_test.bin").expect("Failed to remove BKTree file");
        Ok(())
    }

    #[test]
    #[ignore = "Computationally expensive since it loads the entire dictionary"]
    fn test_full_file_serialization() -> Result<(), Box<dyn Error>> {
        let file: File = File::open("dictionary.txt").expect("Failed to open file");

        let dictionary: Dictionary = Dictionary::from((file, 255));
        let tree: BKTree = BKTree::from(&dictionary);

        tree.to_file("bk_tree_full.bin")?;

        let file: File = File::open("bk_tree_full.bin").expect("Failed to open BKTree file");
        let new_tree: BKTree = BKTree::from(file);

        assert_eq!(tree, new_tree);
        std::fs::remove_file("bk_tree_full.bin").expect("Failed to remove BKTree file");
        Ok(())
    }
}
