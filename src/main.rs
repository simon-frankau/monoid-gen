use std::collections::HashMap;

////////////////////////////////////////////////////////////////////////
// Helpers
//

fn sym_to_c(i: usize) -> char {
    char::from_digit(i as u32 + 10, 36).unwrap()
}

fn syms_to_str(v: &[usize]) -> String {
    v.iter().map(|c| sym_to_c(*c)).collect::<String>()
}

fn c_to_sym(c: char) -> usize {
    c.to_digit(36).unwrap() as usize - 10
}

fn str_to_syms(s: &str) -> Vec<usize> {
    s.chars().map(c_to_sym).collect::<Vec<_>>()
}

////////////////////////////////////////////////////////////////////////
// Union find
//

// Implement union-find ourselves, yet again.
#[derive(Debug, Clone, Eq, PartialEq)]
struct Union {
    // Map things to keys.
    rep_map: HashMap<Vec<usize>, usize>,
    // And back.
    rev_map: Vec<Vec<usize>>,
    // Map keys to other keys.
    ptrs: Vec<usize>,
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

    fn key_for(&mut self, v: &[usize]) -> usize {
        *self.rep_map.entry(v.to_vec()).or_insert_with(|| {
            let i = self.rev_map.len();
            self.rev_map.push(v.to_vec());
	    self.ptrs.push(i);
            i
        })
    }

    fn union(&mut self, mut idx1: usize, mut idx2: usize) {
        // Not efficient, just get it done.

        // Dereference idx1's chain.
        let mut tgt1 = idx1;
        while self.ptrs[tgt1] != tgt1 {
            assert!(self.ptrs[tgt1] < tgt1);
            tgt1 = self.ptrs[tgt1];
        }
        // Dereference idx2's chain.
        let mut tgt2 = idx2;
        while self.ptrs[tgt2] != tgt2 {
            assert!(self.ptrs[tgt2] < tgt2);
            tgt2 = self.ptrs[tgt2];
        }
        // Use lowest index as target.
        let tgt = tgt1.min(tgt2);

        // Repoint idx1's chain to target.
        while self.ptrs[idx1] != idx1 {
            let tmp = self.ptrs[idx1];
            self.ptrs[idx1] = tgt;
            idx1 = tmp;
        }
        self.ptrs[idx1] = tgt;
        // Repoint idx2's chain to target.
        while self.ptrs[idx2] != idx2 {
            let tmp = self.ptrs[idx2];
            self.ptrs[idx2] = tgt;
            idx2 = tmp;
        }
        self.ptrs[idx2] = tgt;
    }

    fn to_sets(&self) -> Vec<Vec<Vec<usize>>> {
        let mut mapping: HashMap<usize, Vec<usize>> = HashMap::new();
        for (idx, tgt) in self.ptrs.iter().enumerate() {
            mapping.entry(*tgt).or_insert_with(|| Vec::new()).push(idx)
        }

	let convert = | set_num: &usize | self.rev_map[*set_num].clone();
	
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

fn main() {
    let mut u = Union::new();

    u.key_for(&vec![0]);
    u.key_for(&vec![1]);
    u.key_for(&vec![2]);

    println!("{:?}", u);

    let sets = u.to_sets();
    println!("{:?}", sets);
}
