/* Table C wiring row: the .CaselessComparator environment entry, asked the questions
   the class surface is wired by -- ~id, ~class, ~superClass,
   ~superClasses, ~metaClass and ~isA(.Class). Derived from
   corpus/docs/class-set.txt by crates/rexx-exec/tests/gate_table_c.rs,
   which re-derives this file on every run and compares it in both
   directions. */
say 'id' .CaselessComparator~id
say 'class' .CaselessComparator~class
say 'superclass' .CaselessComparator~superClass
say 'superclasses' .CaselessComparator~superClasses~makeString('L', ' ')
say 'metaclass' .CaselessComparator~metaClass
say 'isa-class' .CaselessComparator~isA(.Class)
