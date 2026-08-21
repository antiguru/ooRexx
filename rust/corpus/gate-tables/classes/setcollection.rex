/* Table C wiring row: the .SetCollection environment entry, asked what it
   renders as and what its class is -- questions any entry answers --
   and then the questions the class surface is wired by: ~id, ~class,
   ~superClass, ~superClasses, ~metaClass and ~isA(.Class). Derived
   from corpus/docs/class-set.txt by
   crates/rexx-exec/tests/gate_table_c.rs, which re-derives this file on
   every run and compares it in both directions. */
say 'entry' .SetCollection
say 'class-of-entry' .SetCollection~class~id
say 'id' .SetCollection~id
say 'class' .SetCollection~class
say 'superclass' .SetCollection~superClass
say 'superclasses' .SetCollection~superClasses~makeString('L', ' ')
say 'metaclass' .SetCollection~metaClass
say 'isa-class' .SetCollection~isA(.Class)
