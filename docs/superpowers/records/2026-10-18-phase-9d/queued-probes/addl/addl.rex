signal on syntax
interpret 'raise syntax 40.1 additional("abcdefghijklmnopqrstuvwxyz0123456789")'
syntax:
say condition('O')~additional[1]
signal on syntax name s2
raise syntax 40.1 additional('abcdefghijklmnopqrstuvwxyz0123456789')
s2:
say condition('O')~additional[1]
