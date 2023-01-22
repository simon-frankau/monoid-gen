use std::collections::HashMap;

////////////////////////////////////////////////////////////////////////
// Helpers
//

type Sym = u8;
type Word = Vec<Sym>;
type WordRef<'a> = &'a [Sym];

fn sym_to_c(i: Sym) -> char {
    char::from_digit(i as u32 + 10, 36).unwrap()
}

fn syms_to_str(v: WordRef) -> String {
    v.iter().map(|c| sym_to_c(*c)).collect::<String>()
}

fn c_to_sym(c: char) -> Sym {
    c.to_digit(36).unwrap() as Sym - 10
}

fn str_to_syms(s: &str) -> Word {
    s.chars().map(c_to_sym).collect::<Vec<_>>()
}

////////////////////////////////////////////////////////////////////////
// Union find
//

type Key = u32;

// Implement union-find ourselves, yet again.
#[derive(Debug, Clone, Eq, PartialEq)]
struct Union {
    // Map things to keys.
    rep_map: HashMap<Word, Key>,
    // And back.
    rev_map: Vec<Word>,
    // Map keys to other keys.
    ptrs: Vec<Key>,
}

impl Union {
    fn new() -> Union {
        // Initially, all pointers point to themselves.
        Union {
            rep_map: HashMap::new(),
            rev_map: Vec::new(),
            ptrs: Vec::new(),
        }
    }

    fn key_for(&mut self, v: WordRef) -> Key {
        *self.rep_map.entry(v.to_vec()).or_insert_with(|| {
            let i = self.rev_map.len() as Key;
            self.rev_map.push(v.to_vec());
            self.ptrs.push(i);
            i
        })
    }

    fn union(&mut self, mut idx1: Key, mut idx2: Key) {
        // Not efficient, just get it done.

        // Dereference idx1's chain.
        let mut tgt1 = idx1;
        while self.ptrs[tgt1 as usize] != tgt1 {
            assert!(self.ptrs[tgt1 as usize] < tgt1);
            tgt1 = self.ptrs[tgt1 as usize];
        }
        // Dereference idx2's chain.
        let mut tgt2 = idx2;
        while self.ptrs[tgt2 as usize] != tgt2 {
            assert!(self.ptrs[tgt2 as usize] < tgt2);
            tgt2 = self.ptrs[tgt2 as usize];
        }
        // Use shortest word as target.
        let tgt = if self.rev_map[tgt1 as usize].len() < self.rev_map[tgt2 as usize].len() {
            tgt1
        } else {
            tgt2
        };

        // Repoint idx1's chain to target.
        while self.ptrs[idx1 as usize] != idx1 {
            let tmp = self.ptrs[idx1 as usize];
            self.ptrs[idx1 as usize] = tgt;
            idx1 = tmp;
        }
        self.ptrs[idx1 as usize] = tgt;
        // Repoint idx2's chain to target.
        while self.ptrs[idx2 as usize] != idx2 {
            let tmp = self.ptrs[idx2 as usize];
            self.ptrs[idx2 as usize] = tgt;
            idx2 = tmp;
        }
        self.ptrs[idx2 as usize] = tgt;
    }

    // Chase the pointers, to get the word representing the
    // equivalence class. As we always unify on the shortest word,
    // this should be the shortest word in the equivalence class.
    fn rep_of(&self, mut key: Key) -> Word {
        while self.ptrs[key as usize] != key {
            key = self.ptrs[key as usize];
        }
        self.rev_map[key as usize].clone()
    }

    fn to_sets(&self) -> Vec<Vec<Word>> {
        let mut mapping: HashMap<Key, Vec<Key>> = HashMap::new();
        for (idx, tgt) in self.ptrs.iter().enumerate() {
            mapping
                .entry(*tgt)
                .or_insert_with(|| Vec::new())
                .push(idx as Key)
        }

        let convert = |set_num: &Key| self.rev_map[*set_num as usize].clone();

        let mut sets = mapping
            .values()
            .map(|set_list| set_list.iter().map(convert).collect::<Vec<_>>())
            .collect::<Vec<_>>();
        for set in sets.iter_mut() {
            set.sort();
        }
        sets.sort();
        sets
    }
}

////////////////////////////////////////////////////////////////////////
// Main code.
//

fn pretty_print_sets(sets: &[Vec<Word>]) {
    for set in sets.iter() {
        let words = set.iter().map(|sym| syms_to_str(sym)).collect::<Vec<_>>();
        println!("{}", words.join(", "));
    }
}

const NUM_SYMS: Sym = 3;

fn register(u: &mut Union, word: WordRef) {
    let k = u.key_for(&word);
    // Find all sub-squares, and union with square roots.
    for len in 2..=word.len() / 2 {
        for idx in 0..=word.len() - 2 * len {
            if word[idx..][..len] == word[idx + len..][..len] {
                let mut reduced_word = word[..idx].to_vec();
                reduced_word.extend(&word[idx + len..]);
                let k2 = u.key_for(&reduced_word);
                u.union(k, k2);
            }
        }
    }
}

fn extend(u: &mut Union) {
    let len = u.rev_map.len();

    for idx in 0..len {
        let elt = u.rev_map[idx].clone();
        let last = *elt.last().unwrap();
        for sym in 0..NUM_SYMS {
            if last != sym {
                let mut new = elt.to_vec();
                new.push(sym);
                register(u, &new);
            }
        }
    }
}

// Returns a cumulative histogram, where the nth element is the number
// of equivalence classes containing where each class contains a word
// smaller than or equal to n.
fn cumulative_histogram(v: &[usize]) -> Vec<usize> {
    // Collect counts of each min-size.
    let len = v.iter().max().unwrap() + 1;
    let mut counts = vec![0; len];

    // Hack: We don't include the empty string in our search, because
    // concatenating empty strings onto everything, squaring empty
    // string etc. is a waste of time. Instead, we'll hack it into the
    // count here!
    counts[0] = 1;

    for elt in v.iter() {
        counts[*elt] += 1;
    }

    // Convert to cumulative.
    let mut c = 0;
    for count in counts.iter_mut() {
        c += *count;
        *count = c;
    }

    counts
}

fn main() {
    let mut u = Union::new();

    for i in 0..NUM_SYMS {
        u.key_for(&vec![i]);
    }

    const GENERATE_HISTOGRAMS: bool = false;
    const GENERATE_ELEMENTS: bool = false;

    if GENERATE_HISTOGRAMS {
        // Generate cumulative histograms of the number of equivalence
        // classes as the search length increases.

        // On my M1 Macbook: 23 takes 10 minutes, 22 takes 140s.
        for i in 1..=22 {
            extend(&mut u);
            let sets = u.to_sets();
            let min_elts = sets
                .iter()
                .map(|set| set.iter().map(|word| word.len()).min().unwrap())
                .collect::<Vec<usize>>();
            let histogram = cumulative_histogram(&min_elts);
            println!("##### {} ({} entries, {:?})", i, sets.len(), &histogram);
            // pretty_print_sets(&sets);
        }
    } else if GENERATE_ELEMENTS {
        const MAX_LENGTH: usize = 20;
        const MAX_REP_LEN: usize = 8;

        // Do the minimal work to find the 160 elements.
        for _ in 1..=MAX_LENGTH {
            extend(&mut u);
        }

        let sets = u.to_sets();

        // Filter out equivalence classes that don't contain a short word.
        let sets = sets
            .iter()
            .filter(|set| set.iter().map(|word| word.len()).min().unwrap() <= MAX_REP_LEN)
            .collect::<Vec<_>>();

        // For each equivalence class, find the shortest
        // representations. Note we gather all equivalent shortest
        // representations, just in case it turns out the shortest one
        // isn't unique, which I think would be
        // surpising/interesting... turns out it's unique.
        fn reps(set: &[Word]) -> Vec<Word> {
            let shortest = set.iter().map(|word| word.len()).min().unwrap();
            set.iter()
                .filter(|word| word.len() == shortest)
                .map(|x| x.to_vec())
                .collect::<Vec<_>>()
        }
        let shortests = sets.iter().map(|set| reps(set)).collect::<Vec<_>>();
        for elt in shortests.iter() {
            let stringified = elt.iter().map(|word| syms_to_str(word)).collect::<Vec<_>>();

            println!("{}", stringified.join(", "));
        }
    } else {
        // Let's generate the full "multiplication table":
        const MAX_LENGTH: usize = 20;
        const MAX_REP_LEN: usize = 8;

        // Do the minimal work to find the 160 elements.
        for _ in 1..=MAX_LENGTH {
            extend(&mut u);
        }

        let sets = u.to_sets();

        // Filter out equivalence classes that don't contain a short word.
        let sets = sets
            .iter()
            .filter(|set| set.iter().map(|word| word.len()).min().unwrap() <= MAX_REP_LEN)
            .collect::<Vec<_>>();

        // For each equivalence class, find the shortest representation.
        fn rep(set: &[Word]) -> Word {
            set.iter()
                .min_by(|x, y| x.len().cmp(&y.len()))
                .unwrap()
                .to_vec()
        }
        let reps = sets.iter().map(|set| rep(set)).collect::<Vec<_>>();

        let u_count = u.rev_map.len();
        for i in reps.iter() {
            for j in reps.iter() {
                let mut ij = i.clone();
                if i.last() == j.first() {
                    // Avoid repeated letters.
                    ij.extend(&j[1..]);
                } else {
                    ij.extend(j);
                }
                let ij_key = u.key_for(&ij);
                let ij = u.rep_of(ij_key);
                println!(
                    "{} * {} = {}",
                    &syms_to_str(i),
                    &syms_to_str(j),
                    &syms_to_str(&ij)
                );
            }
        }
        // None of the words we've created should be new.
        assert_eq!(u_count, u.rev_map.len());
    }
}
