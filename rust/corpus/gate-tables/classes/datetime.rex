/* Table C wiring row: the .DateTime environment entry, asked what it
   renders as and what its class is -- questions any entry answers --
   and then the questions the class surface is wired by: ~id, ~class,
   ~superClass, ~superClasses, ~metaClass and ~isA(.Class). Derived
   from corpus/docs/class-set.txt by
   crates/rexx-exec/tests/gate_table_c.rs, which re-derives this file on
   every run and compares it in both directions. */
say 'entry' .DateTime
say 'class-of-entry' .DateTime~class~id
say 'id' .DateTime~id
say 'class' .DateTime~class
say 'superclass' .DateTime~superClass
say 'superclasses' .DateTime~superClasses~makeString('L', ' ')
say 'metaclass' .DateTime~metaClass
say 'isa-class' .DateTime~isA(.Class)
