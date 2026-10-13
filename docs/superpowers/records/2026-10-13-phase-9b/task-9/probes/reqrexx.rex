ir = .context~package~importedRoutines
say 'imported' ir~items
do n over ir~allIndexes~sort; say n; end
say 'fs' (.context~package~findRoutine('filespec') == .nil)
say filespec('n', '/a/b.c')
::requires 'REXX' library
