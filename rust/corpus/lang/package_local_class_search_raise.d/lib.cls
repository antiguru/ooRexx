l = .context~package~local
l~put(.array, 'PLFIND')
l~setMethod('UNKNOWN', 'say "unknown called" arg(1); raise syntax 93.900 array("from unknown" arg(1))')
::class T public
::method findc external "LIBRARY orxmethod TestFindClass"
