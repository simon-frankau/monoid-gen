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

**TODO: Write code to generate the monoid, reduce words to canonical
form, and show the path to get there.**
