/* Table C wiring row: the .Comparable environment entry, asked what it
   renders as and what its class is -- questions any entry answers --
   and then the questions the class surface is wired by: ~id, ~class,
   ~superClass, ~superClasses, ~metaClass and ~isA(.Class). Derived
   from corpus/docs/class-set.txt by
   crates/rexx-exec/tests/gate_table_c.rs, which re-derives this file on
   every run and compares it in both directions. */
say 'entry' .Comparable
say 'class-of-entry' .Comparable~class~id
say 'id' .Comparable~id
say 'class' .Comparable~class
say 'superclass' .Comparable~superClass
say 'superclasses' .Comparable~superClasses~makeString('L', ' ')
say 'metaclass' .Comparable~metaClass
say 'isa-class' .Comparable~isA(.Class)
