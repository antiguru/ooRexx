/* A ::REQUIRES in code compiled from source is searched for from the
   directory and extension of the name the code was given, then from its
   parent package's. */
dir = qualify('rqd')
ok = .File~new(dir)~makeDir
call write dir'/rqd_own.cls', 'own'
call write dir'/rqd_ext.zzz', 'ext'
call write dir'/rqd_par.cls', 'parent'
f = .stream~new(dir'/lib.cls')
f~open('write replace')
f~lineout('::routine go public')
f~lineout('  use arg code')
f~lineout('  return .routine~new("bare", code, .context~package)~call')
f~close

-- The name's own directory.
-- The name's own extension.
-- A bare name has no directory, so the parent package's is searched.
go = .package~new(dir'/lib.cls')~findRoutine('GO')
say go~call(("return v()", "::requires 'rqd_par'"))
-- Neither holds the file.
signal on syntax
say .routine~new(dir'/any.zzz', ("return v()", "::requires 'rqd_none'"))~call
exit
syntax:
say 'syntax' condition('o')~code condition('o')~message
do name over .array~of('rqd_own.cls', 'rqd_ext.zzz', 'rqd_par.cls', 'lib.cls')
  ok = .File~new(dir"/"name)~delete
end
ok = .File~new(dir)~delete
exit

write: procedure
  use arg path, answer
  f = .stream~new(path)
  f~open('write replace')
  f~lineout('::routine v public')
  f~lineout('  return "'answer'"')
  f~close
  return
