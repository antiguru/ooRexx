/* Table C wiring row: the .EventSemaphore environment entry, asked the questions
   the class surface is wired by -- ~id, ~class, ~superClass,
   ~superClasses, ~metaClass and ~isA(.Class). Derived from
   corpus/docs/class-set.txt by crates/rexx-exec/tests/gate_table_c.rs,
   which re-derives this file on every run and compares it in both
   directions. */
say 'id' .EventSemaphore~id
say 'class' .EventSemaphore~class
say 'superclass' .EventSemaphore~superClass
say 'superclasses' .EventSemaphore~superClasses~makeString('L', ' ')
say 'metaclass' .EventSemaphore~metaClass
say 'isa-class' .EventSemaphore~isA(.Class)
