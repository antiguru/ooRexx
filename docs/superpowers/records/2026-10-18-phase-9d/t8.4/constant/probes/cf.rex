say .c~j
say .c~s~makestring('l', ',')
::class c
::constant j (.context~executable~class (.context~executable == .c~method('J')) .context~package~name~right(8))
::constant s (.c~frames)
::method frames class
  a = .array~new
  do f over .context~stackframes; a~append(f~type f~name f~line); end
  return a
