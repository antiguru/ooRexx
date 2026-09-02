/* Multidimensional Array construction and indexing. `.Array~new` takes a slot
 * count, a dimension list, or a lone array holding one; `~dimension` answers
 * how many dimensions the array has with no argument and one extent with one,
 * and 0 for a dimension it does not have.
 *
 * The storage order is the discriminator and nothing else here is. A write
 * and the read that matches it miss together under any injective index
 * mapping, so only a reader of the slots in order -- `~toString` below --
 * can see which cell a subscript names, and the oracle moves the first
 * subscript fastest.
 */

/* The shapes ~new builds, and what each answers about itself. An explicit
   zero size fixes the shape where an omitted size does not. */
a = .array~new()
say 'unfixed' a~size a~items a~dimension a~dimension(1)
zero = .array~new(0)
say 'zero' zero~size zero~items zero~dimension zero~dimension(1)
c = .array~new(5)
say 'sized' c~size c~items c~dimension c~dimension(1) c~dimension(2)
say 'list' (1,2)~dimension (1,2)~dimension(1) (1,2)~dimension(2)
m = .array~new(2, 3)
say 'shape' m~size m~items m~dimension m~dimension(1) m~dimension(2) m~dimension(3)
say 'spread' .array~new((2,3))~size .array~new((2,3))~dimension

/* Both spellings of the setter reach the same cell, which is what the
   provide.xml `methodsbyclass` section states of them. */
m[2, 3] = 'first'
say 'operator' m[2, 3]
m~"[]="('second', 2, 3)
say 'message' m[2, 3]

/* The order the cells are laid out in. */
do i = 1 to 2
  do j = 1 to 3
    m[i, j] = i || j
  end
end
say 'order' m~toString('l', ' ')

/* A subscript past a dimension answers .nil on a read and grows the array on
   a write, moving what is already there. */
say 'past' m[3, 1]~isNil m[1, 4]~isNil
m[3, 1] = 'grown'
say 'grown' m~size m~items m~dimension(1) m~dimension(2)
say 'grown' m~toString('l', ' ')
m[1, 4] = 'wider'
say 'wider' m~size m~items m~dimension(1) m~dimension(2)
say 'wider' m~toString('l', ' ')

/* An array with no dimensions array takes its shape from the first
   multidimensional subscript list it is written through. */
u = .array~new()
say 'unset' u[1, 2]~isNil
u[2, 3] = 'v'
say 'set' u~size u~items u~dimension u~dimension(1) u~dimension(2) u[2, 3]

/* One dimension, where a write past the end extends rather than reshapes. */
s = (1, 2)
s[1] = 'over'
s~put('past', 5)
say 'single' s~size s~items s[1] s[5] s[3]~isNil s~dimension s~dimension(1)

/* An explicit zero size fixes the shape without fixing an extent. The one
   entry its dimensions array holds is not the extent: extending the array
   leaves ~dimension answering 1 and ~dimension(1) answering the new size. */
zero~put('v', 3)
say 'grew' zero~size zero~items zero~dimension zero~dimension(1) zero~dimension(2) zero[3]

/* A subscript converts under ARGUMENT_DIGITS, and a lone array is the
   subscript list here too. */
p = .array~new(2, 3)
p[1, 2] = 'v'
say 'converted' p[1.0, 2.0] p['  1  ', '2'] p~at((1, 2))

/* A dimension of zero is legal and makes every subscript out of bounds. */
e = .array~new(2, 0)
say 'empty' e~size e~items e~dimension e~dimension(1) e~dimension(2) e[1, 1]~isNil
