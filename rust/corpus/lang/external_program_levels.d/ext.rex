use arg n
call lvl n
return 1
lvl: procedure; use arg k; if k > 3 then x = k + 'bad'; return
