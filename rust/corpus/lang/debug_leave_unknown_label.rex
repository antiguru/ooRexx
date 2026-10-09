/* A LEAVE naming no enclosing loop raises 28.3 without a debug pause. */
trace ?a
do i = 1 to 2
  leave zz
end
