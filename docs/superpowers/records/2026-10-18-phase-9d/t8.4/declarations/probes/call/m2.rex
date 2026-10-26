signal on syntax
call badlib.rex
exit
syntax: say condition('o')~code sigl; say condition('o')~traceback~makestring; do f over condition('o')~stackframes; say f~type f~name f~line; end
