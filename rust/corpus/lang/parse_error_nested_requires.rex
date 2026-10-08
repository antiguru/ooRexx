/* A ::REQUIRES file that does not parse, or whose prologue raises, inside
   a file Routine~newFile, Method~newFile, Package~new or loadPackage loads:
   the directive's level and the loading method's level both report. */
do kind over .array~of('parse', 'run')
  call try .routine, 'newFile', 'mid_rnf_'kind'.cls'
  call try .method, 'newFile', 'mid_mnf_'kind'.cls'
  call try .package, 'new', 'mid_pkg_'kind'.cls'
  call try .context~package, 'loadPackage', 'mid_lp_'kind'.cls'
end
exit
try: signal on syntax
  r = arg(1)~send(arg(2), arg(3))
  say 'not reached'
  return
syntax:
  o = condition('O')
  say o~code o~position rc o~traceback~items o~stackframes~items
  do l over o~traceback; say '  tb:' l; end
  do f over o~stackframes; say '  sf:' f~type f~name f~line; end
  return
