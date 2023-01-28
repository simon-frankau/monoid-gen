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

fn reduce(word: WordRef) -> Word {
    // Base case.
    if word.is_empty() {
        return Vec::new();
    }

    let letters: HashSet<u8> = HashSet::from_iter(word.iter().copied());
    let n_letters = letters.len();

    // Take letters until you hit n distinct letters. Get back a word
    // with n-1 distinct letters, and the nth letter.
    fn take_distinct<'a>(iter: impl Iterator<Item = &'a u8>, n: usize) -> (Word, Sym) {
        let mut word = Vec::new();
        let mut letters = HashSet::new();
        for c in iter {
            letters.insert(*c);
            if letters.len() == n {
                return (word, *c);
            }
            word.push(*c);
        }

        panic!("Oh dear, not enough distinct letters (shouldn't happen!)");
    }

    let (mut l_word, l_sym) = take_distinct(word.iter(), n_letters);
    l_word = reduce(&l_word);
    l_word.push(l_sym);

    let (mut r_word, r_sym) = take_distinct(word.iter().rev(), n_letters);
    r_word = reduce(&r_word);
    r_word.push(r_sym);
    r_word.reverse();

    merge(&l_word, &r_word)
}

////////////////////////////////////////////////////////////////////////
// Code to print out reduction paths.
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
    // Represents a step from l(m1)r to l(m2)r:
    fn new(l: &[WordRef], m1: &[WordRef], m2: &[WordRef], r: &[WordRef]) -> Steps {
        let lw = chain(l);
        let m1w = chain(m1);
        let m2w = chain(m2);
        let rw = chain(r);

        let ls = word_to_str(&lw);
        let m1s = word_to_str(&m1w);
        let m2s = word_to_str(&m2w);
        let rs = word_to_str(&rw);

        Steps {
            start: chain(&[&lw, &m1w, &rw]),
            end: chain(&[&lw, &m2w, &rw]),
            steps: vec![(format!("{ls}({m1s}){rs}"), format!("{ls}({m2s}){rs}"))],
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

    fn prefix(&self, word: WordRef) -> Steps {
        let str = word_to_str(word);
        Steps {
            start: chain(&[word, &self.start]),
            end: chain(&[word, &self.end]),
            steps: self
                .steps
                .iter()
                .map(|(l, r)| (format!("{}{}", str, l), format!("{}{}", str, r)))
                .collect::<Vec<_>>(),
        }
    }

    fn suffix(&self, word: WordRef) -> Steps {
        let str = word_to_str(word);
        Steps {
            start: chain(&[&self.start, word]),
            end: chain(&[&self.end, word]),
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

        steps.push(Steps::new(
            &[&l[..repeat_point]],
            &[&l[repeat_point..]],
            &[&l[repeat_point..], &l[repeat_point..]],
            &[&r],
        ));

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
//
//
// TODO: Insert and remove overlap, if needed.
fn remove_middle(l: WordRef, m: WordRef, r: WordRef) -> Word {
    // Choose u s.t. L ~ LMRu
    let (steps_u, u) = &find_u(l, &chain(&[m, r]));
    // Choose v s.t. R ~ vLR
    let (steps_v, v) = &find_v(r, l);

    let steps = Steps::join(vec![
        // * LM(R) -> LM(vLR)
        steps_v.prefix(&chain(&[l, m])),
        //   LMv(LR) -> LMv(LRLR)
        Steps::new(&[l, m, v], &[l, r], &[l, r, l, r], &[]),
        // * LM(vLR)LR -> LM(R)LR
        steps_v
            .time_rev()
            .prefix(&chain(&[l, m]))
            .suffix(&chain(&[l, r])),
        // * LMR(L)R -> LMR(LMRu)R
        steps_u.prefix(&chain(&[l, m, r])).suffix(r),
        //   (LMRLMR)uR -> (LMR)uR
        Steps::new(&[], &[l, m, r, l, m, r], &[l, m, r], &[u, r]),
        // * (LMRu) R -> L R
        steps_u.time_rev().suffix(r),
    ]);

    println!("{steps}");
    steps.end
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
        if args.verbose {
            // TODO
            let x = [0, 1, 2, 3];
            let y = [2, 0];
            let z = [3, 2, 0, 1];
            let u = remove_middle(&x, &y, &z);
            println!(
                "{} {} {}",
                word_to_str(&x),
                word_to_str(&y),
                word_to_str(&u)
            );
        } else {
            // Reduce the given word.
            let as_word = reduce_me.as_bytes();
            let reduced = reduce(as_word);
            let as_str = String::from_utf8(reduced.to_vec()).unwrap();
            println!("{}", as_str);
        }
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
