# Idempotent monoid builder for 3 letters

http://oeis.org/A005345 says that the free idempotent monoids on n
letters are finite (and also defines a free idempotent monoid, if you
don't know what that is!).

The free idempotent monoid for 0 letters has one elements: { 0 }.

The free idempotent monoid for 1 letter has two elements: { 0, a }.

The free idempotent monoid for 2 letters has seven elements: { 0, a,
b, ab, ba, aba, bab }.

The free idempotent monoid for 3 letters has 160 elements... but what
are they?

The code in this repo tries to find this out.

4+ letters gets a lot bigger (332381 for 4 letters), and would
probably require real thought, so I'm not doing that here. :)

## Approach

I tried and failed at a few approaches before trying a straightforward
one: iteratively build all the strings of increasing length, and put
them into equivalence classes with versions of that string with square
removed.

To be more precise, we:

1. Start with the strings a, b, c.
2. For all strings we have so far, create new strings by appending a,
   b, and c to them.0
3. For each string, find all the square subsequences, and put that
   string in the same equivalence class as the string with the square
   removed (i.e. the substring converted to a single occurence.
4. Go back to step 2 for the next length, until bored/out of
   memory/time.

As an optimisation, we do not allow for repeats of just the same
letter. This normalisation does not change the equivalences we can
find - if two strings with letter repeats are equivalent, the
no-repeat strings are equivalent too.

Not all strings contain a square, but all sufficiently long strings
are still equivalent to a shorter string - they will be equivalent to
some longer string (some square added) that in turn is equivalent to a
shorter string.

This means I've got an algorithm that seems to have converged on a set
of representative elements, but I'm not totally sure it's correct!

### What??

At each step *n*, we have the strings up to length *n* grouped into
equivalence classes over the equivalence paths that can reached by
strings up to length *n*. Long strings (relative to *n*) are unlikely
to have been joined into their appropriate equivalence class, since
they may take a path through a string longer than *n*. On the other
hand, if we just look at "equivalence classes that contain a member <=
length *k*" for some reasonable small *k*, they're probably real
elements of the monoid.

What we can do is look at how the "number of equivalence classes whose
smallest member length is <= k", and see how this converges over
time. [results.txt](results.txt) does this, as we get up to length 22:

| n  | k = 0 | 1  | 2  | 3  | 4  | 5  | 6   | 7   | 8   | 9   | 10  | 11  | 12  | 13  | 14  | 15  | 16  | 17  | 18  | 19  | 20   | 21    | 22   | 23    |
| -  | ----- | -- | -- | -- | -- | -- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | ---- | ---- | ---- | ---- | ----- |
| 19 | 1     | 4  | 10 | 22 | 40 | 70 | 112 | 148 | 172 | 190 | 214 | 267 | 331 | 378 | 438 | 537 | 568 | 668 | 878 | 2812 | 6264 |      |      |       |
| 20 | 1     | 4  | 10 | 22 | 40 | 70 | 112 | 148 | 160 | 160 | 172 | 189 | 208 | 243 | 294 | 336 | 414 | 420 | 516 | 726  | 3244 | 7873 |      |       |
| 21 | 1     | 4  | 10 | 22 | 40 | 70 | 112 | 148 | 160 | 160 | 160 | 160 | 160 | 169 | 169 | 188 | 212 | 254 | 254 | 357  | 582  | 3886 | 9965 |       |
| 22 | 1     | 4  | 10 | 22 | 40 | 70 | 112 | 148 | 160 | 160 | 160 | 160 | 160 | 160 | 160 | 160 | 160 | 166 | 186 | 186  | 295  | 553  | 4828 | 12840 |

What we can see is that for any fixed $k$, the number of equivalence
classes that have a representative <= *k* characters long converges
pretty solidly, meaning that despite lack of mathematical proof, I
think we've found all the monoid members representable by something <=
*k* characters long, and that they are genuinely distinct.

When *k* hits 8, we stop finding new equivalence classe for larger
*k*, so it looks like we've hit our 160 distinct values!

Now it *could* be that there's a much longer path that unifies two of
these strings, and a longer string that is the shortest string in its
equivalence class, but this seems... unlikely.

Ideally, I'd find proof that there are no monoid members that don't
have a representation with 8 or fewer characters, but right now I
don't.

## Generating the elements

We can then find the shortest element in each equivalence class
(there's a unique shortest in each class), to generate a set of
strings representing all the members of the monoid. This is what we've
done in [elements.txt](elements.txt).

TODO: Generate the "multiplication table".

## A note on research

It's weird, after doing compsci research, finding the OEIS entry
refering to papers and books that I can't just download. For all I
know, those resources contain all the theorems and proofs I'd need to
have done all this much more efficiently, and have a mathematically
proven result. But I couldn't easily get them, so I ended up having to
reinvent everything from scratch. Inefficient but fun, I guess?
