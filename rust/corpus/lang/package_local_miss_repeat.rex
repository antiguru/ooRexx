/* The same missing environment symbol read twice in a row, once after a write
   to the package local under another name and once after the write that
   fills it, and in a loop that writes nothing. */
l = .context~package~local
l~put('other', 'PLMROTHER')
say .plmrname .plmrname
do 3
  say .plmrnone
end
l~put('filled', 'PLMRNAME')
say .plmrname .plmrname
l~remove('PLMRNAME')
say .plmrname .plmrname
