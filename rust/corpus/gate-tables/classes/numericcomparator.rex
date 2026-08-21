/* Table C wiring row: the .NumericComparator environment entry, asked the questions
   the class surface is wired by -- ~id, ~class, ~superClass,
   ~superClasses, ~metaClass and ~isA(.Class). Derived from
   corpus/docs/class-set.txt by crates/rexx-exec/tests/gate_table_c.rs,
   which re-derives this file on every run and compares it in both
   directions. */
say 'id' .NumericComparator~id
say 'class' .NumericComparator~class
say 'superclass' .NumericComparator~superClass
say 'superclasses' .NumericComparator~superClasses~makeString('L', ' ')
say 'metaclass' .NumericComparator~metaClass
say 'isa-class' .NumericComparator~isA(.Class)
