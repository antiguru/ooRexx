# Wrong-type native rows that refuse naming Phase 9

A primitive's native method run on an instance of another class (`self~run(.X~method('M'))`
from a `::class t` method, or installed through `enhanced`) is Deviation 28: the oracle reads
the receiver through the wrong layout. Most such rows refuse through `Loud::receiver_class`
(owner none). These answer `Loud::native_method` instead, naming Phase 9, which tells a reader
to wait for work no phase owes. Measured at 6860d4f80+fix round 1 of Phase 6.1 Task 7
(probes `/tmp/claude-1000/p61/t7/rc6`), each rc 120:

- `ITEMS` of Directory, StringTable, Stem, Set, Bag, Relation and Table: `method "ITEMS" of
  class "T" is not implemented (Phase 9)` (`dispatch/hash.rs` `not_this_task`).
- MutableBuffer `LENGTH`: `method "LENGTH" of class "MutableBuffer" ... (Phase 9)`.
- Message `SEND`: `method "SEND" of class "Message" ... (Phase 9)`.

Class (a) by the census, outside 6.1's R8 scope. Either route the wrong-type case to
`receiver_class`, or have Task 12's Phase 9 amendment say these are not Phase 9's.
