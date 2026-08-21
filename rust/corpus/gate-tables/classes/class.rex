/* Table C wiring row: the .Class environment entry, asked what it
   renders as and what its class is -- questions any entry answers --
   and then the questions the class surface is wired by: ~id, ~class,
   ~superClass, ~superClasses, ~metaClass and ~isA(.Class). Derived
   from corpus/docs/class-set.txt by
   crates/rexx-exec/tests/gate_table_c.rs, which re-derives this file on
   every run and compares it in both directions. */
say 'entry' .Class
say 'class-of-entry' .Class~class~id
say 'id' .Class~id
say 'class' .Class~class
say 'superclass' .Class~superClass
say 'superclasses' .Class~superClasses~makeString('L', ' ')
say 'metaclass' .Class~metaClass
say 'isa-class' .Class~isA(.Class)
