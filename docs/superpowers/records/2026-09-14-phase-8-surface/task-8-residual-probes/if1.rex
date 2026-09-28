call r
say .routine~new('rr', 'do f over .context~stackframes; say f~type "["f~name"]"; end; return 3')~call
exit
::routine r
  nop
  x = 1
  interpret 'nop; do f over .context~stackframes; say f~type f~line f~traceline; end'
  interpret 'interpret "say .context~stackframes~items"; say .context~stackframes~firstItem~traceline'
