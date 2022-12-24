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
        // Use lowest index as target.
        let tgt = tgt1.min(tgt2);

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

    fn to_sets(&self) -> Vec<Vec<Word>> {
        let mut mapping: HashMap<Key, Vec<Key>> = HashMap::new();
        for (idx, tgt) in self.ptrs.iter().enumerate() {
            mapping.entry(*tgt).or_insert_with(|| Vec::new()).push(idx as Key)
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
// Entry point
//

fn pretty_print_sets(sets: &[Vec<Word>]) {
    for set in sets.iter() {
        let words = set.iter().map(|sym| syms_to_str(sym)).collect::<Vec<_>>();
        println!("{}", words.join(", "));
    }
}

fn combine(lhs: WordRef, rhs: WordRef) -> Word {
    let mut res = lhs.to_vec();
    res.extend(if lhs.last() == rhs.first() { &rhs[1..] } else { rhs });
    res
}

fn extend(u: &mut Union) {
    let len = u.rev_map.len();

    // Generate all words from pairs of words.
    for i_idx in 0..len {
	eprintln!("Gen {}/{}", i_idx, len);
        let i = u.rev_map[i_idx].clone();

        let i_rep_idx = u.ptrs[i_idx];
        let i_rep = u.rev_map[i_rep_idx as usize].clone();

        for j_idx in 0..len {
            let j = &u.rev_map[j_idx];
            let ij = combine(&i, j);
            let ij_idx = u.key_for(&ij);

            let j_rep_idx = u.ptrs[j_idx];
            let j_rep = &u.rev_map[j_rep_idx as usize];
	    let ij_rep = combine(&i_rep, j_rep);
            let ij_rep_idx = u.key_for(&ij_rep);

            u.union(ij_idx, ij_rep_idx);
        }
    }

    // Ensure equivalence of squares.
    // TODO let len = u.rev_map.len();
    for i_idx in 0..len {
	eprintln!("Square {}/{}", i_idx, len);
        let i = &u.rev_map[i_idx];
        let ii = combine(i, i);
        let ii_idx = u.key_for(&ii);
        u.union(i_idx as Key, ii_idx);
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

fn extend2(u: &mut Union) {
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

fn main() {
    let mut u = Union::new();

    for i in 0..NUM_SYMS {
	u.key_for(&vec![i]);
    }

    for i in 1..=20 {
	extend2(&mut u);
	let sets = u.to_sets();
	let big_sets = sets.iter().map(|v| v.len()).filter(|x| *x >= 5).count();
	let contains_small = sets.iter().map(|v| v.iter().map(|v| v.len()).min().unwrap()).filter(|x| *x < 10).count();
	println!("##### {} ({} entries, {} big, {} contain small)", i, sets.len(), big_sets, contains_small);
	// pretty_print_sets(&sets);
    }
}
