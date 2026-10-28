/* A whole-number or index argument that is not a string is read as its string value. */
q = .queue~new; q~push('x'); q~push('y')
a = .array~of('a','b','c')
l = .list~of('p','q','r')
cases = .array~of("q~at(.S2~new)", "a~at(.S2~new)", "a[.S2~new]", "a~at(.array~of(.array~of(2)))", "a~at(.array~of(2))", -
  "l~at(.array~of(2))", "l~at(.S2~new)", "l~section(1, .array~of(2))~items", "l~section(1, 'x')", "q~at(.array~of(2))", -
  "'abcdef'~substr(.array~of(2),2)", "'abcdef'~left(.array~of(3))", "'abcdef'~copies(.array~of(2))", -
  "'abcdef'~substr(2, .array~of(2))", ".mutablebuffer~new('abcdef')~substr(.array~of(2), 2)", ".mutablebuffer~new('ab', .array~of(20))~string", -
  "'abcdef'~substr(.S2~new)", "'abcdef'~left(.array~of(3, 4))", "'abcdef'~left(.array~new)", "a~section(.array~of(2))~items", -
  "a~at(.array~of('2'))", "'abc'~pos('b', .array~of(1))", "a~at(.S2~new, 1)", "'abcdef'~right(.S2~new)", "q~remove(.S2~new)", -
  "a~remove(.S2~new)~string", "'abcdef'~substr(.object~new)", "a~at(.SX~new)", ".array~new(.S2~new)~size", "l~insert('n', .S2~new)", "l[.S2~new]")
do c over cases
  call one c
end
exit
one:
  use arg c
  signal on syntax
  interpret 'r = ' c
  say c '->' r
  return
syntax:
  say c '->' condition('o')~code condition('o')~message
  return
::class S2
::method string; return '2'
::class SX
::method string; return 'x'
