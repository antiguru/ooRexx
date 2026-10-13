call SysMkDir 'loop.d'
call SysMkDir 'loop.d/s'
call charout 'loop.d/a', ''
call charout 'loop.d/a'
'ln -s .. loop.d/s/up'
call SysFileTree 'loop.d/*', h., 's'
say h.0
do i = 1 to h.0
  if countstr('/up', h.i) >= 40 then say subword(h.i, 3, 2)
end
'rm -rf loop.d'
