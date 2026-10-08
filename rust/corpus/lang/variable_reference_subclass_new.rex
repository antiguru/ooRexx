/* NEW on a VariableReference subclass: 93.967 naming the subclass. */
say .k~new~go
::class k subclass variablereference
::method go
self~setMethod('m', 'return 42'); return self~m
