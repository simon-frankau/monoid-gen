//
// idem_monoid: Code to generate all the distinct words in an
// idempotent monoid over n letters, normalise words to a canonical
// form, and show the steps to perform that normalisation.
//

type Sym = u8;

type Word = Vec<Sym>;
type WordRef<'a> = &'a [Sym];

type Alphabet = Vec<Sym>;
type AlphabetRef<'a> = &'a [Sym];

fn sym_to_c(i: Sym) -> char {
    char::from_digit(i as u32 + 10, 36).unwrap()
}

fn word_to_str(v: WordRef) -> String {
    v.iter().map(|c| sym_to_c(*c)).collect::<String>()
}

////////////////////////////////////////////////////////////////////////
// Monoid generation
//

// Generate the idempotent monoid of all the words using *exactly* the
// same n letters. e.g. "ab", "ba", "aba", "bab", but not "a" and "b".
fn generate_exact_monoid(n_letters: usize) -> Vec<Word> {
    if n_letters == 0 {
        return vec![vec![]];
    }

    // Start with the words from the (n-1)-letter monoid.
    let shorter_words = generate_exact_monoid(n_letters - 1);

    // Generate all the variants on this (n-1)-letter monoid by using
    // all but one of the letter from the n-letter alphabet,
    // generateing all the (n-1)-letter elements of the n-letter
    // monoid.
    let various_shorter_words = variants_on(&shorter_words, n_letters);

    let mut words = Vec::new();
    for (left_word, left_sym) in various_shorter_words.iter() {
        let mut left = left_word.clone();
        left.push(*left_sym);
        for (right_word, right_sym) in various_shorter_words.iter() {
            let mut right = vec![*right_sym];
            right.extend(right_word.iter());

            words.push(merge(&left, &right));
        }
    }

    words
}

// Given a set of words, generate the set of words with one more
// letter, and the associated missed-out letter.
fn variants_on(words: &[Word], n_letters: usize) -> Vec<(Word, Sym)> {
    let mut res = Vec::new();
    for i in 0..n_letters as u8 {
        for word in words.iter() {
            let new_word = word
                .iter()
                .map(|sym| sym + if *sym >= i { 1 } else { 0 })
                .collect::<Vec<_>>();
            res.push((new_word, i));
        }
    }
    res
}

// Given two words that may overlap, generate the concatenation with
// maximal overlap.
fn merge(left: WordRef, right: WordRef) -> Word {
    let l_len = left.len();
    let r_len = right.len();

    let start = if right.len() > left.len() {
        0
    } else {
        left.len() - right.len()
    };

    for idx in start..=l_len {
        let l_part = &left[idx..];
        let r_part = &right[..l_part.len()];
        if l_part == r_part {
            let mut word = Vec::from(&left[..idx]);
            word.extend(right.iter());
            return word;
        }
    }

    panic!("Should always equal at zero length overlap!");
}

////////////////////////////////////////////////////////////////////////
// Main entry point.
//

fn main() {
    let words = generate_exact_monoid(3);

    for word in words {
        println!("{}", word_to_str(&word));
    }
}