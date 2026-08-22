/* provide.xml `reqstr`'s last limb: a receiver with no makeString and no
 * NOSTRING trap gets a `string` message, whose answer is the object's default
 * name, and that string is what the context uses. So the contexts that can use
 * any string at all answer, and the ones that need a number, a label or a
 * variable symbol raise from the default name rather than from the object.
 *
 * The pair with required_string_contexts.rex is the point: the same contexts
 * with a makeString use its answer, and a build that skipped the conversion
 * entirely would pass this program and fail that one.
 *
 * Phase 5a Task 14.
 */

say .p
i = .p
say 'tail' a.i
parse value .p with w1 w2 w3
say 'parse value' w1'/'w2'/'w3
held = .p
parse var held w4
say 'parse var' w4
call sub .p
push .p
pull q
say 'pull' q
say 'length' length(.p)
say 'concat' 'x' || .p
say 'compare' ('The P class' = .p)
address (.p)
say 'address' address()

signal on syntax name trapped
n = 0

next:
n = n + 1
select
  when n = 1 then do .p
  end
  when n = 2 then do j = 1 to 9 for .p
  end
  when n = 3 then numeric digits .p
  when n = 4 then numeric fuzz .p
  when n = 5 then numeric form value .p
  when n = 6 then trace value .p
  when n = 7 then signal value .p
  when n = 8 then call (.p)
  when n = 9 then do
    list = .p
    drop (list)
  end
  otherwise signal done
end
say n 'answered'
signal next

trapped:
say n 'raised' rc'.'condition('E')
signal on syntax name trapped
signal next

done:
say 'done'
exit 0

sub:
parse arg s
say 'parse arg' s
return

::class p
