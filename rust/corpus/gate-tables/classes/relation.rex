/* Table C wiring row: the .Relation environment entry, asked the questions
   the class surface is wired by -- ~id, ~class, ~superClass,
   ~superClasses, ~metaClass and ~isA(.Class). Derived from
   corpus/docs/class-set.txt by crates/rexx-exec/tests/gate_table_c.rs,
   which re-derives this file on every run and compares it in both
   directions. */
say 'id' .Relation~id
say 'class' .Relation~class
say 'superclass' .Relation~superClass
say 'superclasses' .Relation~superClasses~makeString('L', ' ')
say 'metaclass' .Relation~metaClass
say 'isa-class' .Relation~isA(.Class)
