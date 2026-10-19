/* Condition object entries: NOMETHOD and NOSTRING name the receiver as
   ADDITIONAL; a condition a LIBRARY REXX method raises names the REXX
   package with no POSITION and leads with that method's frame under CALL ON
   too; the interpreter's own library is the REXX package; RAISE PROPAGATE of
   a condition other than SYNTAX keeps the object's position; a SYNTAX code
   with no sub has MESSAGE .nil. */
signal on nomethod name n1
'object'~no_method
n1: call dump 'nomethod string'
signal on nomethod name n2
.nil~no_method
n2: call dump 'nomethod nil'
signal on any name n3
call datatype .RexxInfo
n3: call dump 'nostring'
call on notready name notready_call
x = linein('/nonexistent-t69/zz')
x = chars('/')
x = .stream~new('/')~lines
call off notready
signal on notready name s1
x = charin('/nonexistent-t69/zz')
s1: call dump 'notready signal'
signal on notready name s2
x = stream('/', 'c', 'open read')
s2: call dump 'notready propagated'
signal on syntax name s4
say .Validate~number('LENGTH', 'abc')
s4: call dump 'library syntax'
signal on syntax name s5
.stream~new('/tmp/t69-none')~charin(1, 2, 3)
s5: call dump 'library arguments'
signal on syntax name s6
raise syntax 93
s6: call dump 'bare code'
signal on syntax name s7
x = linein('/tmp/t69-none', 1, 2)
s7: call dump 'linein code'
/* The internal call's copy of the trap takes it, so what follows runs in
   that call. */
signal on notready name s3
x = open_in_routine()
s3: call dump 'notready propagated twice'
call propagate_nomethod
exit

notready_call: call dump 'notready call'; return

dump:
  o = condition('O')
  idx = o~allIndexes~sortWith(.caselessComparator~new)
  line = arg(1)':'
  do i over idx
    v = o[i]
    if i = 'STACKFRAMES' | i = 'TRACEBACK' then v = v~items
    else if i = 'PACKAGE' then v = v~name~right(5)
    else if i = 'PROGRAM' then v = filespec('N', v)
    else if \v~isA(.string) then v = v~class~id':'v~string
    line = line i'='v
  end
  say line
  do f over o~stackframes
    say '   frame' f~type filespec('N', f~name) f~line f~target~class~id
  end
  do t over o~traceback
    if t~pos('*-*') > 0 then say '   tb' t
  end
  return

open_in_routine: return stream('/', 'c', 'open read')

::routine propagate_nomethod
signal on nomethod
.nil~zork
nomethod:
raise propagate
