/* Table C wiring row: the .String environment entry, asked what it
   renders as and what its class is -- questions any entry answers --
   and then the questions the class surface is wired by: ~id, ~class,
   ~superClass, ~superClasses, ~metaClass and ~isA(.Class). Derived
   from corpus/docs/class-set.txt by
   crates/rexx-exec/tests/gate_table_c.rs, which re-derives this file on
   every run and compares it in both directions. */
say 'entry' .String
say 'class-of-entry' .String~class~id
say 'id' .String~id
say 'class' .String~class
say 'superclass' .String~superClass
say 'superclasses' .String~superClasses~makeString('L', ' ')
say 'metaclass' .String~metaClass
say 'isa-class' .String~isA(.Class)
