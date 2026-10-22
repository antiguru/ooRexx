signal on syntax name sx
interpret "signal on novalue name gone_x; y = qq"
exit
sx: f = .context~stackframes[1]; say 'sf' f~type f~line '['f~name']'
exit
