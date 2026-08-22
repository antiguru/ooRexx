/* A Directory's own argument refusals, which are a different set from the
 * Array's for the same three message names: the rows are AddMethod counts
 * rather than A_COUNT, so an extra argument is 93.902 from the send where an
 * Array's is 93.926 from the body, and the index is a stringArgument, so a
 * value with no string value is 88.909 and an omitted one is 88.901.
 *
 * ~put checks its item before its index, so one argument reports the index
 * missing and none reports the item missing.
 *
 * The scope in every frame line is Directory and not IdentityTable, which is
 * what says InheritInstanceMethods donates into the target's own dictionary.
 *
 * The rows that answer are the neighbouring successes, including an array as
 * an index -- an array has a string value, so it is a key and not a refusal.
 *
 * The last send is untrapped so the 88.909 text and the frame line are
 * compared as bytes. rc 168.
 */

signal on syntax name trapped
d = .environment
n = 0

next:
n = n + 1
select
  when n = 1 then say n 'answered' d~at()
  when n = 2 then say n 'answered' d[]
  when n = 3 then say n 'answered' d~at(.nil)
  when n = 4 then say n 'answered' d~at(.environment)
  when n = 5 then say n 'answered' d~at(1,2)
  when n = 6 then say n 'answered' d[1,2]
  when n = 7 then say n 'answered' d~put()
  when n = 8 then say n 'answered' d~put('an item')
  when n = 9 then say n 'answered' d~put('an item', .nil)
  when n = 10 then say n 'answered' d~put('an item', 'ZQ', 'extra')
  when n = 11 then say n 'answered' d~at((1,2))
  when n = 12 then say n 'answered' d~at('ARRAY')
  when n = 13 then say n 'answered' .local~at()
  otherwise signal done
end
signal next

trapped:
say n 'raised' rc'.'condition('E')
signal on syntax name trapped
signal next

done:
signal off syntax
say d~at(.nil)
