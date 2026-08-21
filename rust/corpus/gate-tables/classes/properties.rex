/* Table C wiring row: the .Properties environment entry, asked what it
   renders as and what its class is -- questions any entry answers --
   and then the questions the class surface is wired by: ~id, ~class,
   ~superClass, ~superClasses, ~metaClass and ~isA(.Class). Derived
   from corpus/docs/class-set.txt by
   crates/rexx-exec/tests/gate_table_c.rs, which re-derives this file on
   every run and compares it in both directions. */
say 'entry' .Properties
say 'class-of-entry' .Properties~class~id
say 'id' .Properties~id
say 'class' .Properties~class
say 'superclass' .Properties~superClass
say 'superclasses' .Properties~superClasses~makeString('L', ' ')
say 'metaclass' .Properties~metaClass
say 'isa-class' .Properties~isA(.Class)
