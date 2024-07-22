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
            next: vec![None; max_word_length * 2]
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

    pub fn from_file(file_path: &str) -> std::io::Result<Self> {
        let mut file: File = std::fs::File::open(file_path)?;
        let mut reader: BufReader<&mut File> = BufReader::new(&mut file);        
        
        let mut bytes: Vec<u8> = Vec::new();
        reader.read_to_end(&mut bytes)?;

        let tree_result = rkyv::from_bytes::<BKTree>(&bytes);

        match tree_result {
            Ok(tree) => Ok(tree),
            Err(_) => Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Failed to deserialize BKTree"))
        }
    }

    fn get_damerau_levenshtein_distance(&self, a: &str, b: &str) -> Option<usize> {
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
                let k: usize = da[b.chars().nth(j - 1)? as usize];
                let l: usize = db;

                let cost: usize = if a.chars().nth(i - 1)? == b.chars().nth(j - 1)? { 0 } else { 1 };
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
            da[a.chars().nth(i - 1)? as usize] = i;
        }

        Some(dp[m + 1][n + 1])
    }

    pub fn add(&mut self, word: String) {
        let mut current: usize = 0;
        let mut distance: Option<usize>;

        loop {
            distance = self.get_damerau_levenshtein_distance(&self.tree[current].word, &word);

            match distance {
                Some(0) => return,
                Some(d) if self.tree[current].next[d].is_none() => {
                    if !self.tree[current].word.is_empty() { self.tree[current].next[d] = Some(self.size); }
                    self.tree[self.size as usize].word = word;
                    self.size += 1;
                    return;
                },
                Some(d) => {
                    match self.tree[current].next[d] {
                        Some(n) => current = n as usize,
                        None => return
                    }
                }
                None => return
            }
        }
    }

    pub fn does_contain(&self, word: String) -> bool {
        let mut current: usize = 0;
        let mut distance: Option<usize>;

        loop {
            distance = self.get_damerau_levenshtein_distance(&self.tree[current].word, &word);

            match distance {
                Some(0) => return true,
                Some(d) if self.tree[current].next[d].is_none() => return false,
                Some(d) => {
                    match self.tree[current].next[d] {
                        Some(n) => current = n as usize,
                        None => return false
                    }
                },
                None => return false
            }
        }
    }

    pub fn get_similar_words(&self, word: String, tolerance: usize) -> Option<Vec<String>> {
        let mut result: Vec<String> = Vec::new();
        let mut stack: Vec<usize> = vec![0];

        while !stack.is_empty() {
            let current: usize = stack.pop()?;
            let distance: usize = self.get_damerau_levenshtein_distance(&word, &self.tree[current].word)?;

            if distance <= tolerance {
                result.push(self.tree[current].word.clone());
            }

            let tolerance_start: usize = if distance > tolerance { distance - tolerance } else { 1 };
            let tolerance_end: usize = distance + tolerance;

            for i in tolerance_start..=tolerance_end {
                if self.tree[current].next[i].is_some() {
                    stack.push(self.tree[current].next[i]? as usize);
                }
            }
        }

        Some(result)
    }

    pub fn save_to_file(&self, file_path: &str) -> std::io::Result<()> {
        let bytes: AlignedVec = match rkyv::to_bytes::<_, 256>(self) {
            Ok(b) => b,
            Err(_) => return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Failed to serialize BKTree"))
        };
        let file: File = match std::fs::File::create(file_path) {
            Ok(f) => f,
            Err(e) => return Err(e)
        };
        let mut writer: BufWriter<File> = BufWriter::new(file);
        match writer.write_all(&bytes) {
            Ok(_) => (),
            Err(e) => return Err(e)
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::BKTree;
    use super::super::dictionary::Dictionary;

    #[test]
    #[ignore = "Computationally expensive since it loads the entire dictionary"]
    fn test_from_dictionary() {
        let dictionary: Dictionary = Dictionary::from_file("dictionary.txt", 255).unwrap();
        let mut tree: BKTree = BKTree::new(dictionary.max_word_length as u16, dictionary.alphabet_length as u16, dictionary.words.len());

        assert_eq!(tree.alphabet_length as usize, dictionary.alphabet_length);
        assert_eq!(tree.max_word_length as usize, dictionary.max_word_length);

        for word in dictionary.words.iter() {
            tree.add(word.clone());
        }

        for word in dictionary.words.iter() {
            assert!(tree.does_contain(word.clone()));
        }
    }

    #[test]
    fn test_similar_words() {
        let mut tree: BKTree = BKTree::new(3, 255, 5);

        tree.add("hello".to_string());
        tree.add("world".to_string());
        tree.add("hella".to_string());
        tree.add("hell".to_string());
        tree.add("help".to_string());

        let similar_words: Vec<String> = tree.get_similar_words("hell".to_string(), 1).unwrap();
        assert_eq!(similar_words.len(), 4);
        assert!(similar_words.contains(&"hello".to_string()));
        assert!(similar_words.contains(&"hella".to_string()));
        assert!(similar_words.contains(&"hell".to_string()));
        assert!(similar_words.contains(&"help".to_string()));
    }

    #[test]
    fn test_file_serialization() {
        let mut tree: BKTree = BKTree::new(3, 255, 5);

        tree.add("hello".to_string());
        tree.add("world".to_string());
        tree.add("hella".to_string());
        tree.add("hell".to_string());
        tree.add("help".to_string());

        tree.save_to_file("bk_tree.bin").unwrap();

        let new_tree: BKTree = BKTree::from_file("bk_tree.bin").unwrap();
        
        assert_eq!(tree, new_tree);
        std::fs::remove_file("bk_tree.bin").unwrap();
    }

    #[test]
    #[ignore = "Computationally expensive since it loads the entire dictionary"]
    fn test_full_file_serialization() {
        let dictionary: Dictionary = Dictionary::from_file("dictionary.txt", 255).unwrap();
        let mut tree: BKTree = BKTree::new(dictionary.max_word_length as u16, dictionary.alphabet_length as u16, dictionary.words.len());

        for word in dictionary.words.iter() {
            tree.add(word.clone());
        }

        tree.save_to_file("bk_tree_full.bin").unwrap();

        let new_tree: BKTree = BKTree::from_file("bk_tree_full.bin").unwrap();
 
        assert_eq!(tree, new_tree);
        std::fs::remove_file("bk_tree_full.bin").unwrap();
    }
}
