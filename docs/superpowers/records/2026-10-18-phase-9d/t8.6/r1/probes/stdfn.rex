do n over 'STDIN' 'STDOUT' 'STDERR:' 'stdin:'
  say n stream(n,'c','query streamtype') '['stream(n,'c','query handle')']' '['stream(n,'c','query exists')']' stream(n,'s') '['stream(n,'c','query size')']'
end
say lines('STDIN') chars('STDIN')
call lineout 'STDOUT', 'x'
do n over 'STDIN' 'STDOUT'
  say n stream(n,'c','query streamtype') '['stream(n,'c','query handle')']' '['stream(n,'c','query exists')']' stream(n,'s')
end
s = .stream~new('STDOUT'); say s~query('streamtype') s~state; s~lineout('y'); say s~query('streamtype') s~state s~query('handle'); s~close; say s~query('streamtype') s~state
s = .stream~new('STDERR'); say s~open('write nobuffer') s~query('handle') s~query('exists')
s = .stream~new('STDIN'); say s~query('position') s~seek('1') s~state
s = .stream~new('HANDLE:0'); say s~query('streamtype') '['s~query('handle')']'
