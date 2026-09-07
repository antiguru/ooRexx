/* empty resets the free chain along with the entries, so the handles start
   again from zero -- unlike removing each entry, which frees its handle onto
   the stack the next append reuses.  And List's section is always a List,
   where Array's and Queue's are the receiver's own class. */
emptied = .List~of('a','b','c')
emptied~empty
say 'after empty' emptied~items emptied~append('d') emptied~append('e') emptied~append('f') emptied~append('g')

freed = .List~of('a','b','c')
freed~remove(1)
freed~remove(0)
say 'after removes' freed~append('d') freed~append('e') freed~append('f')

say 'empty answers' (emptied~empty == emptied)

say 'list' .MyList~of('a','b','c')~section(0, 2)~class~id
say 'queue' .MyQ~of('a','b','c')~section(1, 2)~class~id
say 'array' .MyArr~of('a','b','c')~section(1, 2)~class~id
say 'plain' .List~of('a','b')~section(0)~class~id

::CLASS MyList SUBCLASS List
::CLASS MyQ SUBCLASS Queue
::CLASS MyArr SUBCLASS Array
