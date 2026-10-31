/* Compare two directory observations with one controlled intervening create. */
parse arg mode
directory = .File~temporaryPath
before = .array~new
do file over directory~listFiles
  before~append(file~absolutePath)
end
before~sort
injected = .File~new('injected.txt', directory)~absolutePath
say 'mode' mode
say 'directory' directory~absolutePath
say 'before-count' before~items
do name over before
  say 'before' name
end
if mode == 'create' then do
  stream = .stream~new(injected)
  say 'write-residual' stream~charout('added')
  say 'close-result' stream~close
  say 'injected' injected
end
else say 'injected none'
spec = .File~new('*', directory)~absolutePath
result = SysFileTree(spec, found., 'O')
after = .array~new
do i = 1 to found.0
  after~append(found.i)
end
after~sort
say 'tree-rc' result
say 'after-count' found.0
do name over after
  say 'after' name
end
say 'counts-equal' (before~items == found.0)
exit 0
