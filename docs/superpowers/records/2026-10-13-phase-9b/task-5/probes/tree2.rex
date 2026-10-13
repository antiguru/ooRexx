base = directory()
call directory base'/sysfiletree'
call SysMkDir 'x'||'ff'x
call SysFileCopy 'f', 'x'||'ff'x'/in'
call SysFileCopy 'f', '~nosuchuser.rexxutil'
cases = .array~of('~nosuchuser.rexxutil/bin|do', '~nosuchuser.rexxutil|o', '~nosuchuser.rexxutil/|do', -
  'f'||'00'x||'zz|o', 'n'||'ff'x'|o', 'x'||'ff'x'/*|o', 'x'||'ff'x'|o', 'x?/in|o', 'x'||'ff'x'/|so', '~|o', '..|o', 'd/../..|o', '/.|do', '/..|do', '//|do', './/d|o', 'd//|o', 'f/|o', 'f/.|o')
do c over cases
  parse var c spec '|' opts
  drop f.
  rc = SysFileTree(spec, f., opts)
  a = .array~new
  do i = 1 to f.0; a~append(f.i~changestr(base, '<R>')); end
  a~sort
  say '['c~translate('?', '00'x)'] rc' rc 'n' f.0
  do l over a; say '  'l; end
end
call value 'HOME', 'rel', 'ENVIRONMENT'
do c over .array~of('~|o', '~/|o', '~/x|o')
  parse var c spec '|' opts
  drop f.
  rc = SysFileTree(spec, f., opts)
  say '[HOME=rel 'c'] rc' rc 'n' f.0
  do i = 1 to f.0; say '  'f.i~changestr(base, '<R>'); end
end
call directory '/'
do c over .array~of('..|do', '.|do', 'bin|o')
  parse var c spec '|' opts
  drop f.
  rc = SysFileTree(spec, f., opts)
  say '[cwd=/ 'c'] rc' rc 'n' f.0
  a = .array~new
  do i = 1 to f.0; a~append(f.i); end
  a~sort
  do l over a; say '  'l; end
end
call directory base'/sysfiletree'
call SysFileDelete 'x'||'ff'x'/in'
call SysRmDir 'x'||'ff'x
call SysFileDelete '~nosuchuser.rexxutil'
