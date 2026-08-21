/* Table C wiring row: the .Supplier environment entry, asked what it
   renders as and what its class is -- questions any entry answers --
   and then the questions the class surface is wired by: ~id, ~class,
   ~superClass, ~superClasses, ~metaClass and ~isA(.Class). Derived
   from corpus/docs/class-set.txt by
   crates/rexx-exec/tests/gate_table_c.rs, which re-derives this file on
   every run and compares it in both directions. */
say 'entry' .Supplier
say 'class-of-entry' .Supplier~class~id
say 'id' .Supplier~id
say 'class' .Supplier~class
say 'superclass' .Supplier~superClass
say 'superclasses' .Supplier~superClasses~makeString('L', ' ')
say 'metaclass' .Supplier~metaClass
say 'isa-class' .Supplier~isA(.Class)
