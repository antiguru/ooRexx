current = directory()
call SysMkDir 'test_command_cd'
call SysMkDir 'test_command cd'
cd = 'cd'
call directory current
cd 'test_command_cd'
say 1 rc directory() SysGetErrorText(rc)
call directory current
cd '"test_command_cd"'
say 2 rc directory()
call directory current
cd '"test_command cd"'
say 3 rc directory()
call directory current
cd current'/test_command_cd'
say 4 rc directory()
call directory current
cd '"'current'/test_command cd"'
say 5 rc directory()
cd ".."
say 6 rc (directory() == current)
cd "."
say 7 rc (directory() == current)
"cd non-existing"
say 8 rc (directory() == current) SysGetErrorText(rc)
"cd ~"
say 9 rc (directory() == value('HOME',,'ENVIRONMENT'))
call directory current
call SysRmDir 'test_command_cd'
call SysRmDir 'test_command cd'
