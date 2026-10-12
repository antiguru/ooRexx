/* .METHODS, .ROUTINES and .RESOURCES are StringTables whose hash methods
   answer in the store's own order, and writes into them are kept. */
say .methods~class .methods~items .methods~isEmpty
s = ''; do i over .methods~allIndexes; s = s i; end; say s
s = ''; do i over .methods; s = s i; end; say s
say .methods~hasIndex('ZETA') .methods~hasIndex('zeta') .methods~at('ALPHA')~class
say .methods~allItems~items .methods~supplier~index (.methods == .methods)
say .methods~index(.methods['BETA'])
say .routines~items .resources~items
s = ''; do i over .routines~allIndexes; s = s i; end; say s
.context~package~addRoutine('NEWR', .routine~new('newr', 'return 1'))
say .routines~items .routines~hasIndex('NEWR')
s = ''; do i over .routines~allIndexes; s = s i; end; say s
say .resources~at('RES')~items .resources~res~items
.methods['EXTRA'] = 5
say .methods~items .methods~hasIndex('EXTRA') .methods~extra
exit
::method zeta
::method alpha
::method beta
::method gamma
::method delta
::routine r1
::routine q2
::resource res
line1
line2
::END
