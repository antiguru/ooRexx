import argparse
import csv
import hashlib
import json
import math
import os
from pathlib import Path
import re
import signal
import statistics
import subprocess
import time

BASE = Path(__file__).parent
OUT = BASE / 'inventory'
ROOT = Path('/home/moritz/dev/repos/ooRexx-rust-rewrite')
BIN = {'oracle': ROOT / 'build/bin/rexx', 'rust': BASE / 'rust-rexx'}
ENV = dict(os.environ, LD_LIBRARY_PATH=str(ROOT / 'build/lib'))
ENV.pop('REXX_ENGINE', None)
CASES = json.loads((BASE / 'inventory-cases.json').read_text())
UNROLL = 16
CPU = 13
SIBLING = 29
ELAPSED = re.compile(rb'^__BENCH_SECONDS__\s+([0-9.]+)\s*$', re.M)
OUT.mkdir(exist_ok=True)

def save(path, value):
    path.write_text(json.dumps(value, indent=2))

def source(c, count, unroll):
    marker = ".stderr~lineout('__BENCH_SECONDS__' time('E'))" if c.get('sink') == 'devnull' else "say '__BENCH_SECONDS__' time('E')"
    bodies = ''.join(c['body'].replace('{BENCH_SITE}', str(site))+'\n' for site in range(unroll))
    return ('numeric digits 9\n' + c['setup'] + "\ncall time 'R'\n" +
            f'do bench_outer = 1 to {count}\n' + bodies +
            'end\n'+marker+'\n' + c['tail'] + '\n')

def invoke(c, path, side, tag, event=None, discard=False):
    command = ['taskset', '-c', str(CPU), str(BIN[side]), str(path)]
    counter_file = path.parent / f'{tag}-{side}-{event}.perf.csv'
    if event:
        command = ['perf', 'stat', '-x', ',', '-o', str(counter_file), '-e', event, '--'] + command
    started = time.perf_counter()
    p = subprocess.Popen(command, cwd=path.parent, env=ENV, stdin=subprocess.DEVNULL,
                         stdout=subprocess.DEVNULL if discard else subprocess.PIPE,
                         stderr=subprocess.PIPE, start_new_session=True)
    try:
        stdout, stderr = p.communicate(timeout=20)
    except subprocess.TimeoutExpired:
        os.killpg(p.pid, signal.SIGKILL)
        stdout, stderr = p.communicate()
        return dict(side=side, tag=tag, event=event, exit_code=p.returncode,
                    wall_seconds=time.perf_counter()-started, error='timeout',
                    stderr=(stderr or b'').decode(errors='replace'))
    stdout = stdout or b''
    match = ELAPSED.search(stdout) or ELAPSED.search(stderr)
    stderr = ELAPSED.sub(b'', stderr).strip()
    row = dict(side=side, tag=tag, event=event, exit_code=p.returncode,
               wall_seconds=time.perf_counter()-started,
               internal_seconds=float(match[1]) if match else None,
               stdout_bytes=len(stdout),
               stdout_sha256=hashlib.sha256(ELAPSED.sub(b'', stdout)).hexdigest(),
               stderr=stderr.decode(errors='replace'))
    if p.returncode or stderr:
        row['error'] = 'execution failed or wrote stderr'
        row['stdout'] = stdout[:10000].decode(errors='replace')
    if not match:
        row['error'] = 'missing elapsed marker'
    if event and not row.get('error'):
        raw = counter_file.read_text()
        row['counter_raw'] = raw
        try:
            records = [line.split(',') for line in raw.splitlines() if line and not line.startswith('#')]
            record = next(r for r in records if r[2] == event)
            row['counter'] = int(record[0])
            row['counter_running_percent'] = float(record[4])
            if row['counter_running_percent'] < 99.9:
                row['error'] = 'multiplexed counter'
        except (ValueError, StopIteration, IndexError):
            row['error'] = 'unavailable counter'
    if not discard:
        (path.parent / f'{tag}-{side}.stdout').write_bytes(stdout)
    return row

def smoke():
    results = []
    for index, c in enumerate(CASES):
        directory = OUT / c['id']
        directory.mkdir(exist_ok=True)
        path = directory / 'smoke.rex'
        path.write_text(source(c, 3, 2))
        rows = [invoke(c, path, side, 'smoke') for side in BIN]
        status = 'pass'
        reason = ''
        if any(r.get('error') for r in rows):
            status, reason = 'failed', 'execution failure'
        elif rows[0]['stdout_sha256'] != rows[1]['stdout_sha256']:
            status, reason = 'mismatch', 'stdout differs after removing elapsed measurement'
        result = dict(id=c['id'], status=status, reason=reason, rows=rows)
        results.append(result)
        save(OUT / 'smoke.json', results)
        print(f"SMOKE {index+1}/{len(CASES)} {c['id']} {status}", flush=True)
    return results

def paired(c, count, tag, event=None, reverse=False):
    directory = OUT / c['id']
    specs = [(side, width) for width in [1, UNROLL] for side in BIN]
    if reverse:
        specs.reverse()
    rows = []
    for side, width in specs:
        path = directory / f'width-{width}.rex'
        path.write_text(source(c, count, width))
        r = invoke(c, path, side, f'{tag}-width-{width}', event,
                   discard=c.get('sink') == 'devnull')
        r.update(width=width, count=count)
        rows.append(r)
    if c.get('sink') != 'devnull':
        for width in [1, UNROLL]:
            pair = [r for r in rows if r['width'] == width]
            if not any(r.get('error') for r in pair) and pair[0]['stdout_sha256'] != pair[1]['stdout_sha256']:
                for r in pair:
                    r['error'] = 'stdout mismatch'
    return rows

def measure_case(c):
    sibling_before = sibling_ticks()
    count = c.get('calibration_count', 100 if c['group'] == 'io' else 2000)
    calibration = paired(c, count, 'calibration')
    if any(r.get('error') for r in calibration):
        return dict(id=c['id'], status='calibration-failed', rows=calibration)
    high = [r for r in calibration if r['width'] == UNROLL]
    costs = [max(.0001, r['internal_seconds']) for r in high]
    factor = min(.8/max(costs), max(.12/max(costs), .003/min(costs)))
    count = max(10, min(c.get('max_count', 1000000), int(count*factor)))
    rows = list(calibration)
    rows += paired(c, count, 'warmup', reverse=True)
    for round_ in range(3):
        rows += paired(c, count, f'wall-{round_}', reverse=bool(round_ % 2))
    for event in ['instructions:u', 'cycles:u']:
        rows += paired(c, count, event, event=event)
    save(OUT / c['id'] / 'runs.json', rows)
    errors = [r for r in rows[4:] if r.get('error')]
    result = {k: c[k] for k in ['id', 'group', 'covers', 'body', 'setup', 'tail', 'unit', 'units_per_body', 'notes']}
    sibling_after = sibling_ticks()
    total = sibling_after[0]-sibling_before[0]
    busy = 1-(sibling_after[1]-sibling_before[1])/max(1,total)
    result.update(count=count, unroll=UNROLL, status='measured', errors=errors,
                  cpu=CPU, sibling_cpu=SIBLING, sibling_busy_fraction=busy)
    if errors:
        result['status'] = 'measurement-failed'
        return result
    units = count*(UNROLL-1)*c['units_per_body']
    for side in BIN:
        samples = []
        for round_ in range(3):
            low = next(r for r in rows if r['side'] == side and r['tag'].startswith(f'wall-{round_}-') and r['width'] == 1)
            high = next(r for r in rows if r['side'] == side and r['tag'].startswith(f'wall-{round_}-') and r['width'] == UNROLL)
            samples.append((high['wall_seconds']-low['wall_seconds'])*1e9/units)
        internal = []
        for round_ in range(3):
            low = next(r for r in rows if r['side'] == side and r['tag'].startswith(f'wall-{round_}-') and r['width'] == 1)
            high = next(r for r in rows if r['side'] == side and r['tag'].startswith(f'wall-{round_}-') and r['width'] == UNROLL)
            internal.append((high['internal_seconds']-low['internal_seconds'])*1e9/units)
        primary = samples if c['id'] == 'method-reply-last' else internal
        d = dict(ns_median=statistics.median(samples), ns_min=min(samples), ns_max=max(samples), samples_ns=samples,
                 internal_ns_median=statistics.median(internal), internal_samples_ns=internal,
                 primary_ns=statistics.median(primary), primary_min_ns=min(primary), primary_max_ns=max(primary),
                 primary_timing='process-wall' if c['id'] == 'method-reply-last' else 'in-process-elapsed')
        for event in ['instructions:u', 'cycles:u']:
            low = next(r for r in rows if r['side'] == side and r['event'] == event and r['width'] == 1)
            high = next(r for r in rows if r['side'] == side and r['event'] == event and r['width'] == UNROLL)
            d[event] = (high['counter']-low['counter'])/units
        result[side] = d
    result['rust_over_oracle'] = result['rust']['primary_ns']/result['oracle']['primary_ns'] if result['oracle']['primary_ns'] > 0 else None
    result['quality'] = 'stable' if all(result[s]['primary_min_ns'] > 0 and (result[s]['primary_max_ns']-result[s]['primary_min_ns'])/result[s]['primary_ns'] <= .15 for s in BIN) else 'noisy'
    if busy > .10:
        result['quality'] += ';sibling-busy'
    return result

def sibling_ticks():
    for line in Path('/proc/stat').read_text().splitlines():
        fields=line.split()
        if fields and fields[0] == f'cpu{SIBLING}':
            values=list(map(int,fields[1:]))
            return sum(values[:8]), values[3]+values[4]
    raise RuntimeError('Missing sibling CPU statistics')

def table(results):
    fields = ['id','group','status','quality','oracle_ns','rust_ns','rust_over_oracle','oracle_instructions','rust_instructions','oracle_cycles','rust_cycles','oracle_wall_ns','rust_wall_ns','sibling_busy_fraction','unit','covers','body','notes']
    with (OUT/'costs.csv').open('w',newline='') as f:
        writer=csv.DictWriter(f,fieldnames=fields)
        writer.writeheader()
        for r in results:
            row={k:r.get(k,'') for k in ['id','group','status','quality','rust_over_oracle','unit','body','notes']}
            row['covers']=';'.join(r.get('covers',[]))
            row['sibling_busy_fraction']=r.get('sibling_busy_fraction','')
            for s in BIN:
                d=r.get(s,{})
                for suffix,key in [('ns','primary_ns'),('instructions','instructions:u'),('cycles','cycles:u'),('wall_ns','ns_median')]:
                    row[f'{s}_{suffix}']=d.get(key,'')
            writer.writerow(row)

def measure():
    smoke_results={r['id']:r for r in json.loads((OUT/'smoke.json').read_text())}
    results=json.loads((OUT/'results.json').read_text()) if (OUT/'results.json').exists() else []
    done={r['id'] for r in results}
    for index,c in enumerate(CASES):
        if c['id'] in done:
            continue
        if smoke_results[c['id']]['status'] != 'pass':
            r=dict(c,status='smoke-failed')
        else:
            r=measure_case(c)
        results.append(r)
        save(OUT/'results.json',results)
        table(results)
        print(f"RESULT {index+1}/{len(CASES)} {c['id']} {r['status']} " + json.dumps({s:r.get(s,{}).get('ns_median') for s in BIN}) + ' ' + r.get('quality',''),flush=True)
    print('COMPLETE '+json.dumps({status:sum(r['status']==status for r in results) for status in sorted({r['status'] for r in results})}),flush=True)

if __name__ == '__main__':
    parser=argparse.ArgumentParser()
    parser.add_argument('mode',choices=['smoke','measure'])
    mode=parser.parse_args().mode
    if mode=='smoke':
        smoke()
    else:
        measure()
