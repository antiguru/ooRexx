/* Table C wiring row: the .Monitor environment entry, asked what it
   renders as and what its class is -- questions any entry answers --
   and then the questions the class surface is wired by: ~id, ~class,
   ~superClass, ~superClasses, ~metaClass and ~isA(.Class). Derived
   from corpus/docs/class-set.txt by
   crates/rexx-exec/tests/gate_table_c.rs, which re-derives this file on
   every run and compares it in both directions. */
say 'entry' .Monitor
say 'class-of-entry' .Monitor~class~id
say 'id' .Monitor~id
say 'class' .Monitor~class
say 'superclass' .Monitor~superClass
say 'superclasses' .Monitor~superClasses~makeString('L', ' ')
say 'metaclass' .Monitor~metaClass
say 'isa-class' .Monitor~isA(.Class)
