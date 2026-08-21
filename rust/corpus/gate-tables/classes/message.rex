/* Table C wiring row: the .Message environment entry, asked what it
   renders as and what its class is -- questions any entry answers --
   and then the questions the class surface is wired by: ~id, ~class,
   ~superClass, ~superClasses, ~metaClass and ~isA(.Class). Derived
   from corpus/docs/class-set.txt by
   crates/rexx-exec/tests/gate_table_c.rs, which re-derives this file on
   every run and compares it in both directions. */
say 'entry' .Message
say 'class-of-entry' .Message~class~id
say 'id' .Message~id
say 'class' .Message~class
say 'superclass' .Message~superClass
say 'superclasses' .Message~superClasses~makeString('L', ' ')
say 'metaclass' .Message~metaClass
say 'isa-class' .Message~isA(.Class)
