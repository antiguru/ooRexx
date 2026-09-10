/* decimal control plus the renderings rexxcps performs on the control value */
total=0
n = 400000
do i=1 to n
  do j=1.1 to 2.2 by 1.1
    if 17<length(j)-1 then say 'no'
    if j='foobar'     then say 'no'
    total=total+1
    end
  end
say total
