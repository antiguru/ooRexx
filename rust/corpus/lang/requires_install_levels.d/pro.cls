call show 'pro'
::requires 'pro2.cls'
::routine show public
  use arg label
  do f over .context~stackframes
    if f~type == 'ROUTINE' & f~name == 'SHOW' then iterate
    say '  ' label f~type filespec('name', f~name) f~line
  end
