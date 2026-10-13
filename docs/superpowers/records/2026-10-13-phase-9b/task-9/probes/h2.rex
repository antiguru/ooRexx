signal on syntax
do n over .array~of('HANDLE:7', 'handle:0', 'HANDLE:abc', 'HANDLE:', 'HANDLE: 0', 'HANDLE:2')
  s = .Stream~new(n)
  say n 'init ok' s~state
  say n 'qs' s~query('streamtype') 'qh [' || s~query('handle') || ']'
  say n 'open' s~open('read')
  say n 'l [' || s~linein || ']' s~state s~description
  say n 'qh [' || s~query('handle') || '] qs' s~query('streamtype') 'qe [' || s~query('exists') || ']'
  say n 'close' s~close
end
exit
syntax:
  say 'syntax' condition('o')~code condition('o')~message
