/* .rexxinfo~executable and ~libraryPath answer a File each.  The paths
   differ between interpreters, so only properties both hold are shown. */
x = .rexxinfo~executable
say x~class x~isFile x~exists x~isDirectory
say (.rexxinfo~executable == x) (x~absolutePath == .rexxinfo~executable~absolutePath)
l = .rexxinfo~libraryPath
say l~class l~isDirectory l~exists
signal on syntax
c = .rexxinfo~copy
say 'not reached'
exit
syntax:
  say condition('O')['CODE'] condition('O')['MESSAGE']
