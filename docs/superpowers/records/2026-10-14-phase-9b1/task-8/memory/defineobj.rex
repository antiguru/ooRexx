parse arg n
m = .object~method('STRING')
do i = 1 to n; .k~delete('BAR'); end
say 'done'
::class k
