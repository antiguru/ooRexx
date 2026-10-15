parse arg n
do i = 1 to n
  interpret 'x = "' || copies('abcdefgh', 20) || i || '"'
end
say length(x)
