/* Table C wiring row: the .StringTable environment entry, asked what it
   renders as and what its class is -- questions any entry answers --
   and then the questions the class surface is wired by: ~id, ~class,
   ~superClass, ~superClasses, ~metaClass and ~isA(.Class). Derived
   from corpus/docs/class-set.txt by
   crates/rexx-exec/tests/gate_table_c.rs, which re-derives this file on
   every run and compares it in both directions. */
say 'entry' .StringTable
say 'class-of-entry' .StringTable~class~id
say 'id' .StringTable~id
say 'class' .StringTable~class
say 'superclass' .StringTable~superClass
say 'superclasses' .StringTable~superClasses~makeString('L', ' ')
say 'metaclass' .StringTable~metaClass
say 'isa-class' .StringTable~isA(.Class)
