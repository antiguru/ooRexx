/* ABSTRACT is applied last, after the INHERIT list is walked, so a directive
   owing both refusals gets INHERIT's: 98.909 naming the unresolvable mixin,
   not the 98.990 class_abstract_metaclass.rex measures for the same class. */
say 'main ran'

::CLASS S MIXINCLASS Class ABSTRACT INHERIT zzznotaclass
