'ln -s nowhere dangling'
call SysFileTree 'dangling', f.
say f.0 f.1
'rm dangling'
