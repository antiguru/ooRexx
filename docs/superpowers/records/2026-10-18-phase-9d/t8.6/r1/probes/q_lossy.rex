d = 'd' || 'ff'x
call SysMkDir d
call directory d
say c2x(right(directory(), 1))
address system 'pwd >/dev/null'
say 'rc' rc
