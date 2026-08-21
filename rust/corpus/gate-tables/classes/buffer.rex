/* Table C wiring row: the .Buffer environment entry, asked the questions
   the class surface is wired by -- ~id, ~class, ~superClass,
   ~superClasses, ~metaClass and ~isA(.Class). Derived from
   corpus/docs/class-set.txt by crates/rexx-exec/tests/gate_table_c.rs,
   which re-derives this file on every run and compares it in both
   directions. */
say 'id' .Buffer~id
say 'class' .Buffer~class
say 'superclass' .Buffer~superClass
say 'superclasses' .Buffer~superClasses~makeString('L', ' ')
say 'metaclass' .Buffer~metaClass
say 'isa-class' .Buffer~isA(.Class)
