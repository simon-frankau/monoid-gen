//
// idem_monoid: Code to generate all the distinct words in an
// idempotent monoid over n letters, normalise words to a canonical
// form, and show the steps to perform that normalisation.
//

use clap::Parser;
use itertools::Itertools;

use std::collections::HashSet;
use std::fmt;

////////////////////////////////////////////////////////////////////////
// Types and utilities
//

type Sym = u8;

type Word = Vec<Sym>;
type WordRef<'a> = &'a [Sym];

fn sym_to_c(i: Sym) -> char {
    char::from_digit(i as u32 + 10, 36).unwrap()
}

fn word_to_str(v: WordRef) -> String {
    v.iter().map(|c| sym_to_c(*c)).collect::<String>()
}

fn str_to_word(s: &str) -> Word {
    s.bytes().map(|c| c - b'a').collect()
}

fn chain(words: &[WordRef]) -> Word {
    words.iter().flat_map(|w| w.to_vec()).collect::<Vec<_>>()
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
        let left = chain(&[left_word, &[*left_sym]]);
        for (right_word, right_sym) in various_shorter_words.iter() {
            let right = chain(&[&[*right_sym], right_word]);
            words.push(merge(&left, &right));
        }
    }

    words
}

// Generate all the members of the monoid, not just those using all
// possible letters.
fn generate_monoid(n_letter: usize) -> Vec<Word> {
    let mut res = Vec::new();

    // For each i letter subset of the alphabet...
    for i in 0..=n_letter {
        let words = generate_exact_monoid(i);
        for comb in (0..n_letter as Sym).combinations(i) {
            // Create all the words using that subset:
            for word in words.iter() {
                res.push(word.iter().map(|c| comb[*c as usize]).collect::<Word>());
            }
        }
    }

    res
}

// Given a set of words, generate the set of words with one more
// letter, and the associated missed-out letter.
fn variants_on(words: &[Word], n_letters: usize) -> Vec<(Word, Sym)> {
    let mut res = Vec::new();
    for i in 0..n_letters as u8 {
        for word in words.iter() {
            let new_word = word
                .iter()
                .map(|sym| sym + u8::from(*sym >= i))
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

    let start = if r_len > l_len { 0 } else { l_len - r_len };

    for idx in start..=l_len {
        let l_part = &left[idx..];
        let r_part = &right[..l_part.len()];
        if l_part == r_part {
            return chain(&[&left[..idx], right]);
        }
    }

    panic!("Should always equal at zero length overlap!");
}

////////////////////////////////////////////////////////////////////////
// Word reduction
//

// Find the length of the longest left subword using n distinct letters.
fn find_left_subword(word: WordRef, n: usize) -> usize {
    let mut letters = HashSet::new();
    for (idx, sym) in word.iter().enumerate() {
        letters.insert(*sym);
        if letters.len() == n + 1 {
            return idx;
        }
    }
    panic!("Oh dear, not enough distinct letters (shouldn't happen!)");
}

// Find the index of the start of the longest right subword using n
// distinct letters.
fn find_right_subword(word: WordRef, n: usize) -> usize {
    let mut letters = HashSet::new();
    for (idx, sym) in word.iter().enumerate().rev() {
        letters.insert(*sym);
        if letters.len() == n + 1 {
            return idx + 1;
        }
    }
    panic!("Oh dear, not enough distinct letters (shouldn't happen!)");
}

// Reduce the left sub-word that uses all but one of the characters in
// the word.
fn reduce_left(word: WordRef, n_letters: usize) -> Steps {
    let len = find_left_subword(word, n_letters - 1);
    let to_reduce = &word[..len];
    let rest = &word[len..];
    reduce(to_reduce).suffix(&[rest])
}

// Same, but for the right.
fn reduce_right(word: WordRef, n_letters: usize) -> Steps {
    let len = find_right_subword(word, n_letters - 1);
    let to_reduce = &word[len..];
    let rest = word[..len].to_vec();
    Steps::prefix(&[&rest], &reduce(to_reduce))
}

// Like `merge`, but returns steps. Finds the unsquaring the maximally
// shortens the word.
fn reduce_middle(left: WordRef, right: WordRef) -> Steps {
    let l_len = left.len();
    let r_len = right.len();

    // Starting index of the biggest possible overlap.
    let start = if r_len > l_len { 0 } else { l_len - r_len };

    for idx in start..l_len {
        // Get the left and right potential parts of the overlap, see
        // if they do.
        let l_part = &left[idx..];
        let r_part = &right[..l_part.len()];
        if l_part == r_part {
            // They do. Build the unsquaring operation to eliminate
            // it.
            let l = &left[..idx];
            let m = l_part;
            let r = &right[l_part.len()..];
            return Steps::prefix(&[l], &Steps::square(&[m]).suffix(&[r])).time_rev();
        }
    }

    Steps::empty(&chain(&[left, right]))
}

// Given a word, produces the steps that maximally shortens it to
// normal form.
fn reduce(word: WordRef) -> Steps {
    // Base case - do nothing for empty string.
    if word.is_empty() {
        return Steps::empty(word);
    }

    // Get alphabet size.
    let letters: HashSet<u8> = HashSet::from_iter(word.iter().copied());
    let n_letters = letters.len();

    // Place to accumulate the steps performed:
    let mut steps = Vec::new();

    // Reduce the subwords (using n - 1 letters) on the left and right.
    steps.push(reduce_left(word, n_letters));
    let word = &steps.last().unwrap().end;
    steps.push(reduce_right(word, n_letters));
    let word = &steps.last().unwrap().end;

    // Extract the left and right shortest words using all the letters
    // (one longer than the longest words using all but one letter!).
    let l_len = find_left_subword(word, n_letters - 1) + 1;
    let l_word = word[..l_len].to_vec();

    let r_idx = find_right_subword(word, n_letters - 1) - 1;
    let r_word = word[r_idx..].to_vec();

    // If the left and right subwords overlap no further reduction is
    // possible, they're already in minimal form.
    if l_len <= r_idx {
        // Only try to remove a middle section if there is one.
        if l_len < r_idx {
            steps.push(remove_middle(&l_word, &word[l_len..r_idx], &r_word));
        }

        // Then remove overlap between left and right subwords.
        steps.push(reduce_middle(&l_word, &r_word));
    }
    Steps::join(steps)
}

////////////////////////////////////////////////////////////////////////
// Structure to represent a sequence of squaring/unsquaring
// steps. Intended to make it impossible (when using the interface) to
// generate invalid sequences of operations.
//

// A sequence of steps to go from a word to another representation of
// it. It tries to encapsulate the steps to make sure we don't
// accidentally mis-step.
struct Steps {
    start: Word,
    end: Word,
    // We use strings to allow us to make the steps clearer.  Each
    // step represents before and after the step, so that the after of
    // one step should be the same as the before of the next.
    steps: Vec<(String, String)>,
}

impl fmt::Display for Steps {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for step in self.steps.iter() {
            writeln!(f, "{} -> {}", step.0, step.1)?;
        }
        Ok(())
    }
}

impl Steps {
    // No-op
    fn empty(w: WordRef) -> Steps {
        Steps {
            start: w.to_vec(),
            end: w.to_vec(),
            steps: Vec::new(),
        }
    }

    // Represents a step from w to ww:
    fn square(m: &[WordRef]) -> Steps {
        let mw = chain(m);
        let m2w = chain(&[&mw, &mw]);

        let m1s = word_to_str(&mw);
        let m2s = word_to_str(&m2w);

        Steps {
            start: mw,
            end: m2w,
            steps: vec![(format!("({m1s})"), format!("({m2s})"))],
        }
    }

    fn join(list: Vec<Steps>) -> Steps {
        let start = list.first().unwrap().start.clone();
        let mut end = start.clone();
        let mut steps = Vec::new();

        for mut step in list.into_iter() {
            assert_eq!(end, step.start);
            steps.append(&mut step.steps);
            end = step.end;
        }

        Steps { start, end, steps }
    }

    // Written this way so we can use it in prefix form
    fn prefix(words: &[WordRef], s: &Steps) -> Steps {
        let word = chain(words);
        let str = word_to_str(&word);
        Steps {
            start: chain(&[&word, &s.start]),
            end: chain(&[&word, &s.end]),
            steps: s
                .steps
                .iter()
                .map(|(l, r)| (format!("{}{}", str, l), format!("{}{}", str, r)))
                .collect::<Vec<_>>(),
        }
    }

    fn suffix(&self, words: &[WordRef]) -> Steps {
        let word = chain(words);
        let str = word_to_str(&word);
        Steps {
            start: chain(&[&self.start, &word]),
            end: chain(&[&self.end, &word]),
            steps: self
                .steps
                .iter()
                .map(|(l, r)| (format!("{}{}", l, str), format!("{}{}", r, str)))
                .collect::<Vec<_>>(),
        }
    }

    // Generate steps for the reverse operation.
    fn time_rev(&self) -> Steps {
        Steps {
            start: self.end.clone(),
            end: self.start.clone(),
            steps: self
                .steps
                .iter()
                .rev()
                .map(|(l, r)| (r.clone(), l.clone()))
                .collect(),
        }
    }

    // Generate steps for the word written backwards.
    fn word_rev(&self) -> Steps {
        fn backwards(s: &str) -> String {
            s.chars()
                .rev()
                .map(|c| match c {
                    ')' => '(',
                    '(' => ')',
                    _ => c,
                })
                .collect::<String>()
        }

        Steps {
            start: self.start.iter().rev().copied().collect::<Vec<_>>(),
            end: self.end.iter().rev().copied().collect::<Vec<_>>(),
            steps: self
                .steps
                .iter()
                .map(|(l, r)| (backwards(l), backwards(r)))
                .collect::<Vec<_>>(),
        }
    }
}

////////////////////////////////////////////////////////////////////////
// Core reduction algorithm, from Lothaire.
//

// Given x, y, alph(y) <= alph(x), find u s.t. x ~ xyu, and the steps
// to go from x to xyu.
fn find_u(x: WordRef, y: WordRef) -> (Steps, Word) {
    // Keep squaring appropriate subwords to build up a word of the
    // form xyu. 'l' holds the word left of the insertion point, 'r'
    // the word to the right.
    let mut l = x.to_vec();
    let mut r: Word = Vec::new();

    let mut steps = Vec::new();

    for sym in y.iter() {
        let (repeat_point, _) = l
            .iter()
            .enumerate()
            .rev()
            .find(|(_, sym2)| **sym2 == *sym)
            .unwrap();

        steps.push(
            Steps::prefix(&[&l[..repeat_point]], &Steps::square(&[&l[repeat_point..]]))
                .suffix(&[&r]),
        );

        r = chain(&[&l[repeat_point + 1..], &r]);
        l.push(*sym);
    }

    (Steps::join(steps), r)
}

// Given x, y, alph(y) <= alph(x), find v s.t. x ~ vyx
fn find_v(x: WordRef, y: WordRef) -> (Steps, Word) {
    let mut xr = x.to_vec();
    xr.reverse();
    let mut yr = y.to_vec();
    yr.reverse();
    let (steps, mut ur) = find_u(&xr, &yr);
    ur.reverse();
    (steps.word_rev(), ur)
}

// Convert a string from LMR to LR. Doesn't eliminate overlap between
// L and R.
fn remove_middle(l: WordRef, m: WordRef, r: WordRef) -> Steps {
    // Choose u s.t. L ~ LMRu
    let (l_to_lmru, u) = &find_u(l, &chain(&[m, r]));
    let lmru_to_l = l_to_lmru.time_rev();
    // Choose v s.t. R ~ vLR
    let (r_to_vlr, v) = &find_v(r, l);
    let vlr_to_r = r_to_vlr.time_rev();

    Steps::join(vec![
        // LM(R) -> LM(vLR)
        Steps::prefix(&[l, m], r_to_vlr),
        //   LMv(LR) -> LMv(LRLR)
        Steps::prefix(&[l, m, v], &Steps::square(&[l, r])),
        // LM(vLR)LR -> LM(R)LR
        Steps::prefix(&[l, m], &vlr_to_r.suffix(&[l, r])),
        // LMR(L)R -> LMR(LMRu)R
        Steps::prefix(&[l, m, r], &l_to_lmru.suffix(&[r])),
        // (LMRLMR)uR -> (LMR)uR
        Steps::square(&[l, m, r]).suffix(&[u, r]).time_rev(),
        // (LMRu)R -> LR
        lmru_to_l.suffix(&[r]),
    ])
}

////////////////////////////////////////////////////////////////////////
// Main entry point.
//

#[derive(Debug, Parser)]
#[clap(name = "idem_monoid")]
#[clap(about = "Tool for generating and reducing elements of an idempotent free monoid", long_about = None)]
struct Cli {
    /// Size of alphabet to use when generating the idempotent monoid.
    #[clap(long, value_parser, default_value_t = 3)]
    generators: usize,

    /// Or a word to reduce to canonical form
    #[clap(long, value_parser)]
    reduce: Option<String>,

    /// If reducing a word, show the reduction path?
    #[clap(long, value_parser)]
    verbose: bool,
}

fn main() {
    let args = Cli::parse();

    if let Some(reduce_me) = args.reduce {
        // Reduce the given word.
        let as_word = str_to_word(&reduce_me);
        let steps = reduce(&as_word);
        if args.verbose {
            println!("{}", steps);
        }
        let as_str = word_to_str(&steps.end);
        println!("{}", as_str);
    } else {
        // Generate all the elements of the monad.
        let words = generate_monoid(args.generators);

        for word in words {
            let word_str = if word.is_empty() {
                "0".to_string()
            } else {
                word_to_str(&word)
            };
            println!("{}", word_str);
        }
    }
}
