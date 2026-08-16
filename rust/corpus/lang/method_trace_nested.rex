trace i
do i = 1 to 1
  say .K~outer
end

::class K

::method outer class
  trace i
  return .K~inner

::method inner class
  trace i
  return 'deep'
