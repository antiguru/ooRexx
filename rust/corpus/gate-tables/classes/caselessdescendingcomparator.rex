/* Table C wiring row: the .CaselessDescendingComparator environment entry, asked the questions
   the class surface is wired by -- ~id, ~class, ~superClass,
   ~superClasses, ~metaClass and ~isA(.Class). Derived from
   corpus/docs/class-set.txt by crates/rexx-exec/tests/gate_table_c.rs,
   which re-derives this file on every run and compares it in both
   directions. */
say 'id' .CaselessDescendingComparator~id
say 'class' .CaselessDescendingComparator~class
say 'superclass' .CaselessDescendingComparator~superClass
say 'superclasses' .CaselessDescendingComparator~superClasses~makeString('L', ' ')
say 'metaclass' .CaselessDescendingComparator~metaClass
say 'isa-class' .CaselessDescendingComparator~isA(.Class)
