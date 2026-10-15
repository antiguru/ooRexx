parse arg n
do i = 1 to n
  p = .Package~new('p', ('::class k', '::constant c "' || copies('abcdefgh', 20) || i || '"'))
  x = p~classes['K']~new~c
end
say length(x)
