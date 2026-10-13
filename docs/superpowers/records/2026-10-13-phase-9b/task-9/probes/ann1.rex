a = .context~package~findRoutine('ff')
say a~annotations~items a~annotation('k1') a~annotation('k2')
l = .Routine~loadExternalRoutine('x','LIBRARY REXX Filespec')
say (l == a) l~annotations~items
u = .Routine~loadExternalRoutine('x','LIBRARY rexxutil SysSleep')
say u~annotations~items u~annotation('k3')
::routine ff external 'LIBRARY REXX Filespec'
::routine gg external 'LIBRARY REXX Filespec'
::routine uu external 'LIBRARY rexxutil SysSleep'
::annotate routine ff k1 'one'
::annotate routine gg k2 'two'
::annotate routine uu k3 'three'
