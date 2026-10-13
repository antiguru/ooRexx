/* The cd command run in this process, as sys_process_cd runs it: the home
   directory and the tilde forms, quotes and backslash escapes, an account
   name, the null target, leading and trailing blanks, a link and the
   directory beyond its parent, and DIRECTORY() reporting the physical path
   after each. */
here = directory()
d = 'cd.d'
call SysMkDir d
call SysMkDir d'/real'
call SysMkDir d'/real/sub'
call SysMkDir d'/ab'
call SysMkDir d'/a b'
call SysMkDir d'/a"b'
call SysMkDir d'/closed'
'ln -s real/sub' d'/link'
'chmod 600' d'/closed'
call lineout d'/file', 'x'
call lineout d'/file'
base = here'/'d
oldHome = value('HOME', , 'ENVIRONMENT')
call value 'HOME', base'/ab', 'ENVIRONMENT'
me = userid()
myHome = directory('~'me)
call directory here
call t 'cd'
call t 'cd    '
call t 'cd ~'
call t 'cd ~/'
call t 'cd ~/../real'
call t 'cd ~ real'
call t 'cd ~'me
call t 'cd ~'me'/'
call t 'cd ~nosuchuser.cd'
call t 'cd ~nosuchuser.cd/x'
call t 'cd "~"'
call t 'cd ab'
call t 'cd   ab'
call t 'cd ab '
call t 'cd "ab"'
call t 'cd "a"b'
call t 'cd "a b"'
call t 'cd a\ b'
call t 'cd a\"b'
call t 'cd a\'
call t 'cd ""'
call t 'cd link'
call t 'cd link/..'
call t 'cd link/../sub'
call t 'cd link/../../ab'
call t 'cd nowhere/..'
call t 'cd file'
call t 'cd file/..'
call t 'cd closed'
call t 'cd' base'/link'
call t 'cd' base'/real/sub/../..'
say 'DIRECTORY(link)' shown(directory(base'/link'))
call directory here
call value 'HOME', '', 'ENVIRONMENT'
call t 'cd ~'
call t 'cd'
call value 'HOME', oldHome, 'ENVIRONMENT'
call directory here
'chmod 700' d'/closed'
'rm -r' d
exit

/* The command from inside the base directory: its rc and where it left the
   program. */
t:
  parse arg command
  call directory base
  address system command
  say left(shown(command), 30) rc shown(directory())
  return

shown: return changestr(me, changestr(myHome, changestr(base, arg(1), '<base>'), '<home>'), '<me>')
