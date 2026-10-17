out = .array~new
address '' with output append using (out) error append using (out)
address '' 'rexx -o "trace all" showArgs.rex -o "my output file"'
do i = 1 to out~items; say i out[i]; end
