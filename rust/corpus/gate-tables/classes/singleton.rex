/* Table C wiring row: the .Singleton environment entry, asked what it
   renders as and what its class is -- questions any entry answers --
   and then the questions the class surface is wired by: ~id, ~class,
   ~superClass, ~superClasses, ~metaClass and ~isA(.Class). Derived
   from corpus/docs/class-set.txt by
   crates/rexx-exec/tests/gate_table_c.rs, which re-derives this file on
   every run and compares it in both directions. */
say 'entry' .Singleton
say 'class-of-entry' .Singleton~class~id
say 'id' .Singleton~id
say 'class' .Singleton~class
say 'superclass' .Singleton~superClass
say 'superclasses' .Singleton~superClasses~makeString('L', ' ')
say 'metaclass' .Singleton~metaClass
say 'isa-class' .Singleton~isA(.Class)
