use std::{fs::File, io::{BufReader, BufWriter, Read, Write}};
use rkyv::{AlignedVec, Archive, Deserialize, Serialize};

#[derive(Clone, Debug, Archive, Serialize, Deserialize, PartialEq)]
#[archive(compare(PartialEq), check_bytes)]
#[archive_attr(derive(Debug))]
#[readonly::make]
pub struct Node {
    pub word: String,
    pub next: Vec<Option<u32>>
}

impl Node {
    pub fn new(word: String, max_word_length: usize) -> Self {
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
            tree: vec![Node::new("".to_string(), max_word_length as usize); max_words],
            size: 0
        }
    }

    fn get_damerau_levenshtein_distance(&self, a: &str, b: &str) -> u8 {
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
                let k: usize = da[b.chars().nth(j - 1).expect("Couldn't get the (j - 1)th character of b") as usize];
                let l: usize = db;

                let a_char: char = a.chars().nth(i - 1).expect("Couldn't get the (i - 1)th character of a");
                let b_char: char = b.chars().nth(j - 1).expect("Couldn't get the (j - 1)th character of b");
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
            da[a.chars().nth(i - 1).expect("Couldn't get the (i - 1)th character of a") as usize] = i;
        }

        dp[m + 1][n + 1] as u8
    }

    pub fn add(&mut self, word: String) {
        let mut current: usize = 0;
        let mut distance: u8;

        loop {
            distance = self.get_damerau_levenshtein_distance(&self.tree[current].word, &word);

            match distance {
                0 => return,
                d if self.tree[current].next[d as usize].is_none() => {
                    if !self.tree[current].word.is_empty() { self.tree[current].next[d as usize] = Some(self.size); }
                    self.tree[self.size as usize].word = word;
                    self.size += 1;
                    return;
                },
                d => {
                    match self.tree[current].next[d as usize] {
                        Some(n) => current = n as usize,
                        None => return
                    }
                }
            }
        }
    }

    pub fn does_contain(&self, word: String) -> bool {
        let mut current: usize = 0;
        let mut distance: u8;

        loop {
            distance = self.get_damerau_levenshtein_distance(&self.tree[current].word, &word);

            match distance {
                0 => return true,
                d if self.tree[current].next[d as usize].is_none() => return false,
                d => {
                    match self.tree[current].next[d as usize] {
                        Some(n) => current = n as usize,
                        None => return false
                    }
                },
            }
        }
    }

    pub fn get_similar_words(&self, word: String, tolerance: u8) -> Vec<String> {
        let mut result: Vec<String> = Vec::new();
        let mut stack: Vec<usize> = vec![0];

        while !stack.is_empty() {
            let current: usize = stack.pop().expect("Failed to pop from stack");
            let distance: u8 = self.get_damerau_levenshtein_distance(&word, &self.tree[current].word);

            if distance <= tolerance {
                result.push(self.tree[current].word.clone());
            }

            let tolerance_start: u8 = if distance > tolerance { distance - tolerance } else { 1 };
            let tolerance_end: u8 = distance + tolerance;

            for i in tolerance_start..=tolerance_end {
                if self.tree[current].next[i as usize].is_some() {
                    stack.push(self.tree[current].next[i as usize].expect("Couldn't get next element from node in BKTree") as usize);
                }
            }
        }

        result
    }

    pub fn to_file(&self, file_path: &str) {
        let bytes: AlignedVec = rkyv::to_bytes::<_, 256>(self).expect("Failed to serialize BKTree");
        let file: File = std::fs::File::create(file_path).expect("Failed to create BKTree file"); 
        let mut writer: BufWriter<File> = BufWriter::new(file);
        writer.write_all(&bytes).expect("Failed to write BKTree to file")
    }
}

impl From<File> for BKTree {
    fn from(mut value: File) -> Self {
        let mut reader: BufReader<&mut File> = BufReader::new(&mut value);        
        
        let mut bytes: Vec<u8> = Vec::new();
        reader.read_to_end(&mut bytes).expect("Failed to read BKTree file");

        let tree: BKTree = rkyv::from_bytes::<BKTree>(&bytes).expect("Failed to deserialize BKTree");
        tree
    }
}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use super::BKTree;
    use super::super::dictionary::Dictionary;

    #[test]
    #[ignore = "Computationally expensive since it loads the entire dictionary"]
    fn test_from_dictionary() {
        let file: File = File::open("dictionary.txt").expect("Failed to open file"); 

        let dictionary: Dictionary = Dictionary::from((file, 255));
        let mut tree: BKTree = BKTree::new(dictionary.max_word_length as u16, dictionary.alphabet_length as u16, dictionary.words.len());

        assert_eq!(tree.alphabet_length, dictionary.alphabet_length);
        assert_eq!(tree.max_word_length, dictionary.max_word_length);

        for word in dictionary.words.iter() {
            tree.add(word.clone());
        }

        for word in dictionary.words.iter() {
            assert!(tree.does_contain(word.clone()));
        }
    }

    #[test]
    fn test_similar_words() {
        let mut tree: BKTree = BKTree::new(5, 255, 5);

        tree.add("hello".to_string());
        tree.add("world".to_string());
        tree.add("hella".to_string());
        tree.add("hell".to_string());
        tree.add("help".to_string());

        let similar_words: Vec<String> = tree.get_similar_words("hell".to_string(), 1);
        assert_eq!(similar_words.len(), 4);
        assert!(similar_words.contains(&"hello".to_string()));
        assert!(similar_words.contains(&"hella".to_string()));
        assert!(similar_words.contains(&"hell".to_string()));
        assert!(similar_words.contains(&"help".to_string()));
    }

    #[test]
    fn test_file_serialization() {
        let mut tree: BKTree = BKTree::new(5, 255, 5);

        tree.add("hello".to_string());
        tree.add("world".to_string());
        tree.add("hella".to_string());
        tree.add("hell".to_string());
        tree.add("help".to_string());

        tree.to_file("bk_tree_test.bin");

        let file: File = File::open("bk_tree_test.bin").expect("Failed to open BKTree file");
        let new_tree: BKTree = BKTree::from(file);
        
        assert_eq!(tree, new_tree);
        std::fs::remove_file("bk_tree_test.bin").expect("Failed to remove BKTree file");
    }

    #[test]
    #[ignore = "Computationally expensive since it loads the entire dictionary"]
    fn test_full_file_serialization() {
        let file: File = File::open("dictionary.txt").expect("Failed to open file");

        let dictionary: Dictionary = Dictionary::from((file, 255));
        let mut tree: BKTree = BKTree::new(dictionary.max_word_length as u16, dictionary.alphabet_length as u16, dictionary.words.len());

        for word in dictionary.words.iter() {
            tree.add(word.clone());
        }

        tree.to_file("bk_tree_full.bin");

        let file: File = File::open("bk_tree_full.bin").expect("Failed to open BKTree file");
        let new_tree: BKTree = BKTree::from(file);
 
        assert_eq!(tree, new_tree);
        std::fs::remove_file("bk_tree_full.bin").expect("Failed to remove BKTree file"); 
    }
}
