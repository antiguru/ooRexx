"""Mutants for Phase 9c Task 4: one per converted raise site.

Usage: mutants.py list | apply BATCH | restore
Each mutant is (id, file, line, old, new, pair). `apply` writes every mutant of
the batch by exact replacement on its line, after saving copies.
"""
import json, os, re, shutil, sys

ROOT = '/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-parse/'
SAVE = os.path.join(os.path.dirname(os.path.abspath(__file__)), 'mut-saved')
FILES = ['src/block.rs', 'src/directive.rs', 'src/lib.rs']


def mutants():
    out = []
    lines = {f: open(ROOT + f).read().split('\n') for f in FILES}

    def find(f, needle, nth=0):
        hits = [i for i, l in enumerate(lines[f]) if needle in l]
        assert len(hits) > nth, (f, needle, hits)
        return hits[nth] + 1

    def add(f, needle, old, new, pair, nth=0):
        ln = find(f, needle, nth)
        assert lines[f][ln - 1].count(old) == 1, (f, ln, old, lines[f][ln - 1])
        out.append(dict(file=f, line=ln, old=old, new=new, pair=pair))

    b = 'src/block.rs'
    add(b, 'Control::Do => (E14_1', 'E14_1', 'E7_1', '14.1')
    add(b, '(E14_2, frame.index())', 'E14_2', 'E7_1', '14.2')
    add(b, '.parent', '.parent', '.index', '14.3', nth=1)
    add(b, 'Control::IfThen | Control::WhenThen => (E14_3, frame.index())', 'frame.index()', 'frame.parent.unwrap()', '14.3')
    add(b, '(E14_4, frame.index())', 'E14_4', 'E7_1', '14.4')
    add(b, '(E14_5, frame.index())', 'E14_5', 'E7_1', '14.5')
    add(b, '(E14_901, frame.index())', 'E14_901', 'E7_1', '14.901')
    add(b, 'Control::IfThen | Control::Else => E47_3', 'E47_3', 'E47_2', '47.3')
    add(b, '| Control::Otherwise => E47_4', 'E47_4', 'E47_2', '47.4')
    add(b, 'Control::Do | Control::Loop => E47_2', 'E47_2', 'E47_3', '47.2')
    add(b, '{ E10_4 } else { E10_2 }', '{ E10_4 }', '{ E10_2 }', '10.4')
    add(b, '{ E10_4 } else { E10_2 }', 'else { E10_2 }', 'else { E10_4 }', '10.2')
    add(b, '{ E10_7 } else { E10_3 }', '{ E10_7 }', '{ E10_3 }', '10.7')
    add(b, '{ E10_7 } else { E10_3 }', 'else { E10_3 }', 'else { E10_7 }', '10.3')
    add(b, '[name, label, line]', '[name, label, line]', '[label, name, line]', '10.2')
    add(b, '[name, line]', '[name, line]', '[line, name]', '10.3')
    add(b, 'self.error_line(E7_1', 'E7_1', 'E7_2', '7.1')
    add(b, 'ParseError::raise(E10_1', 'E10_1', 'E8_2', '10.1')
    add(b, 'block.error_line(E7_2', 'E7_2', 'E7_1', '7.2')
    add(b, 'ParseError::raise(E8_2', 'E8_2', 'E10_1', '8.2')
    add(b, '[b"ELSE".to_vec()]', 'b"ELSE"', 'b"MUT"', '47.3')
    add(b, 'ParseError::raise(E9_2', 'E9_2', 'E10_1', '9.2')
    add(b, 'self.source.line_of(opener)', 'line_of(opener)', 'line_of(byte)', '18.1')
    add(b, '[name.to_vec()]', 'name.to_vec()', 'b"MUT".to_vec()', '47.2')
    add(b, 'self.source.line_of(byte).to_string()', 'self.source.line_of(byte).to_string()', '(self.source.line_of(byte) + 1).to_string()', '14.1')

    add('src/lib.rs', 'ParseError::raise(E99_914', 'E99_914', 'rexx_inventory::errors::E99_908', '99.914')

    d = 'src/directive.rs'
    add(d, 'let position = opener.span.start', 'line_start + 1', 'line_start + 2', '36.901')
    add(d, 'self.error(E24_1', 'bad.into_iter().collect()', 'b"MUT".to_vec()', '24.1')
    add(d, 'self.error(E99_925', 'name.to_vec()', 'b"MUT".to_vec()', '99.925')
    add(d, 'let bad = || self.error(E99_917', 'spec.to_vec()', 'b"MUT".to_vec()', '99.917')
    add(d, 'self.error(E25_927', 'self.value_of(token).into_vec()', 'b"MUT".to_vec()', '25.927')
    add(d, 'return Err(self.error(E20_929, [name()]));', 'name()', 'b"MUT".to_vec()', '20.929', nth=0)
    add(d, 'return Err(self.error(E20_929, [name()]));', 'name()', 'b"MUT".to_vec()', '20.929', nth=1)
    add(d, 'self.error(E19_925', 'target.to_vec()', 'b"MUT".to_vec()', '19.925')
    for t in ['CLASS', 'ROUTINE', 'METHOD', 'ATTRIBUTE', 'CONSTANT']:
        add(d, f'self.annotated_name(b"{t}")', f'b"{t}"', 'b"MUT"', '19.925')
    add(d, 'self.error(E19_923, [insert])', '[insert]', '[b"MUT".to_vec()]', '19.923')
    add(d, 'self.error(E26_5', 'value.to_vec()', 'b"MUT".to_vec()', '26.5')
    add(d, 'self.error(E26_6', 'value.to_vec()', 'b"MUT".to_vec()', '26.6')
    add(d, 'self.required_end(E21_913)', 'E21_913', 'E21_914', '21.913')
    add(d, 'self.required_end(E21_914)', 'E21_914', 'E21_913', '21.914')
    # found(E.., Some(token)) / found(E.., token)
    for i, l in enumerate(lines[d]):
        m = re.search(r'self\.found\((E(\d+)_(\d+)), (Some\(token\)|token)\)', l)
        if m:
            out.append(dict(file=d, line=i + 1, old=m.group(0),
                            new=f'self.error({m.group(1)}, [b"MUT".to_vec()])',
                            pair=f'{m.group(2)}.{m.group(3)}'))
    # Msg<0> sites
    for i, l in enumerate(lines[d]):
        if l.strip().startswith('//'):
            continue
        for m in re.finditer(r'(self\.error\(|require_name\(|require_symbol\(|require_literal\(|class_reference\(|check_directive\(cursor, )(E(\d+)_(\d+))(, \[\]\)|\))', l):
            if m.group(1) == 'self.error(' and not m.group(5).startswith(', []'):
                continue
            if 'fn ' in l:
                continue
            repl = 'E20_916' if m.group(2) == 'E99_916' else 'E99_916'
            out.append(dict(file=d, line=i + 1, old=m.group(0),
                            new=m.group(1) + repl + m.group(5),
                            pair=f'{m.group(3)}.{m.group(4)}'))
    # batches: a pair at most once per batch
    seen = {}
    for k, mu in enumerate(out):
        mu['id'] = f'M{k:02d}'
        n = seen.get(mu['pair'], 0)
        mu['batch'] = n
        seen[mu['pair']] = n + 1
        if mu['new'].startswith('(self.source.line_of(byte) + 1)'):
            mu['batch'] = 9
    return out


def main():
    cmd = sys.argv[1]
    if cmd == 'list':
        for mu in mutants():
            print(json.dumps(mu))
    elif cmd == 'apply':
        batch = int(sys.argv[2])
        ms = [m for m in mutants() if m['batch'] == batch]
        os.makedirs(SAVE, exist_ok=True)
        for f in FILES:
            shutil.copy(ROOT + f, os.path.join(SAVE, os.path.basename(f)))
        text = {f: open(ROOT + f).read().split('\n') for f in FILES}
        for m in ms:
            l = text[m['file']][m['line'] - 1]
            assert l.count(m['old']) == 1, m
            text[m['file']][m['line'] - 1] = l.replace(m['old'], m['new'])
        for f in FILES:
            open(ROOT + f, 'w').write('\n'.join(text[f]))
        for m in ms:
            print(m['id'], m['pair'], m['file'], m['line'])
    elif cmd == 'restore':
        for f in FILES:
            shutil.copy(os.path.join(SAVE, os.path.basename(f)), ROOT + f)


main()
