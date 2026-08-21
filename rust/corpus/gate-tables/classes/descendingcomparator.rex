/* Table C wiring row: the .DescendingComparator environment entry, asked what it
   renders as and what its class is -- questions any entry answers --
   and then the questions the class surface is wired by: ~id, ~class,
   ~superClass, ~superClasses, ~metaClass and ~isA(.Class). Derived
   from corpus/docs/class-set.txt by
   crates/rexx-exec/tests/gate_table_c.rs, which re-derives this file on
   every run and compares it in both directions. */
say 'entry' .DescendingComparator
say 'class-of-entry' .DescendingComparator~class~id
say 'id' .DescendingComparator~id
say 'class' .DescendingComparator~class
say 'superclass' .DescendingComparator~superClass
say 'superclasses' .DescendingComparator~superClasses~makeString('L', ' ')
say 'metaclass' .DescendingComparator~metaClass
say 'isa-class' .DescendingComparator~isA(.Class)
