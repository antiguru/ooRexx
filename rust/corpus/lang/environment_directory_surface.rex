/* .environment answers every method Directory's own table holds, as a
 * Directory a program makes does.
 *
 * The whole-collection reads are compared as ordered output: the contents
 * come back in bucket order and LOCAL comes last, because Setup.cpp installs
 * it with setMethod and a Directory appends its method table after its
 * contents. A program-added entry lands where its hash puts it.
 *
 * The entry family upper-cases the name and the index family does not;
 * setEntry with no value removes. Removing a class name from .environment
 * leaves .NAME resolving it, because the REXX package's public classes are
 * searched first.
 */

e = .environment
say 'items    ' e~items e~isEmpty
say 'indexes  ' e~allIndexes~makeString('L', ' ')
say 'makeArray' e~makeArray~makeString('L', ' ')
line = ''
do name over e
  line = line name
end
say 'over     ' strip(line)

s = e~supplier
n = 0
do while s~available
  n = n + 1
  item = s~item
  if item~isA(.String) then shown = c2x(item)
  else shown = item~string
  say 'supplier ' n s~index shown
  s~next
end
items = e~allItems
say 'allItems ' items~items (items[items~items] == .local) (items~lastItem == .local)

say 'at       ' e['ARRAY'] e~at('ARRAY') e['array'] e~at('LOCAL')
say 'hasIndex ' e~hasIndex('ARRAY') e~hasIndex('array') e~hasIndex('LOCAL')
say 'hasItem  ' e~hasItem(.array) e~hasItem(.local) e~hasItem('no such item')
say 'index    ' e~index(.array) e~index(.local) e~index('no such item')
say 'entry    ' e~entry('array') e~entry('Local') e~hasEntry('string') e~hasEntry('nope')
say 'unknown  ' e~string e~unknown('TABLE', .array~new) e~nosuchentry

e['zzAdded'] = 'lower'
e~put('upper', 'ZZADDED')
e~setEntry('viaEntry', 'entry')
say 'added    ' e['zzAdded'] e['ZZADDED'] e['viaEntry'] e['VIAENTRY'] .zzAdded .viaEntry
say 'order    ' e~allIndexes~makeString('L', ' ')
e~setEntry('viaentry')
say 'no value ' e~hasIndex('VIAENTRY') e~items
say 'removeEnt' e~removeEntry('zzadded') e~hasIndex('ZZADDED') e~hasIndex('zzAdded')

e~setMethod('GREETING', 'return "hello from" self~objectName')
say 'setMethod' e~greeting .greeting e~items e~allIndexes~lastItem
e~unsetMethod('GREETING')
say 'unset    ' e~hasIndex('GREETING') e~items .greeting

removed = e~remove('ARRAY')
say 'remove   ' removed e~hasIndex('ARRAY') e['ARRAY'] .array e~items
e['ARRAY'] = removed
say 'restored ' e~hasIndex('ARRAY') e~allIndexes~makeString('L', ' ')
say 'removeItm' e~removeItem(.string) e~hasIndex('STRING') .string
say 'removeLoc' e~remove('LOCAL') e~hasIndex('LOCAL') e~items
e~init
say 'init     ' e~items
e~empty
say 'empty    ' e~items e~isEmpty e~allIndexes~items
