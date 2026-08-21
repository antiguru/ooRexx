/* Table C wiring row: the .ColumnComparator environment entry, asked the questions
   the class surface is wired by -- ~id, ~class, ~superClass,
   ~superClasses, ~metaClass and ~isA(.Class). Derived from
   corpus/docs/class-set.txt by crates/rexx-exec/tests/gate_table_c.rs,
   which re-derives this file on every run and compares it in both
   directions. */
say 'id' .ColumnComparator~id
say 'class' .ColumnComparator~class
say 'superclass' .ColumnComparator~superClass
say 'superclasses' .ColumnComparator~superClasses~makeString('L', ' ')
say 'metaclass' .ColumnComparator~metaClass
say 'isa-class' .ColumnComparator~isA(.Class)
