do n over .array~of('STDIN', 'HANDLE:0')
  s = .Stream~new(n)
  say n 'init' s~state s~description 'qs' s~query('streamtype') 'qh [' || s~query('handle') || '] qe [' || s~query('exists') || '] q [' || s~qualify || '] str' s~string
  say n 'size [' || s~query('size') || '] chars' s~chars 'lines' s~lines
  say n 'open' s~open('read') s~state 'qs' s~query('streamtype') 'qh [' || s~query('handle') || '] qe [' || s~query('exists') || '] q [' || s~qualify || ']'
  say n 'l [' || s~linein || ']' s~state s~description
  say n 'close' s~close s~state 'qh [' || s~query('handle') || '] qs' s~query('streamtype')
  say n 'open2' s~open s~state
  say n 'close2' s~close
end
