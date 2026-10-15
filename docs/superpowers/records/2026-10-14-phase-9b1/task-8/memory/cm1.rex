parse arg n
do i = 1 to n
  p = .Package~new('p', ('::class k', '::constant c "' || copies('abcdefgh', 20) || i || '"'))
end
say 'done'
