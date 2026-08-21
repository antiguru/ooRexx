/* Table C wiring row: the .Array environment entry, asked what it
   renders as and what its class is -- questions any entry answers --
   and then the questions the class surface is wired by: ~id, ~class,
   ~superClass, ~superClasses, ~metaClass and ~isA(.Class). Derived
   from corpus/docs/class-set.txt by
   crates/rexx-exec/tests/gate_table_c.rs, which re-derives this file on
   every run and compares it in both directions. */
say 'entry' .Array
say 'class-of-entry' .Array~class~id
say 'id' .Array~id
say 'class' .Array~class
say 'superclass' .Array~superClass
say 'superclasses' .Array~superClasses~makeString('L', ' ')
say 'metaclass' .Array~metaClass
say 'isa-class' .Array~isA(.Class)
