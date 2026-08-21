/* Table C wiring row: the .Pointer environment entry, asked what it
   renders as and what its class is -- questions any entry answers --
   and then the questions the class surface is wired by: ~id, ~class,
   ~superClass, ~superClasses, ~metaClass and ~isA(.Class). Derived
   from corpus/docs/class-set.txt by
   crates/rexx-exec/tests/gate_table_c.rs, which re-derives this file on
   every run and compares it in both directions. */
say 'entry' .Pointer
say 'class-of-entry' .Pointer~class~id
say 'id' .Pointer~id
say 'class' .Pointer~class
say 'superclass' .Pointer~superClass
say 'superclasses' .Pointer~superClasses~makeString('L', ' ')
say 'metaclass' .Pointer~metaClass
say 'isa-class' .Pointer~isA(.Class)
