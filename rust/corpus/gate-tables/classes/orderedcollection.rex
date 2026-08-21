/* Table C wiring row: the .OrderedCollection environment entry, asked what it
   renders as and what its class is -- questions any entry answers --
   and then the questions the class surface is wired by: ~id, ~class,
   ~superClass, ~superClasses, ~metaClass and ~isA(.Class). Derived
   from corpus/docs/class-set.txt by
   crates/rexx-exec/tests/gate_table_c.rs, which re-derives this file on
   every run and compares it in both directions. */
say 'entry' .OrderedCollection
say 'class-of-entry' .OrderedCollection~class~id
say 'id' .OrderedCollection~id
say 'class' .OrderedCollection~class
say 'superclass' .OrderedCollection~superClass
say 'superclasses' .OrderedCollection~superClasses~makeString('L', ' ')
say 'metaclass' .OrderedCollection~metaClass
say 'isa-class' .OrderedCollection~isA(.Class)
