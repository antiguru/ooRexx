call lineout 'a' || '00'x || 'b', 'x'
call lineout 'a' || '00'x || 'b'
say 'nul stream' (stream('a', 'c', 'query exists') <> '') (stream('a' || '00'x || 'zz', 'c', 'query exists') <> '')
say 'nul .Stream' (.Stream~new('q' || '00'x || 'z')~qualify == qualify('q'))
say 'no user lineout' lineout('~nosuchuser.rexxutil/x', 'y')
say 'no user exists [' || stream('~nosuchuser.rexxutil/x', 'c', 'query exists') || ']'
say 'no user state' stream('~nosuchuser.rexxutil/x', 's') stream('~nosuchuser.rexxutil/x', 'd')
say 'no user .Stream [' || .Stream~new('~nosuchuser.rexxutil/x')~qualify || ']'
say 'empty .Stream [' || .Stream~new('')~qualify || ']'
address system 'echo hi' with output stream ('o' || '00'x || 'p')
say 'nul redirect' (stream('o', 'c', 'query exists') <> '')
h = value('HOME', directory(), 'ENVIRONMENT')
address system 'echo tilde' with output stream '~/t.txt'
say 'tilde redirect' (stream(directory()'/t.txt', 'c', 'query exists') <> '') linein(directory()'/t.txt')
call value 'HOME', h, 'ENVIRONMENT'
