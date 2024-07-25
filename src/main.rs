mod bloom_filter;
mod bk_tree;
mod dictionary;
mod processor;
mod spell_check;
mod utils;

use spell_check::SpellCheck;

fn main() {
    let mut spell_check: SpellCheck = SpellCheck::new();
    let _ = spell_check.populate_bk_tree();
    let _ = spell_check.populate_bloom_filter();

    spell_check.run();
}
