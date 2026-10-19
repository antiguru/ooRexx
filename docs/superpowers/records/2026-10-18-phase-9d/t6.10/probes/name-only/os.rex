.traceObject~option='P'
arr=.array~new
.traceObject~collector=arr
.routine~new("myTestRoutine",.array~of('trace all', 't1=.test~new', 't1~setMethod1("m1float",.methods~fm,''float'')   -- scope=FLOAT (hence .nil)', 't1~m1float  -- OBJECTID, SCOPE=.nil', 't1~setMethod1("m1object",.methods~fm,''object'') -- scope=OBJECT''s scope', 't1~m1object -- OBJECTID, SCOPE=.TEST', '', '::method fm       -- a floating method (access via .methods possible)', '  expose info', '  trace all             -- since retro-trace will cause <I<', '  a1=.context~name":" self~identityHash "(floating method)"', '  a2="self:" self "hello, world, scope:" .context~executable~scope "| info:" info', '', '::class test', '::method init', '  expose info', '  trace all             -- since retro-trace will cause <I<', '  b=.context~name":" self~identityHash', '  info="initialized by .TEST''s init"', '', '::attribute info get', '  expose info', '  trace all             -- since retro-trace will cause <I<', '  return info', '', '::method setMethod1', '  use arg name, meth, scope', '  -- say .context~name":" self~identityHash', '  self~setMethod(name, meth, scope)'))~~call
.traceObject~collector=.nil
.traceObject~option='N'
say arr~items
do i over arr
  sf=i~stackFrame
  if sf~type="METHOD" then sc=sf~executable~scope; else sc=.nil
  say i~traceLine '|' sf~type '|' sf~target~isNil '|' sc~isNil
end
