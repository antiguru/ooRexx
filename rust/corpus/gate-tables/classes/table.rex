/* Table C wiring row: the .Table environment entry, asked what it
   renders as and what its class is -- questions any entry answers --
   and then the questions the class surface is wired by: ~id, ~class,
   ~superClass, ~superClasses, ~metaClass and ~isA(.Class). Derived
   from corpus/docs/class-set.txt by
   crates/rexx-exec/tests/gate_table_c.rs, which re-derives this file on
   every run and compares it in both directions. */
say 'entry' .Table
say 'class-of-entry' .Table~class~id
say 'id' .Table~id
say 'class' .Table~class
say 'superclass' .Table~superClass
say 'superclasses' .Table~superClasses~makeString('L', ' ')
say 'metaclass' .Table~metaClass
say 'isa-class' .Table~isA(.Class)
