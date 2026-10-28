o = .k~new
say 'run' o~viarun
a = .array~of(.k~new, .k~new)
a~sort
say 'hasitem' .array~of(1)~hasItem(.k~new)
say 'request' .k~new~request('ARRAY')~class
s = .set~new
s~put(.k~new)
say 'send' .message~new(.k~new, 'two')~send
say 'sendwith' .k~new~sendWith('two', .array~new)
say 'osend' .k~new~send('two')
r = .routine~new('rr', 'call show "routine"; return 0')
say 'call' r~call
say 'callwith' r~callWith(.array~new)
exit
::routine show public
  use arg label
  do f over .context~stackframes
    if f~type == 'ROUTINE' & f~name == 'SHOW' then iterate
    say '  ' label f~type '|' f~name '|' f~line
  end
::class k
::method viarun
  return self~run(.method~new('r', 'call show "run"; return 1'))
::method compareTo
  call show 'sort'
  return 0
::method '=='
  call show 'eq'
  return 0
::method makearray
  call show 'makearray'
  return .array~new
::method hashCode
  call show 'hash'
  return 'a'
::method two
  call show 'two'
  return 2
