/* Table C wiring row: the .TimeSpan environment entry, asked what it
   renders as and what its class is -- questions any entry answers --
   and then the questions the class surface is wired by: ~id, ~class,
   ~superClass, ~superClasses, ~metaClass and ~isA(.Class). Derived
   from corpus/docs/class-set.txt by
   crates/rexx-exec/tests/gate_table_c.rs, which re-derives this file on
   every run and compares it in both directions. */
say 'entry' .TimeSpan
say 'class-of-entry' .TimeSpan~class~id
say 'id' .TimeSpan~id
say 'class' .TimeSpan~class
say 'superclass' .TimeSpan~superClass
say 'superclasses' .TimeSpan~superClasses~makeString('L', ' ')
say 'metaclass' .TimeSpan~metaClass
say 'isa-class' .TimeSpan~isA(.Class)
