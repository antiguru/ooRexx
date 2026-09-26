parse arg n kind
t = 0
do i = 1 to n
  if kind = 'int' then t = t + CStr(i)
  else if kind = 'str' then t = t + CStr('k'i)
  else t = t + length('k'i)
end
say n kind t
::requires 'rr' LIBRARY
