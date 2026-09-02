/* An array size the allocator cannot satisfy raises rather than dying.        */
/*                                                                            */
/* The oracle asks the allocator rather than a size limit, so this is the one  */
/* array error MaxFixedArraySize does not decide: case 5 is that limit's own   */
/* 93.959 and is what separates the two, and case 6 is the adjacent success    */
/* that says the guard did not simply refuse everything.                       */
/*                                                                            */
/* Every case is trapped, so what this compares is the condition each raises   */
/* and not a traceback carrying this file's own path.                          */
huge = 999999999999999
n = 0
signal on syntax name trapped

next:
n = n + 1
select
  when n = 1 then say n 'answered' .array~new(huge)~size
  when n = 2 then say n 'answered' .array~new(huge, 1)~size
  when n = 3 then do
    a = .array~new(0)
    a[huge] = 1
    say n 'answered' a~size
  end
  when n = 4 then do
    b = .array~new(1, 1)
    b[999999999, 999999] = 1
    say n 'answered' b~dimension
  end
  when n = 5 then say n 'answered' .array~new(100000000000000001)~size
  when n = 6 then do
    c = .array~new(3)
    c[2] = 9
    say n 'answered' c~size c[2]
  end
  otherwise signal done
end
signal next

trapped:
say n 'raised' rc'.'condition('E')
signal on syntax name trapped
signal next

done:
signal off syntax
say 'end'
