do 1
  interpret 'do 1; do 1; do f over .context~stackframes; say f~traceline; end; end; end'
end
interpret 'do 1; interpret "do 1; do f over .context~stackframes; say f~traceline; end; end"; end'
call t2
exit
t2:
  do 1
    interpret 'do 1; nop; end; do f over .context~stackframes; say "t2" f~traceline; end'
  end
  return
