parse arg n
do i = 1 to n; m = .Method~new('m', 'return 1'); m~setPrivate; end
say 'done'
