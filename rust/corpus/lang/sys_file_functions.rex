/* The `REXXUTIL` file-system routines, and the codes they answer with.

   The three deletions do not share one rule. `SysFileDelete` checks that it
   may write before it unlinks and reports that check's own failure, so a name
   that is not there answers 13 rather than the 2 the unlink would give;
   `SysRmDir` is `remove`, which answers the raw errno and takes a plain file
   as readily as an empty directory; `SysMkDir` is `mkdir` and answers its
   errno too.

   `SysFileTree` reports fully-qualified names, so the lines here are cut back
   to their last component, and the order a directory is read in is not the
   sorted one, so they are sorted before printing. */

call SysMkDir 'work'
call directory 'work'

call lineout 'fa.txt', 'a'; call stream 'fa.txt', 'c', 'close'
call lineout 'fb.txt', 'bb'; call stream 'fb.txt', 'c', 'close'
call lineout 'f9.txt', 'ccc'; call stream 'f9.txt', 'c', 'close'
call SysMkDir 'nest'
call lineout 'nest/deep.txt', 'd'; call stream 'nest/deep.txt', 'c', 'close'

say 'exists file  ' SysFileExists('fa.txt')
say 'exists dir   ' SysFileExists('nest')
say 'exists gone  ' SysFileExists('nothing')
say 'exists empty ' SysFileExists('')
say 'isfile file  ' SysIsFile('fa.txt')
say 'isfile dir   ' SysIsFile('nest')

say 'tree letters ' tree('f[a-z].txt', 'FO')
say 'tree negated ' tree('f[!a-z].txt', 'FO')
say 'tree any     ' tree('f?.txt', 'FO')
say 'tree dirs    ' tree('*', 'DO')
say 'tree deep    ' tree('*.txt', 'FOS')
say 'tree literal ' tree('nest/deep.txt', 'FO')
say 'tree glob dir' tree('ne*/deep.txt', 'FO')
say 'tree nomatch ' tree('none*.zzz', 'FO')

say 'mkdir again  ' SysMkDir('nest')
say 'mkdir nested ' SysMkDir('no/such/deep')
say 'rmdir full   ' SysRmDir('nest')
say 'delete dir   ' SysFileDelete('nest')
say 'rmdir file   ' SysRmDir('f9.txt')
say 'f9 gone      ' SysFileExists('f9.txt')
say 'delete file  ' SysFileDelete('fb.txt')
say 'delete again ' SysFileDelete('fb.txt')
say 'rmdir gone   ' SysRmDir('nothing')

say 'sleep        ' SysSleep(0)
say 'version pair ' (SysVersion() == SysLinVer())
say 'version words' words(SysVersion())

exit 0

/* The matches as bare names, sorted, so neither the run directory nor the
   order the file system reports reaches the output. */
tree: procedure
  use arg spec, opts
  drop found.
  code = SysFileTree(spec, 'found.', opts)
  names = ''
  do i = 1 to found.0
    one = found.i
    do while pos('/', one) > 0
      parse var one . '/' one
    end
    names = names one
  end
  return 'rc='code 'n='found.0 sorted(names)

sorted: procedure
  use arg text
  list = .array~new
  do while text \== ''
    parse var text word text
    list~append(word)
  end
  do i = 1 to list~items - 1
    do j = i + 1 to list~items
      if list[j] < list[i] then do
        hold = list[i]; list[i] = list[j]; list[j] = hold
      end
    end
  end
  out = ''
  do i = 1 to list~items
    out = out list[i]
  end
  return out
