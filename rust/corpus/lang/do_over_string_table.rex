/* `DO name OVER` a `StringTable`, which is `requestArray` answering
 * `allIndexes()`. Phase 5a Task 23.
 *
 * **Nothing here depends on the order the indexes arrive in**, and that is
 * deliberate rather than incidental: this crate iterates in sorted key order
 * where the oracle iterates its hash table's bucket order, so a row that
 * printed an index as it arrived would pin an order the oracle does not
 * share. What every row asks instead is a property of the whole pass -- how
 * many indexes there were, and that each one reads back the value the table
 * holds for it.
 *
 * `.methods`, `.routines`, `.resources` and a package's `~publicClasses` are
 * the `StringTable`s a program can reach. A file with no directive of a kind
 * gets that name's own text instead of a table, and `DO OVER` a string
 * iterates once yielding the string, which is what the `.resources` row asks.
 *
 * `FOR` bounds the pass independently of how many indexes there are, and
 * `LEAVE` ends it early.
 *
 * A `StringTable` and not an `IdentityTable`, and that is a rule rather than a
 * choice: the bucket a key lands in is its `getHashValue`, which is content
 * for a string and the object's address for anything else, so an object-keyed
 * collection iterates in a different order on each oracle run. `README.md`'s
 * determinism rule has the measurement and the prohibition.
 */

p = .context~package
n = 0
ids = ''
do name over p~publicClasses
  n = n + 1
  ids = ids || p~publicClasses[name]~id
end
say 'public classes' n
say 'ids sorted' sorted(ids)

m = 0
names = ''
do name over .methods
  m = m + 1
  names = names || name
end
say 'unattached methods' m
say 'names sorted' sorted(names)

f = 0
do name over p~publicClasses for 2
  f = f + 1
end
say 'for two' f

l = 0
do name over p~publicClasses
  l = l + 1
  leave
end
say 'after leave' l

do e over .resources
  say 'resources' e
end

do e over 'abc'
  say 'string' e
end

::method zz
::method yy

::class Cee public
::class Aay public
::class Bee public

::routine sorted
  use arg s
  out = ''
  do while s \== ''
    least = substr(s, 1, 1)
    do i = 2 to length(s)
      if substr(s, i, 1) < least then least = substr(s, i, 1)
    end
    out = out || least
    at = pos(least, s)
    s = substr(s, 1, at - 1) || substr(s, at + 1)
  end
  return out
