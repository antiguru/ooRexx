/* Table C wiring row: the .Orderable environment entry, asked what it
   renders as and what its class is -- questions any entry answers --
   and then the questions the class surface is wired by: ~id, ~class,
   ~superClass, ~superClasses, ~metaClass and ~isA(.Class). Derived
   from corpus/docs/class-set.txt by
   crates/rexx-exec/tests/gate_table_c.rs, which re-derives this file on
   every run and compares it in both directions. */
say 'entry' .Orderable
say 'class-of-entry' .Orderable~class~id
say 'id' .Orderable~id
say 'class' .Orderable~class
say 'superclass' .Orderable~superClass
say 'superclasses' .Orderable~superClasses~makeString('L', ' ')
say 'metaclass' .Orderable~metaClass
say 'isa-class' .Orderable~isA(.Class)
