/* A supplied input table is one even when it is empty, and it matches no byte. */
say '['||'12'~translate('','')||']'
say '['||'12'~translate('ab','')||']'
say '['||'12'~translate(,'')||']'
say '['||'12'~translate('', , '*')||']'
say '['||'12'~translate('', '', '*', 2)||']'
say '['||'12'~translate('')||']'
say '['||'12'~translate('ab')||']'
say '['translate('12','','')']'
say '['translate('12','')']'
mb = .mutablebuffer~new('12')
say '['mb~translate('','')~string']'
