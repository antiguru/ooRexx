.context~package~local~put(.list, 'BAR')
::class T public
::method find external "LIBRARY orxmethod TestFindContextClass"
::method local
  return .context~package~findClass(arg(1))
::routine findr public external "LIBRARY orxfunction TestFindContextClass"
