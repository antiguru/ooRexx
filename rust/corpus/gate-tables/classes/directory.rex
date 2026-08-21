/* Table C wiring row: the .Directory environment entry, asked the questions
   the class surface is wired by -- ~id, ~class, ~superClass,
   ~superClasses, ~metaClass and ~isA(.Class). Derived from
   corpus/docs/class-set.txt by crates/rexx-exec/tests/gate_table_c.rs,
   which re-derives this file on every run and compares it in both
   directions. */
say 'id' .Directory~id
say 'class' .Directory~class
say 'superclass' .Directory~superClass
say 'superclasses' .Directory~superClasses~makeString('L', ' ')
say 'metaclass' .Directory~metaClass
say 'isa-class' .Directory~isA(.Class)
