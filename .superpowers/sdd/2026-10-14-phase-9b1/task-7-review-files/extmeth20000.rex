do i = 1 to 20000
  m = .Method~loadExternalMethod('m', 'LIBRARY rxregexp RegExp_Init')
end
say 'done' m~class~id
