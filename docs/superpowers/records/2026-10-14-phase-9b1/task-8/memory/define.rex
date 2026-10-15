parse arg n
m = .k~method('FOO')
do i = 1 to n; .k~define('BAR', m); end
say .k~new~bar
::class k
::method foo
  return 1
