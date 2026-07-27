/* Method-dispatch dimension: tight message-send loop, one object. */
n = 5000000
c = .counter~new
do i = 1 to n
    c~bump
end
say c~total

::class counter
::method init
    expose total
    total = 0
::method bump
    expose total
    total = total + 1
::method total
    expose total
    return total
