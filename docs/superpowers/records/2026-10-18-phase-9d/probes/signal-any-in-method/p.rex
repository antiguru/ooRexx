o = .t~new
do n over 'err fail lost nostr notr noval'~subwords
  o~send(n)
end
::class t
::method err
  signal on any
  "exit 1"
  say 'err none'; return
  any: say 'err' condition('c') condition('d')
::method fail
  signal on any
  address a_failure ""
  say 'fail none'; return
  any: say 'fail' condition('c') condition('d')
::method lost
  signal on any
  a = 1 / 9999999999
  say 'lost none'; return
  any: say 'lost' condition('c') condition('d')
::method nostr
  signal on any
  call datatype .nil
  say 'nostr none'; return
  any: say 'nostr' condition('c') condition('d')
::method notr
  signal on any
  call charin "/"
  say 'notr none'; return
  any: say 'notr' condition('c') condition('d')
::method noval
  signal on any
  call datatype no_value
  say 'noval none'; return
  any: say 'noval' condition('c') condition('d')
