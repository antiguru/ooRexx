/* Table C wiring row: the .Collection environment entry, asked what it
   renders as and what its class is -- questions any entry answers --
   and then the questions the class surface is wired by: ~id, ~class,
   ~superClass, ~superClasses, ~metaClass and ~isA(.Class). Derived
   from corpus/docs/class-set.txt by
   crates/rexx-exec/tests/gate_table_c.rs, which re-derives this file on
   every run and compares it in both directions. */
say 'entry' .Collection
say 'class-of-entry' .Collection~class~id
say 'id' .Collection~id
say 'class' .Collection~class
say 'superclass' .Collection~superClass
say 'superclasses' .Collection~superClasses~makeString('L', ' ')
say 'metaclass' .Collection~metaClass
say 'isa-class' .Collection~isA(.Class)
