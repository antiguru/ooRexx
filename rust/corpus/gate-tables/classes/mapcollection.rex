/* Table C wiring row: the .MapCollection environment entry, asked what it
   renders as and what its class is -- questions any entry answers --
   and then the questions the class surface is wired by: ~id, ~class,
   ~superClass, ~superClasses, ~metaClass and ~isA(.Class). Derived
   from corpus/docs/class-set.txt by
   crates/rexx-exec/tests/gate_table_c.rs, which re-derives this file on
   every run and compares it in both directions. */
say 'entry' .MapCollection
say 'class-of-entry' .MapCollection~class~id
say 'id' .MapCollection~id
say 'class' .MapCollection~class
say 'superclass' .MapCollection~superClass
say 'superclasses' .MapCollection~superClasses~makeString('L', ' ')
say 'metaclass' .MapCollection~metaClass
say 'isa-class' .MapCollection~isA(.Class)
