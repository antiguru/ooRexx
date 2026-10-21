do o over .array~of(.environment, .local, .context~package~routines, .methods)
  c = .Message~new(o, .array~of('COPY', .Object))~send
  say c~class~id (c == o)
end
::routine r
::method m
