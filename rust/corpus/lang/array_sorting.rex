/* Array's sort family.  All four names land on two C++ bodies -- the
   interpreter maps the unstable spellings onto the stable ones -- so there is
   one algorithm here and not two.

   The default order is NOT numeric: it sends `compareTo`, which for a string
   or an integer is a string comparison.  `1,10,100,2,9` is the whole of that
   point and a numeric sort would answer `1,2,9,10,100`.

   Each case builds its own array rather than copying one, because `~copy` is
   a separate unimplemented row. */

say 'default order' .Array~of(10, 9, 2, 100, 1)~sort~makeString('L', ',')
say 'numeric comparator',
    .Array~of(10, 9, 2, 100, 1)~sortWith(.NumericComparator~new)~makeString('L', ',')

/* Case-sensitive and caseless differ on this data and not on every data:
   upper case sorts before lower case by byte. */
say 'case matters' .Array~of('b', 'A', 'a', 'B')~sort~makeString('L', ',')
say 'caseless',
    .Array~of('b', 'A', 'a', 'B')~sortWith(.CaselessComparator~new)~makeString('L', ',')
say 'descending',
    .Array~of('b', 'A', 'a', 'B')~sortWith(.DescendingComparator~new)~makeString('L', ',')

/* Sorting is in place and answers the receiver. */
p = .Array~of('b', 'a')
q = p~sort
say 'in place' p~makeString('L', ',') 'answered' q~makeString('L', ',')

/* The unstable spellings answer what the stable ones do. */
say 'sort vs stableSort',
    .Array~of(10, 9, 2)~sort~makeString('L', ','),
    .Array~of(10, 9, 2)~stableSort~makeString('L', ',')
say 'sortWith vs stableSortWith',
    .Array~of(10, 9, 2)~sortWith(.NumericComparator~new)~makeString('L', ','),
    .Array~of(10, 9, 2)~stableSortWith(.NumericComparator~new)~makeString('L', ',')

/* A comparator of one's own is called back into from the sort. */
say 'user comparator' .Array~of(3, 1, 2)~sortWith(.MyCmp~new)~makeString('L', ',')

/* Stability: equal keys keep their original relative order. */
say 'stable' .Array~of('b1', 'a1', 'b2', 'a2')~stableSortWith(.FirstChar~new)~makeString('L', ',')

/* The default order reaches a user class's own compareTo.  The values are
   read back one by one because every K renders the same. */
line = ''
do k over .Array~of(.K~new(3), .K~new(1), .K~new(2))~sort
  line = line k~value
end
say 'user compareTo' line

/* Empty and single-element arrays sort without complaint. */
say 'empty' .Array~new~sort~items 'single' .Array~of('only')~sort~makeString('L', ',')

/* A multi-dimensional array sorts, flattened -- the sort family is not among
   the four methods that refuse one. */
m = .Array~new(2, 2)
m[1, 1] = 'd'
m[1, 2] = 'c'
m[2, 1] = 'b'
m[2, 2] = 'a'
say 'multi-dimensional' m~sort~allItems~makeString('L', ',') 'size' m~size

/* But a HOLE is a refusal rather than a skip, where `allItems` skips it. */
h = .Array~new
h[1] = 'c'
h[3] = 'a'
say 'sparse allItems' h~allItems~makeString('L', ',')
say h~sort~makeString('L', ',')

::CLASS MyCmp SUBCLASS Comparator
::METHOD compare
  use arg a, b
  return b - a

::CLASS FirstChar SUBCLASS Comparator
::METHOD compare
  use arg a, b
  return a~left(1)~compareTo(b~left(1))

::CLASS K
::METHOD init
  expose v
  use arg v
::METHOD value
  expose v
  return v
::METHOD compareTo
  expose v
  use arg other
  if v < other~value then return -1
  if v > other~value then return 1
  return 0
