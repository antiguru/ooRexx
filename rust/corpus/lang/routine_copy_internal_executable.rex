/* A copy of a REXX-package routine, reached by a name .ROUTINES binds it to,
   called through ~call, or the original called through ~call: the frame of
   the call that raised answers the Routine object the call ran. */
fsr = .routines~fs
copy = fsr~copy
.routines['Y'] = copy
signal on syntax name s
call Y 'Q'
exit
s: c = condition('O')
 do f over c~stackframes
  say f~type f~name (f~executable == copy) (f~executable == fsr)
 end
signal on syntax name s2
copy~call('Q')
exit
s2: c = condition('O')
 do f over c~stackframes
  say '2' f~type f~name (f~executable == copy) (f~executable == fsr)
 end
signal on syntax name s3
fsr~call('Q')
exit
s3: c = condition('O')
 do f over c~stackframes
  say '3' f~type f~name (f~executable == copy) (f~executable == fsr)
 end
signal on syntax name s4
x = y('Q', 'a')
exit
s4: c = condition('O')
 do f over c~stackframes
  say '4' f~type f~name (f~executable == copy) (f~executable == fsr)
 end
::routine fs external 'LIBRARY REXX Filespec'
