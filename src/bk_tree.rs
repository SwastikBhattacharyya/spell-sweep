#[derive(Clone, Debug)]
pub struct Node {
    pub word: String,
    pub next: Vec<Option<usize>>
}

impl Node {
    pub fn new(word: String, max_word_length: usize) -> Self {
        Self {
            word,
            next: vec![None; max_word_length * 2]
        }
    }
}

#[derive(Debug)]
#[readonly::make]
pub struct BKTree {
    pub max_word_length: usize,
    pub alphabet_length: usize,
    pub tree: Vec<Node>,
    pub size: usize
}

impl BKTree {
    pub fn new(max_word_length: usize, alphabet_length: usize, max_words: usize) -> Self {
        Self {
            max_word_length,
            alphabet_length,
            tree: vec![Node::new("".to_string(), max_word_length); max_words],
            size: 0
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

        let mut da: Vec<usize> = vec![0; self.alphabet_length];

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
                    self.tree[self.size].word = word;
                    self.size += 1;
                    return;
                },
                Some(d) => current = self.tree[current].next[d].unwrap(),
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
                Some(d) => current = self.tree[current].next[d].unwrap(),
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
                    stack.push(self.tree[current].next[i]?);
                }
            }
        }

        Some(result)
    }
}


#[cfg(test)]
mod tests {
    use super::BKTree;
    use super::super::dictionary::Dictionary;

    #[test]
    #[ignore = "Computationally Expensive"]
    fn test_from_dictionary() {
        let dictionary: Dictionary = Dictionary::from_file("dictionary.txt", 255).unwrap();
        let mut tree: BKTree = BKTree::new(dictionary.max_word_length, dictionary.alphabet_length, dictionary.words.len());

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
}
