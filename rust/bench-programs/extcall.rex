/* An external routine from liborxfunction called in a loop: the cost of one
   native call and its return. The library is found through LD_LIBRARY_PATH. */
n = 3000000
t = 0
do i = 1 to n
    t = TestIntArg(i)
end
say t

::requires 'orxfunction' LIBRARY
