parse arg n
do i = 1 to n
  p = .Package~new('p', ('::class k', '::method a', '::method b'))
end
say 'done'
