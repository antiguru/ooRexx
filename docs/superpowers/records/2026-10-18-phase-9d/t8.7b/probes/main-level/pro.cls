do f over .context~stackframes; say f~type filespec('name', f~name) f~line; end
say .context~stackframes~items
::requires 'pro2.cls'
