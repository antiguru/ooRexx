/* Table C wiring row: the .VariableReference environment entry, asked what it
   renders as and what its class is -- questions any entry answers --
   and then the questions the class surface is wired by: ~id, ~class,
   ~superClass, ~superClasses, ~metaClass and ~isA(.Class). Derived
   from corpus/docs/class-set.txt by
   crates/rexx-exec/tests/gate_table_c.rs, which re-derives this file on
   every run and compares it in both directions. */
say 'entry' .VariableReference
say 'class-of-entry' .VariableReference~class~id
say 'id' .VariableReference~id
say 'class' .VariableReference~class
say 'superclass' .VariableReference~superClass
say 'superclasses' .VariableReference~superClasses~makeString('L', ' ')
say 'metaclass' .VariableReference~metaClass
say 'isa-class' .VariableReference~isA(.Class)
