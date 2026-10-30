import copy
import csv
import hashlib
import html
import json
import math
from pathlib import Path
import statistics
import zipfile

BASE=Path(__file__).parent
OUT=BASE/'inventory'
raw=json.loads((OUT/'results.json').read_text())
catalog=[]
for source in raw:
    if source['id']=='label':
        continue
    r=copy.deepcopy(source)
    r['comparable']=r['group']!='queue' and r['group']!='io' and r['id']!='method-reply-last'
    r['comparison_class']='fixed-kernel'
    if r['group']=='queue':
        r['comparison_class']='different-backends'
        r['notes']+=' Backend mismatch: Rust uses its in-process interpreter queue. The oracle uses the external queue API. RXAPI integration is not implemented in the Rust build, as confirmed by the user. Excluded from like-for-like summaries.'
    if r['group']=='io':
        r['comparison_class']='io-or-process-composite'
        r['notes']+=' I/O buffering or OS process work is included. This is not an isolated language-operation cost.'
    if r['id']=='method-reply-last':
        r['status']='unresolved-asynchronous-timing'
        r['comparison_class']='asynchronous-not-isolated'
        r['notes']+=' Published costs and ratio withheld: process-wall subtraction is inconsistent and negative for Rust. Sender-loop elapsed times and raw counters remain in runs.json. Native continuation scheduling prevents a reliable isolated cost in this kernel.'
        r['rust_over_oracle']=None
        for side in ['oracle','rust']:
            r[side]['primary_ns']=None
            r[side]['primary_min_ns']=None
            r[side]['primary_max_ns']=None
    if r['status']=='measured':
        low=r['rust']['primary_min_ns']/r['oracle']['primary_max_ns']
        high=r['rust']['primary_max_ns']/r['oracle']['primary_min_ns']
        r['observed_ratio_min']=low
        r['observed_ratio_max']=high
        r['sample_direction']='rust-faster' if high<1 else 'oracle-faster' if low>1 else 'sample-ranges-overlap'
    catalog.append(r)

exit_data=json.loads((OUT/'process-exit'/'summary.json').read_text())
exit_row=dict(id='process-exit',group='process-end-to-end',covers=['Exit'],body='exit 0',setup='',tail='',
              unit='whole process invocation',units_per_body=1,count=1,unroll=1,status='end-to-end-only',
              quality='includes-startup',comparison_class='program-startup-and-termination',comparable=False,
              notes='Whole CLI process: loader, runtime initialization, parsing, execution of EXIT 0 and termination. No isolated EXIT instruction cost is inferred. Seven measured alternating pairs after two warmups per side.')
for side,d in exit_data.items():
    exit_row[side]=dict(primary_ns=d['median_seconds']*1e9,primary_min_ns=d['min_seconds']*1e9,
                        primary_max_ns=d['max_seconds']*1e9,primary_timing='whole-process-wall')
exit_row['rust_over_oracle']=exit_row['rust']['primary_ns']/exit_row['oracle']['primary_ns']
catalog.append(exit_row)
(OUT/'catalog.json').write_text(json.dumps(catalog,indent=2))

coverage=json.loads((BASE/'instruction-coverage.json').read_text())
for c in coverage:
    if c['instruction']=='Label':
        c['case_ids']=[x for x in c['case_ids'] if x!='label']
        c['classification']='marker-in-context'
        c['notes']='Covered in CALL/SIGNAL context. A standalone repeated label probe was rejected by both interpreters with 47.2 because labels are forbidden inside DO/LOOP blocks. No independent label price is inferred.'
    if c['instruction']=='Exit':
        c['case_ids']=['process-exit']
    if c['instruction']=='Reply':
        c['notes']+=' The measured sender/process boundaries did not yield a reliable independent cost. Catalog row is explicitly unresolved.'
    if c['instruction'] in ['Push','Pull','Queue']:
        c['notes']+=' Queue roundtrips exercise different backend implementations because Rust lacks RXAPI integration.'
(OUT/'instruction-coverage.json').write_text(json.dumps(coverage,indent=2))
with (OUT/'instruction-coverage.csv').open('w',newline='') as f:
    writer=csv.writer(f)
    writer.writerow(['instruction','classification','case_ids','notes'])
    for c in coverage:
        writer.writerow([c['instruction'],c['classification'],';'.join(c['case_ids']),c['notes']])

flat=[]
for r in catalog:
    row={k:r.get(k,'') for k in ['id','group','status','quality','comparison_class','comparable','unit','units_per_body','count','unroll','rust_over_oracle','observed_ratio_min','observed_ratio_max','sample_direction','body','setup','tail','notes']}
    row['covers']=';'.join(r['covers'])
    row['sibling_busy_pct']=100*r.get('sibling_busy_fraction',0) if 'sibling_busy_fraction' in r else ''
    for side in ['oracle','rust']:
        d=r[side]
        for suffix,key in [('ns','primary_ns'),('ns_min','primary_min_ns'),('ns_max','primary_max_ns'),('instructions_u','instructions:u'),('cycles_u','cycles:u'),('wall_ns','ns_median')]:
            row[side+'_'+suffix]=None if r['status']=='unresolved-asynchronous-timing' else d.get(key)
        row[side+'_timing']=d.get('primary_timing','')
    flat.append(row)
fields=list(flat[0])
with (OUT/'costs.csv').open('w',newline='') as f:
    writer=csv.DictWriter(f,fieldnames=fields)
    writer.writeheader()
    writer.writerows(flat)

summary=dict(catalog_rows=len(catalog),normal_positive_kernels=sum(r['status']=='measured' for r in catalog),
             unresolved_rows=[r['id'] for r in catalog if r['status'].startswith('unresolved')],
             instruction_families=len(coverage),
             different_backend_rows=[r['id'] for r in catalog if r['comparison_class']=='different-backends'],
             noisy_rows=sum('noisy' in r['quality'] for r in catalog),
             sibling_busy_rows=sum('sibling-busy' in r['quality'] for r in catalog),
             excluded_probe=dict(id='label',reason='Both implementations reject labels within DO/LOOP with 47.2. Covered contextually instead.'),
             comparable_fixed_kernels=sum(r['status']=='measured' and r['comparable'] for r in catalog))
(OUT/'summary.json').write_text(json.dumps(summary,indent=2))

method='''Rexx instruction cost inventory

The catalog maps all 40 parser instruction families to representative workloads.
It contains 132 positive repeated-kernel measurements, an unresolved asynchronous REPLY row, and one end-to-end EXIT row.
Context requirements are explicit: calls, returns, guards, forwards, loops and exception delivery are composite operations.
Marker instructions are covered through enclosing control flow, and no isolated marker cost is inferred.

The same frozen Rust executable and version-matched optimized C++ oracle were used throughout.
Rust source revision is unverified because the existing release executable was copied while another agent owned the working tree.
The oracle flags contain -O3 and LTO, and environment.json records executable/library/image hashes.
The first CPU-2 sweep was superseded after sibling interference and process-startup cancellation were detected, and its raw evidence remains in the parent directory.

Each normal kernel has fixed operand values and a fixed outer iteration count shared by both interpreters.
The body is unrolled at 1 and 16 sites, and marginal cost is (high - low) / (COUNT * 15 * units_per_body).
Three alternating-order timing pairs follow calibration and a warmup, with elapsed time read inside Rexx using TIME(R)/TIME(E).
Instructions and cycles are separate, nonmultiplexed whole-process user-mode counter pairs, so their high-minus-low differences can retain small translation and initialization residuals.

Measurements use CPU 13 on an AMD RYZEN AI MAX+ 395, with a 2 GiB cgroup memory cap around the sequential harness.
Sibling CPU 29 utilization is recorded per case and values above 10 percent are flagged.
The noisy flag means the observed sample range exceeds 15 percent of its median, and sample ranges are not confidence intervals.
These are context-specific costs on a shared machine, not portable constants or universal opcode prices.

Queue roundtrips are excluded from like-for-like interpretation because Rust uses an in-process queue and lacks RXAPI integration.
SAY uses an eight-byte line redirected to /dev/null and includes each runtime's buffering policy, while command rows include a shell process.
REPLY's process-wall differences were inconsistent, so its public cost and ratio are blank and its raw sender-loop/counter evidence is retained.
EXIT measures complete program startup and termination, so it does not establish the cost of the EXIT instruction alone.

The inventory covers instruction families and selected subforms rather than the complete language/library surface.
It does not enumerate every builtin, every method, all operand sizes, message setters, USE aliases/defaults/LOCAL, ADDRESS redirection, or contended GUARD.
Numeric cases record initialized operand spelling, precision, and separately computed integer operands where relevant.
The CSV and interactive report expose fixtures, units, sample ranges, quality flags and counter qualifiers so those boundaries remain visible.

Files
* costs.csv: complete published cost table, with invalid costs withheld.
* catalog.json: full annotated catalog.
* instruction-coverage.csv and .json: all 40 instruction-family mappings.
* environment.json: provenance, formulas and measurement limitations.
* report.html: standalone sortable and filterable table.
* comparison.svg: selected fixed-kernel time ratios.
* <case>/runs.json: raw timings, correctness status and counter records.
* <case>/width-1.rex and width-16.rex: exact measured programs.
* results.json: raw sweep output, including the failed label probe and invalid REPLY subtraction.
'''
(OUT/'README.txt').write_text(method)

selected=[('nop','NOP'),('assign_simple_read','Simple variable assignment'),('increment','Increment i = i + 1'),('arith_tagged_plus','Computed-integer addition'),('arith_decimal_plus','Decimal addition'),('arith_decimal_multiply','Decimal multiplication'),('prefix_plus','Unary plus, integer spelling'),('compare_equal','Numeric-string equality'),('do-controlled','Controlled DO + NOP / iteration'),('do-while','WHILE + increment / iteration'),('select-hit','SELECT, second WHEN hit'),('internal-call-return','Internal CALL + RETURN'),('message-send-return','Message send + RETURN'),('method-use-expose','Method + EXPOSE + USE + RETURN'),('numeric_digits9','NUMERIC DIGITS 9'),('trace_off','TRACE OFF'),('parse_words','PARSE VAR, three words'),('parse_value','PARSE VALUE, three words'),('interpret_constant','INTERPRET short assignment')]
lookup={r['id']:r for r in catalog}
w=1000;h=100+len(selected)*30
left=360;right=850
x=lambda ratio:left+(math.log2(ratio)+2)/5*(right-left)
svg=[f'<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" viewBox="0 0 {w} {h}">', '<rect width="100%" height="100%" fill="white"/>','<g font-family="sans-serif" font-size="13" fill="#17212b">','<text x="20" y="25" font-size="18">Rust / oracle elapsed cost: selected fixed kernels</text>','<text x="20" y="47">Lower is faster. Logarithmic ratio axis. Flags retained in the full table.</text>']
for tick in [.25,.5,1,2,4,8]:
    xx=x(tick)
    svg += [f'<line x1="{xx}" y1="65" x2="{xx}" y2="{h-35}" stroke="{"#555" if tick==1 else "#ddd"}"/>', f'<text x="{xx}" y="{h-15}" text-anchor="middle">{tick:g}x</text>']
for index,(id,label) in enumerate(selected):
    r=lookup[id];ratio=r['rust_over_oracle'];yy=84+30*index;end=x(ratio);start=x(1)
    color='#2171b5' if ratio<1 else '#c65d17'
    flag=' †' if r['quality']!='stable' else ''
    svg += [f'<text x="20" y="{yy+4}">{html.escape(label+flag)}</text>',f'<rect x="{min(start,end)}" y="{yy-8}" width="{max(2,abs(end-start))}" height="16" fill="{color}"/>',f'<text x="{end+6 if ratio>=1 else end-6}" y="{yy+4}" text-anchor="{"start" if ratio>=1 else "end"}">{ratio:.2f}x</text>']
svg += ['</g></svg>']
(OUT/'comparison.svg').write_text('\n'.join(svg))

data=json.dumps(flat).replace('<','\\u003c')
page=r'''<!doctype html><html lang="en"><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>Rexx instruction cost inventory</title>
<style>body{font:15px system-ui;margin:24px;color:#17212b}h1{font-size:25px}p{max-width:1100px;line-height:1.5}.tools{display:flex;gap:18px;align-items:center;flex-wrap:wrap;margin:20px 0}input[type=search]{padding:8px;min-width:280px}select{padding:8px}table{border-collapse:collapse;width:100%;font-size:13px}th,td{border-bottom:1px solid #ddd;padding:9px;text-align:left;vertical-align:top}th{position:sticky;top:0;background:#edf2f7;cursor:pointer}.num{text-align:right;white-space:nowrap}code{white-space:pre-wrap;display:block;max-width:440px}.fast{color:#1565a3}.slow{color:#a8460a}.flag{color:#8b4c00}small{font-size:11px}.scroll{overflow:auto}details{max-width:500px}a{color:#1565a3}</style>
<h1>Rexx instruction cost inventory</h1>
<p>132 repeated-kernel costs, one unresolved REPLY row, and a whole-process EXIT row. <b>Rust/oracle below 1 means Rust is faster.</b> Costs include the exact kernel shown, including assignment or call context where required. Composite loop rows report their stated unit, not the entire loop.</p>
<p>Same frozen binaries as the earlier comparison; Rust source revision unverified. CPU 13, SMT sibling 29 monitored, 2 GiB cap. Three paired elapsed samples and separate nonmultiplexed user-mode counter pairs. Sample ranges are observed ranges, not confidence intervals. Queue rows exercise different backends because Rust lacks RXAPI integration. I/O and asynchronous rows need their stated timing boundaries.</p>
<p><a href="costs.csv">CSV</a> · <a href="catalog.json">Full JSON</a> · <a href="instruction-coverage.csv">All 40 instruction families</a> · <a href="README.txt">Method and limitations</a> · <a href="environment.json">Build fingerprints</a> · <a href="comparison.svg">Comparison chart</a></p>
<div class="tools"><input id="search" type="search" placeholder="Search case, keyword, body or notes"><select id="group"><option value="">All groups</option></select><label><input type="checkbox" id="comparable"> Fixed kernels only</label><label><input type="checkbox" id="stable"> Unflagged only</label><span id="count"></span></div>
<div class="scroll"><table><thead><tr><th data-key="id">Kernel</th><th data-key="oracle_ns">Oracle ns/unit</th><th data-key="rust_ns">Rust ns/unit</th><th data-key="rust_over_oracle">Rust/oracle</th><th data-key="oracle_instructions_u">Oracle user instructions</th><th data-key="rust_instructions_u">Rust user instructions</th><th data-key="quality">Quality</th><th>Unit and fixture</th></tr></thead><tbody id="rows"></tbody></table></div>
<script>const data=__DATA__;let sortKey='id',ascending=true;const $=id=>document.getElementById(id);const groups=[...new Set(data.map(r=>r.group))].sort();for(const g of groups){const o=document.createElement('option');o.value=g;o.textContent=g;$('group').append(o)}
function cell(tr,text,cls=''){const td=document.createElement('td');td.textContent=text;td.className=cls;tr.append(td);return td}function fmt(v){return v===null||v===''||v===undefined?'—':Number(v).toLocaleString(undefined,{maximumFractionDigits:2})}
function render(){const query=$('search').value.toLowerCase();let rows=data.filter(r=>(!$('group').value||r.group===$('group').value)&&(!$('comparable').checked||r.comparable)&&(!$('stable').checked||r.quality==='stable')&&JSON.stringify(r).toLowerCase().includes(query));rows.sort((a,b)=>{let x=a[sortKey],y=b[sortKey];if(x===null||x===undefined)return 1;if(y===null||y===undefined)return -1;return (typeof x==='number'?x-y:String(x).localeCompare(String(y)))*(ascending?1:-1)});$('rows').replaceChildren();$('count').textContent=rows.length+' rows';for(const r of rows){const tr=document.createElement('tr');const title=cell(tr,r.id);const sub=document.createElement('small');sub.textContent='\n'+r.group+' · '+r.status;title.append(document.createElement('br'),sub);cell(tr,fmt(r.oracle_ns),'num');cell(tr,fmt(r.rust_ns),'num');cell(tr,fmt(r.rust_over_oracle),'num '+(r.rust_over_oracle<1?'fast':'slow'));cell(tr,fmt(r.oracle_instructions_u),'num');cell(tr,fmt(r.rust_instructions_u),'num');cell(tr,r.quality+(r.sibling_busy_pct!==''?' · sibling '+Number(r.sibling_busy_pct).toFixed(1)+'%':''),'flag');const detail=cell(tr,r.unit);const d=document.createElement('details'),s=document.createElement('summary');s.textContent='Fixture and notes';d.append(s);const code=document.createElement('code');code.textContent=r.body;d.append(code);const p=document.createElement('p');p.textContent=r.notes;d.append(p);const q=document.createElement('p');q.textContent='Covers: '+r.covers+'; class: '+r.comparison_class+'; observed Rust/oracle range: '+fmt(r.observed_ratio_min)+'–'+fmt(r.observed_ratio_max);d.append(q);detail.append(d);$('rows').append(tr)}}for(const id of ['search','group','comparable','stable'])$(id).addEventListener('input',render);for(const th of document.querySelectorAll('[data-key]'))th.addEventListener('click',()=>{ascending=sortKey===th.dataset.key?!ascending:true;sortKey=th.dataset.key;render()});render();</script></html>'''
(OUT/'report.html').write_text(page.replace('__DATA__',data))

archive=BASE/'rexx-instruction-cost-inventory.zip'
with zipfile.ZipFile(archive,'w',zipfile.ZIP_DEFLATED) as z:
    for name in ['inventory.py','inventory-cases.json','instruction-coverage.json','metadata.json','exit-measure.py','publish-inventory.py']:
        z.write(BASE/name,name)
    for p in OUT.rglob('*'):
        if p.is_file() and p.suffix in ['.json','.csv','.rex','.txt','.html','.svg']:
            z.write(p,str(p.relative_to(BASE)).replace(':','_'))
print(json.dumps(summary,indent=2))
print('archive',archive,'bytes',archive.stat().st_size)
