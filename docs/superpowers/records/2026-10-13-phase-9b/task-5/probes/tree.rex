base = directory()
call directory base'/sysfiletree'
cases = .array~of( -
  'f|', 'f|b', 'f|f', 'f|h', 'f|i', 'f|l', 'f|o', 'f|t', 'd|B', 'd|D', 'd|H', 'd|L', 'd|T', 'f|d', 'd|f', -
  '../sysfiletree/d|h', 'ROOTd|il', 'UpperFile|bdf', 'UpperDir|fd', 'upperfile|fi', 'upperfile|o', -
  'b b|f', 'd|dt', 'd|ldt', 'Upper*|bdf', 'Upper*|fd', 'Upper*|dfb', 'u*|fi', 'u*|o', 'b?b|fo', -
  '*|', '*|F', '*|D', '../sysfiletree/*|', 'ROOT*|i', '../sysfiletree/|', '.|h', './.|o', 'd/..|l', 'ROOT|i', -
  '?|h', '?|hf', '?|d', '../sysfiletree/?|ob', '../sysfiletree/?|of', '../sysfiletree/?|do', -
  '../sysfiletree*/d|', '../*/d|', '../**/d|', '[fd]|', '../sysfiletree/[fd]|', -
  'f|FSI', 'd|ds', 'f|S', 'f|sd', 'd|sf', '*|so', '*|sD', '*|s', -
  '..|o', '../|o', '/|do', '~|do', '~/|do', 'd/|o', 'd/.|o', 'd/../d|o', './d|o', '.d/|o', -
  'n*|o', 'linkdir|', 'linkdir/|o', 'dangling|', 'locked|', 'locked/|o', 'locked/*|o', -
  'sticky|', 'suid|', 'sgid|', 'pipe|', 'b b|fh', 'b b|th', 'b b|lh', '[!a-z]*|o', '[]x]*|o', '*[|o', -
  '.|so', 'd/F|o', 'D/f|oi', 'D/f|o', 'UPPERDIR|di', '[[:upper:]]*|o', 'x\*|o', '\f|o', -
  'f*|io', '*.|o', '..*|o' )
do c over cases
  parse var c spec '|' opts
  if spec~startsWith('ROOT') then spec = base'/sysfiletree/'spec~substr(5)
  drop f.
  if opts == '' then rc = SysFileTree(spec, f.)
  else rc = SysFileTree(spec, f., opts)
  a = .array~new
  do i = 1 to f.0; a~append(f.i~changestr(base, '<R>')); end
  a~sort
  say '['c'] rc' rc 'n' f.0
  do l over a; say '  'l; end
end
