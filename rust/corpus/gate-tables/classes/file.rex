/* Table C wiring row: the .File environment entry, asked what it
   renders as and what its class is -- questions any entry answers --
   and then the questions the class surface is wired by: ~id, ~class,
   ~superClass, ~superClasses, ~metaClass and ~isA(.Class). Derived
   from corpus/docs/class-set.txt by
   crates/rexx-exec/tests/gate_table_c.rs, which re-derives this file on
   every run and compares it in both directions. */
say 'entry' .File
say 'class-of-entry' .File~class~id
say 'id' .File~id
say 'class' .File~class
say 'superclass' .File~superClass
say 'superclasses' .File~superClasses~makeString('L', ' ')
say 'metaclass' .File~metaClass
say 'isa-class' .File~isA(.Class)
