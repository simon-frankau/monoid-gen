# Idempotent monoid builder/element normaliser

http://oeis.org/A005345 says that the free idempotent monoids on n
letters are finite (and also defines a free idempotent monoid, if you
don't know what that is!).

The code in this repo will generate all the elements of such a monoid
(without repeats), and given an arbitrary word reduce it to the normal
form used when generating the elements.

Originally, I build some code to brute-force generate the 3-letter
monoid from scratch, without recourse to any useful theory. That code
now lives in the `original` directory. See its
[README.md](original/README.md) for more details.

Since then, I was pointed at [Chapter 2](paper/Lothaire-Ch2.pdf) of
Lothaire's *Combinatorics on Words*, which shows what the equivalences
across words look like. From this, we can build tools to generate the
monoid and normalise any word to a canonical form that can be found in
the list.

## The theory

Two words are equivalent if and only if they have the same alphabet
(use exactly the same set of letters), and can be expressed as (p, a,
b, q) and (p', a, b, q') respectively, where p and p' are the longest
prefixes that use all but one of the letters ('a' being the missing
letter), and q and q' the similar longest suffixes, and p and p' and q
and q' are equivalent.

We can make the canonical form for a word the shortest possible form
of the word. Then:

To generate all the n-letter words we start by generating all the
n-1-letter words with an n-1-letter alphabet, and then use all the
substitutions from n-1 letters to n letters to generate the n-1-letter
words with an n-letter alphabet. These form our ps and qs. Take all
pairs, substitute in the missing letters as a and b, carve out any
overlap from pa and bq, and you have all the n-letter words, in
minimum-length form.

Alternatively, to normalise a word, get its (p, a, b, q)
representation, recursively normalise p and q, and then reassemble it
into the minimum length word.

Lothaire constructively shows how you can convert words with the same
(p, a, b, q) to each other, so we can use this to write out the
sequence of steps used to minimise a word.

I'll admit my explanation's a bit hand-wavy. Read the code for
details! :)

## Tool usage

To produce the idempotent monoid generated from 3 letters, run:

```
$ cargo run --bin idem_monoid -- --generators 3
...
0
a
b
c
bab
ba
ab
aba
cac
...
```

You can change '3' to any other number, but beware the number of
elements grows exponentially (3 has 160 elements, 4 has 332381
elements), so '4' is probably the biggest you'll want to try.

To reduce a word to its canonical form, use something like this:

```
$ cargo run --bin idem_monoid -- --reduce ababcbcbab
...
abcbab
```

To see the reduction steps you can perform to achieve this
minimisation, add `--verbose`:

```
$ cargo run --bin idem_monoid -- --reduce ababcbcbab --verbose
...
(abab)cbcbab -> (ab)cbcbab
abcb(c)bab -> abcb(cc)bab
abcb(ccb)ab -> abcb(ccbccb)ab
abcbcc(bccba)b -> abcbcc(bccbabccba)b
abcbccbccb(abccbab) -> abcbccbccb(abccbababccbab)
abcbcc(bccbabccba)babccbab -> abcbcc(bccba)babccbab
abcb(ccbccb)ababccbab -> abcb(ccb)ababccbab
...
```

As you can see, the algorithm does not take the quick and simply
route, and isn't entirely obvious in what it's doing, but it follows
Lothaire's approach and gets there in the end.
