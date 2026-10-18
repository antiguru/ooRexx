/* A ::REQUIRES in code compiled from source is searched for from the
   directory and extension of the name the code was given, then from its
   parent package's. */
dir = qualify('rqd')
ok = .File~new(dir)~makeDir
call write dir'/rqd_own.cls', 'own'
call write dir'/rqd_ext.zzz', 'ext'
call write dir'/rqd_par.cls', 'parent'
call write dir'/rqd_cwd.cls', 'parent directory'
call write 'rqd_cwd.cls', 'current directory'
call write dir'/.hid.cls', 'hidden'
f = .stream~new(dir'/lib.cls')
f~open('write replace')
f~lineout('::routine go public')
f~lineout('  use arg code')
f~lineout('  return .routine~new("bare", code, .context~package)~call')
f~close

-- The name's own directory.
say .routine~new(dir'/any.zzz', ("return v()", "::requires 'rqd_own'"))~call
-- The name's own extension.
say .routine~new(dir'/any.zzz', ("return v()", "::requires 'rqd_ext'"))~call
-- A bare name has no directory, so the parent package's is searched.
go = .package~new(dir'/lib.cls')~findRoutine('GO')
say go~call(("return v()", "::requires 'rqd_par'"))
-- The current directory comes before the parent package's directory.
say go~call(("return v()", "::requires 'rqd_cwd'"))
-- A dot starting the last component after a directory starts an extension,
-- so no extension is appended and rqd/.hid.cls is not tried.
say try(dir, ("return v()", "::requires 'rqd/.hid'"))
-- Neither holds the file.
say try(dir, ("return v()", "::requires 'rqd_none'"))
do name over .array~of('rqd_own.cls', 'rqd_ext.zzz', 'rqd_par.cls', -
    'rqd_cwd.cls', '.hid.cls', 'lib.cls')
  ok = .File~new(dir'/'name)~delete
end
ok = .File~new('rqd_cwd.cls')~delete
ok = .File~new(dir)~delete
exit

try: procedure
  use arg dir, code
  signal on syntax
  return .routine~new(dir'/any.zzz', code)~call
syntax:
  return 'syntax' condition('o')~code condition('o')~message

write: procedure
  use arg path, answer
  f = .stream~new(path)
  f~open('write replace')
  f~lineout('::routine v public')
  f~lineout('  return "'answer'"')
  f~close
  return
