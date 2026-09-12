/* `.File` over a tree this program builds in its own run directory.
   Three things are deliberate. The listing is sorted before it prints, because
   `list` answers raw `readdir` order, which belongs to the file system and not
   to the interpreter. No permission predicate appears: `canRead` and `canWrite`
   answer 1 for everything when the run is root, so they would pin the user
   rather than the code. And each file is written and closed through **one**
   stream object -- two live streams over one name is a buffering question of
   its own, and not this program's. */
zz = .File~new('tree/inner')~makeDirs
w = .Stream~new('tree/one.txt')
zz = w~arrayout(.array~of('alpha','beta'))
zz = w~close

f = .File~new('tree/one.txt')
say 'exists     ' f~exists f~isFile f~isDirectory f~isHidden
say 'length     ' f~length
say 'name       [' || f~name || ']'
d = .File~new('tree')
say 'dir        ' d~exists d~isFile d~isDirectory
names = .array~new
do n over d~list
  names~append(n)
end
say 'list       ' names~items
do n over names~sort
  say '  [' || n || ']'
end
say 'of a file  [' || f~list~string || ']'
say 'of missing [' || .File~new('tree/nosuch')~list~string || ']'

f~lastModified = .DateTime~new(2021, 7, 8, 9, 10, 11)
say 'round trip [' || .File~new('tree/one.txt')~lastModified~isoDate || ']'
say 'missing ts [' || .File~new('tree/nosuch')~lastModified~string || ']'

say 'renameTo   ' f~renameTo(.File~new('tree/two.txt'))
say 'again      ' .File~new('tree/two.txt')~renameTo(.File~new('tree/two.txt'))
say 'delete     ' .File~new('tree/two.txt')~delete
say 'delete dir ' .File~new('tree')~delete
say 'inner gone ' .File~new('tree/inner')~delete
say 'tree gone  ' .File~new('tree')~delete
say 'caseSense  ' .File~isCaseSensitive
say 'roots      ' .File~listRoots~items .File~listRoots[1]~class~id
