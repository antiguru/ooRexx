/* Table C wiring row: the .List environment entry, asked the questions
   the class surface is wired by -- ~id, ~class, ~superClass,
   ~superClasses, ~metaClass and ~isA(.Class). Derived from
   corpus/docs/class-set.txt by crates/rexx-exec/tests/gate_table_c.rs,
   which re-derives this file on every run and compares it in both
   directions. */
say 'id' .List~id
say 'class' .List~class
say 'superclass' .List~superClass
say 'superclasses' .List~superClasses~makeString('L', ' ')
say 'metaclass' .List~metaClass
say 'isa-class' .List~isA(.Class)
