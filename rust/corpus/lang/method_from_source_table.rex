/* `Class~defineMethods` handed source text: the second caller of
 * `newMethodObject`, reached through `createMethodDictionary`
 * (classes/ClassClass.cpp:1242-:1269).
 *
 * Its table is walked by supplier, so the entry's own index is the method's
 * name and the dictionary is keyed by that name upcased -- the same pair
 * `method_from_source.rex` reads through `~define`, reached a different way.
 * Every entry goes through `newScope` twice on this path, so the object stored
 * is a copy of the one the table held, and the table's own object comes away
 * carrying the scope the first of those two calls set on it.
 *
 * The file ends untrapped on an array whose last index filled is past its last
 * item, which is the same 93.952 `~define` raises for the same array. The
 * position string is what separates the two reports: `createMethodDictionary`
 * passes `method source` where `defineMethod` passes `method`. rc 163.
 */

signal on syntax name trapped
n = 0

next:
n = n + 1
select
  when n = 1  then .methods~put('return "from a string"', 'STR')
  when n = 2  then .methods~put(('use arg q', 'return q + 1'), 'arr')
  when n = 3  then .jar~defineMethods(.methods)
  when n = 4  then say n 'string-entry' .jar~method("STR")~class .jar~method("STR")~scope~id
  when n = 5  then say n 'array-entry' .jar~method("ARR")~class .jar~method("ARR")~scope~id
  when n = 6  then say n 'directive-entry' .jar~method("Z")~scope~id
  when n = 7  then say n 'table-keeps-its-own' .methods~z~scope
  when n = 8  then say n 'annotation-empty' .jar~method("STR")~annotation('A')
  otherwise signal done
end
signal next

trapped:
say n 'raised' rc'.'condition('E')
signal on syntax name trapped
signal next

done:
signal off syntax
.methods~put(('return 1', , 'nop'), 'HOLE')
.jar~defineMethods(.methods)

::method z
  return 'unattached'

::class jar
