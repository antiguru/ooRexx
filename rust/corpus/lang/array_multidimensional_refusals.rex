/* Every way a multidimensional Array subscript list, or an `.Array~new`
 * argument, can be wrong. The errors are not interchangeable and which one
 * arrives says which check raised it: 93.925 and 93.926 count the array's own
 * dimensions where the single-dimensional 93.926 counts 1; 93.924 rejects a
 * subscript of a multidimensional index and names no position where 93.907
 * rejects the one subscript of a single-dimensional index and does name one;
 * 93.906 rejects a size or a dimension; 93.959 bounds the element count; and
 * 93.903 is an omitted subscript, whose position counts from the value that
 * `[]=` takes ahead of the list.
 *
 * The rows that answer are what stops "refuse every subscript" from passing.
 * The last send is untrapped so its text and frame line are compared as
 * bytes. rc 163.
 */

signal on syntax name trapped
m = .array~new(2, 3)
one = (1, 2)
zero = .array~new(0)
n = 0

next:
n = n + 1
select
  when n = 1 then say n 'answered' m[1]
  when n = 2 then say n 'answered' m[1,2,3]
  when n = 3 then say n 'answered' m[0,1]
  when n = 4 then say n 'answered' m[1,0]
  when n = 5 then say n 'answered' m['x',1]
  when n = 6 then say n 'answered' m[1.5,1]
  when n = 7 then say n 'answered' m[.array,1]
  when n = 8 then say n 'answered' m[,2]
  when n = 9 then say n 'answered' m[1,]
  when n = 10 then say n 'answered' m~dimension(0)
  when n = 11 then say n 'answered' m~dimension(1,2)
  when n = 12 then say n 'answered' one[1,2]
  when n = 13 then say n 'answered' zero[2,3]
  when n = 14 then say n 'answered' m['1e1',1]
  when n = 15 then say n 'answered' m~dimension(3)
  when n = 16 then say n 'answered' m~at((1,2))
  when n = 17 then do
    w = .array~new(2, 3)
    w~put('v')
    say n 'answered' w~items
  end
  when n = 18 then do
    w = .array~new(2, 3)
    w~put(,1,2)
    say n 'answered' w~items
  end
  when n = 19 then do
    w = .array~new(2, 3)
    w[,2] = 'v'
    say n 'answered' w~items
  end
  when n = 20 then do
    w = .array~new(2, 3)
    w[0,1] = 'v'
    say n 'answered' w~items
  end
  when n = 21 then do
    w = (1, 2)
    w~put('v',0)
    say n 'answered' w~items
  end
  when n = 22 then do
    w = .array~new(2, 3)
    w[1] = 'v'
    say n 'answered' w~items
  end
  when n = 23 then do
    w = .array~new(2, 3)
    w[1,2,3] = 'v'
    say n 'answered' w~items
  end
  when n = 24 then do
    w = .array~new(0)
    w[2,3] = 'v'
    say n 'answered' w~size
  end
  when n = 25 then say n 'answered' .array~new(-1)~size
  when n = 26 then say n 'answered' .array~new('x')~size
  when n = 27 then say n 'answered' .array~new(2,-1)~size
  when n = 28 then say n 'answered' .array~new(2,'-1.0')~size
  when n = 29 then say n 'answered' .array~new((,3))~size
  when n = 30 then say n 'answered' .array~new(100000000000000001)~size
  when n = 31 then say n 'answered' .array~new(2,3)~size
  otherwise signal done
end
signal next

trapped:
say n 'raised' rc'.'condition('E')
signal on syntax name trapped
signal next

done:
signal off syntax
say m[1]
