/* Table C wiring row: the .MutableBuffer environment entry, asked the questions
   the class surface is wired by -- ~id, ~class, ~superClass,
   ~superClasses, ~metaClass and ~isA(.Class). Derived from
   corpus/docs/class-set.txt by crates/rexx-exec/tests/gate_table_c.rs,
   which re-derives this file on every run and compares it in both
   directions. */
say 'id' .MutableBuffer~id
say 'class' .MutableBuffer~class
say 'superclass' .MutableBuffer~superClass
say 'superclasses' .MutableBuffer~superClasses~makeString('L', ' ')
say 'metaclass' .MutableBuffer~metaClass
say 'isa-class' .MutableBuffer~isA(.Class)
