/* Table C wiring row: the .InvertingComparator environment entry, asked what it
   renders as and what its class is -- questions any entry answers --
   and then the questions the class surface is wired by: ~id, ~class,
   ~superClass, ~superClasses, ~metaClass and ~isA(.Class). Derived
   from corpus/docs/class-set.txt by
   crates/rexx-exec/tests/gate_table_c.rs, which re-derives this file on
   every run and compares it in both directions. */
say 'entry' .InvertingComparator
say 'class-of-entry' .InvertingComparator~class~id
say 'id' .InvertingComparator~id
say 'class' .InvertingComparator~class
say 'superclass' .InvertingComparator~superClass
say 'superclasses' .InvertingComparator~superClasses~makeString('L', ' ')
say 'metaclass' .InvertingComparator~metaClass
say 'isa-class' .InvertingComparator~isA(.Class)
