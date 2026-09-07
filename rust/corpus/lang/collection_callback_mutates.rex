/* A collection search whose comparison runs Rexx that empties the collection.

   Every item-searching method copies the collection's contents out before it
   starts sending `==`, and every one of those sends can run a user method
   that removes the items being copied.  Under ordinary allocation the copy
   survives because nothing collects; under collect-on-every-allocation it is
   a dangling reference unless the copy is rooted.  This program is in the
   phase subset so `collect_stress` runs it. */

a = .Array~new
do i = 1 to 6
  a~append('made' || i || '-' || (i * 3))
end
say 'array hasItem' a~hasItem(.Emptier~new(a)) 'and the array now holds' a~items

b = .Array~new
do i = 1 to 6
  b~append('made' || i || '-' || (i * 5))
end
say 'array removeItem' b~removeItem(.Emptier~new(b)) 'and it now holds' b~items

c = .Array~new
do i = 1 to 6
  c~append('made' || i || '-' || (i * 11))
end
say 'array index' c~index(.Emptier~new(c)) 'and it now holds' c~items

d = .List~new
do i = 1 to 6
  zz = d~append('made' || i || '-' || (i * 13))
end
say 'list hasItem' d~hasItem(.Emptier~new(d)) 'and it now holds' d~items

/* And a comparator that empties the array it is sorting. */
e = .Array~new
do i = 1 to 4
  e~append('made' || (9 - i) || '-' || (i * 17))
end
say 'sortWith' e~sortWith(.Shrinker~new(e))~items 'and it now holds' e~items

::CLASS Emptier
::METHOD init
  expose target
  use arg target
::METHOD "=="
  expose target
  target~empty
  /* Allocate after emptying, so a collector running on every allocation has
     somewhere to run between the copy being taken and the next item being
     read from it. */
  junk = ''
  do i = 1 to 20
    junk = junk || 'j' || i
  end
  return 0

::CLASS Shrinker SUBCLASS Comparator
::METHOD init
  expose target
  use arg target
::METHOD compare
  expose target
  use arg one, two
  target~empty
  junk = ''
  do i = 1 to 20
    junk = junk || 'j' || i
  end
  return 0
