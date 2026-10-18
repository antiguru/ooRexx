# Classifies each differing scenario of a NOBUFFER fuzz run by the operation
# that explains it: A relative SEEK, B char SEEK from the end, C both-pointer
# char SEEK (line positioning from character 0 follows), else unexplained.
import re, sys, glob
tot = {'A': 0, 'B': 0, 'C': 0, 'unexplained': 0}
scen = 0
for diff in sorted(glob.glob('*.diff')):
    name = diff[:-5]
    ops = {m.group(1): m.group(2) for m in re.finditer(r'call scenario "(s\d+)", (.*)', open(name + '.rex').read())}
    text = open(diff).read()
    scen += int(re.search(r'scenarios (\d+)', text).group(1))
    for s in re.findall(r'oracle: (s\d+) :', text):
        o = ops[s]
        if re.search(r'"S [+-]', o): k = 'A'
        elif '"S <1 write"' in o: k = 'B'
        elif '"S =7"' in o: k = 'C'
        else:
            k = 'unexplained'; print('UNEXPLAINED', name, s, o)
        tot[k] += 1
print('scenarios', scen, tot)
