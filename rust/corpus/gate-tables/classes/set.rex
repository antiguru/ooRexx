/* Table C wiring row: the .Set environment entry, asked what it
   renders as and what its class is -- questions any entry answers --
   and then the questions the class surface is wired by: ~id, ~class,
   ~superClass, ~superClasses, ~metaClass and ~isA(.Class). Derived
   from corpus/docs/class-set.txt by
   crates/rexx-exec/tests/gate_table_c.rs, which re-derives this file on
   every run and compares it in both directions. */
say 'entry' .Set
say 'class-of-entry' .Set~class~id
say 'id' .Set~id
say 'class' .Set~class
say 'superclass' .Set~superClass
say 'superclasses' .Set~superClasses~makeString('L', ' ')
say 'metaclass' .Set~metaClass
say 'isa-class' .Set~isA(.Class)
