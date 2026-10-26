p = .package~new('pk', .array~of('say .c~s~makestring("l", ",")', '::class c', '::constant s (.c~frames)', '::method frames class', '  a = .array~new', '  do f over .context~stackframes; a~append(f~type filespec("name", f~name) f~line); end', '  return a'))
p~call
