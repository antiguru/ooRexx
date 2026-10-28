say '['||'12'~translate('','')||']'
say '['||'12'~translate('ab','')||']'
say '['||'12'~translate(,'')||']'
say '['||'12'~translate('', , '*')||']'
e = 'abc'~substr(5)
say '['||'12'~translate('', e)||']'
e = left('', 0)
say '['||'12'~translate('', e)||']'
e = ''
say '['||'12'~translate('', e)||']'
e = 'x'~left(0)
say '['||'12'~translate('', e)||']'
e = .string~new('')
say '['||'12'~translate('', e)||']'
e = ''~copies(3)
say '['||'12'~translate('', e)||']'
e = 'ab'~delstr(1)
say '['||'12'~translate('', e)||']'
say '['translate('12','','')']'
say '['translate('12','',e)']'
say '['||'12'~translate('', '', '*', 2)||']'
mb = .mutablebuffer~new('12'); say '['mb~translate('','')~string']'
