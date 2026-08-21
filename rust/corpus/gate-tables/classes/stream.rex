/* Table C wiring row: the .Stream environment entry, asked what it
   renders as and what its class is -- questions any entry answers --
   and then the questions the class surface is wired by: ~id, ~class,
   ~superClass, ~superClasses, ~metaClass and ~isA(.Class). Derived
   from corpus/docs/class-set.txt by
   crates/rexx-exec/tests/gate_table_c.rs, which re-derives this file on
   every run and compares it in both directions. */
say 'entry' .Stream
say 'class-of-entry' .Stream~class~id
say 'id' .Stream~id
say 'class' .Stream~class
say 'superclass' .Stream~superClass
say 'superclasses' .Stream~superClasses~makeString('L', ' ')
say 'metaclass' .Stream~metaClass
say 'isa-class' .Stream~isA(.Class)
