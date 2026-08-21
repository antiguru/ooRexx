/* Table C wiring row: the .MutexSemaphore environment entry, asked what it
   renders as and what its class is -- questions any entry answers --
   and then the questions the class surface is wired by: ~id, ~class,
   ~superClass, ~superClasses, ~metaClass and ~isA(.Class). Derived
   from corpus/docs/class-set.txt by
   crates/rexx-exec/tests/gate_table_c.rs, which re-derives this file on
   every run and compares it in both directions. */
say 'entry' .MutexSemaphore
say 'class-of-entry' .MutexSemaphore~class~id
say 'id' .MutexSemaphore~id
say 'class' .MutexSemaphore~class
say 'superclass' .MutexSemaphore~superClass
say 'superclasses' .MutexSemaphore~superClasses~makeString('L', ' ')
say 'metaclass' .MutexSemaphore~metaClass
say 'isa-class' .MutexSemaphore~isA(.Class)
