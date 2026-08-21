/* Table C wiring row: the .IdentityTable environment entry, asked what it
   renders as and what its class is -- questions any entry answers --
   and then the questions the class surface is wired by: ~id, ~class,
   ~superClass, ~superClasses, ~metaClass and ~isA(.Class). Derived
   from corpus/docs/class-set.txt by
   crates/rexx-exec/tests/gate_table_c.rs, which re-derives this file on
   every run and compares it in both directions. */
say 'entry' .IdentityTable
say 'class-of-entry' .IdentityTable~class~id
say 'id' .IdentityTable~id
say 'class' .IdentityTable~class
say 'superclass' .IdentityTable~superClass
say 'superclasses' .IdentityTable~superClasses~makeString('L', ' ')
say 'metaclass' .IdentityTable~metaClass
say 'isa-class' .IdentityTable~isA(.Class)
