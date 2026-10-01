f = .flag~new
f~start('setFlag')
do while \f~done
end
return 'ext ended'
::class flag
::attribute done unguarded
::method init
  expose done
  done = 0
::method setFlag unguarded
  self~done = 1
