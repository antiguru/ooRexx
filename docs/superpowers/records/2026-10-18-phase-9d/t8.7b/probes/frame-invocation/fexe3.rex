.message~new(.w~new, 'two')~send
::class w
::method two
  do f over .context~stackframes
    say f~type f~name
  end
