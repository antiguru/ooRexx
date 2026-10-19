arr=.array~new
.traceObject~collector=arr
.traceObject~option='P'
src=.array~of('  return .myTestTO      -- line # 1', '/* ---> do not edit (insert or delete) the following lines, baseline calcs above depend on this   */', '::routine getSourceLines public  -- expects an array of indexes pointing to the source lines caller is interested in', '  use arg lineArray        -- line # 4', '  resArr=.array~new', '  do line over lineArray', ' -- say "#" .line": line="line "["sourceline(line)"]"', '     resArr~append(line~right(6) "*-*" sourceLine(line)~strip(''left''))', '  end', '  return resArr            -- line # 10', '::class myTestTO', '::attribute baseline class -- stores line # of class init''s assignment: a=19', '::method  init class', '  expose a b baseline', '  baseline=.line+2      -- method: # 15', '  trace all', '  a=19                  -- method: # 17', '  b=32                  -- method: # 18', '::method    a class', '  expose a', '  trace all             -- since retro-trace will cause <I<', '  reply  a              -- method: # 22', '  a=a                   -- method: # 23', '::method    b class', '  expose b', '  trace all             -- since retro-trace will cause <I<', '  reply  b              -- method: # 27', '  b=b                   -- method: # 28', '::attribute abc', '::method    test        -- method: abc=pp(19*32) is in baseline', '  expose    abc baseline srcLine1 srcLine2', '  trace all             -- since retro-trace will cause <I<', '  reply                 -- method: # 33', '  abc=pp(.myTestTO~a*.myTestTO~b)   -- method: # 34', '::routine pp', '  trace all             -- since retro-trace will cause <I<', '  return "["arg(1)"]"   -- routine: # 37', '/* <--- from now on editing is o.k. again  */')
code=.routine~new("testRoutine",src)
clz=code~call
t=clz~new~~test
call syssleep 0.1
.traceObject~collector=.nil
.traceObject~option='N'
out=.array~new
do o over arr; out~append(o~traceLine); end
do l over out~sort; say l; end
