-- Main busy-waits on a flag a sleeping activity sets once it wakes: the
-- sleeper wakes while main is running, and main sees the flag.
f = .flag~new
f~start('setLater')
do while \f~done
end
say 'saw flag'
::class flag
::attribute done unguarded
::method init
  expose done
  done = 0
::method setLater unguarded
  call SysSleep 0.3
  say 'set'
  self~done = 1
