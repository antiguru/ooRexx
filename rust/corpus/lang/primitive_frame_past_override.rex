/* A primitive's live frame names the primitive's own method and scope when a
   subclass override forwards to it: SORT, []=, REQUEST, and a class NEW. */
a = .myarr~of(.k~new(2), .k~new(1))
a~sort
t = .mytab~new
t[.k~new(3)] = 4
r = .myk~new(1)~request('FOO')
r = .myr~new('r', .array~of('return 1', "::requires 'pro.cls'"))
say r~class
signal on syntax
c = .myarr~of(.k~new(2), .bad~new)
c~sort
exit
syntax:
  o = condition('o')
  say 'syntax' o~code
  do f over o~stackframes; say '  cf' f~type filespec('name', f~name) f~line; end
  say o~traceback~makearray~toString('l', '/')
  exit
::routine show public
  do f over .context~stackFrames
    e = f~executable
    if f~line == .nil then say '  ' f~type f~name 'array sort' (e == .array~method('SORT')) 'myarr sort' (e == .myarr~method('SORT')) 'tab' (e == .table~method('[]=')) (e == .mytab~method('[]=')) 'req' (e == .object~method('REQUEST')) (e == .myk~method('REQUEST')) f~traceline
  end
::class k
::method init; expose v; use arg v
::method compareTo; expose v; use arg o
  call show "compare"
  return v - o~v
::method v; expose v; return v
::method hashCode; call show 'hash'; return 'x'
::class myk subclass k
::method request; say 'myk request'; forward class (super)
::method makefoo; call show 'makefoo'; return 1
::class bad
::method compareTo; raise syntax 93.900 array('boom')
::class myarr subclass array
::method sort
  forward class (super)
::class mytab subclass table
::method '[]='
  forward class (super)
::class myr subclass routine
::method new class
  forward class (super)
