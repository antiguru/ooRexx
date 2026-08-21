/* Table C wiring row: the .InputOutputStream environment entry, asked the questions
   the class surface is wired by -- ~id, ~class, ~superClass,
   ~superClasses, ~metaClass and ~isA(.Class). Derived from
   corpus/docs/class-set.txt by crates/rexx-exec/tests/gate_table_c.rs,
   which re-derives this file on every run and compares it in both
   directions. */
say 'id' .InputOutputStream~id
say 'class' .InputOutputStream~class
say 'superclass' .InputOutputStream~superClass
say 'superclasses' .InputOutputStream~superClasses~makeString('L', ' ')
say 'metaclass' .InputOutputStream~metaClass
say 'isa-class' .InputOutputStream~isA(.Class)
