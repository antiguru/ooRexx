#!/usr/bin/env python3
"""Adapt the retained performance tools for an explicitly accepted Q-to-R pin."""

import hashlib
import importlib.util
import json
from pathlib import Path
import re
import sys

CACHE = Path(__file__).resolve().parent
OUTPUT = CACHE / 'metrics'
ORIGINAL = CACHE / 'originals'
RECORDS = Path('/home/moritz/dev/repos/ooRexx-rust-rewrite/docs/superpowers/records/2026-10-18-phase-9d/t8.8/performance')
Q_HASH = '22c317baa3ed111935a6577ac33097e428a53b64eb64b4697af2b85664e3f8eb'
R_PATH = Path('/tmp/codex-p9d-t88-pins/8.8-final-fix-rexx')


def sha(path):
    with Path(path).open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def accepted_manifest():
    manifest = json.loads((CACHE / 'r-inputs.json').read_text())
    assert manifest['status'] == 'ACCEPTED_FOR_COMPARISON' and manifest['profiling_authorized'], 'Root has not authorized R profiling'
    assert re.fullmatch('[0-9a-f]{40}', manifest['runtime_source_revision'] or ''), 'Authoritative R source revision required'
    assert manifest['authorization'] and manifest['review_evidence_revision'], 'Root review acceptance required'
    pins = {pin['name']: pin for pin in manifest['pins']}
    assert pins['q']['sha256'] == Q_HASH
    assert Path(pins['r']['path']) == R_PATH
    assert re.fullmatch('[0-9a-f]{64}', pins['r']['sha256'] or ''), 'Authoritative R pin hash required'
    for pin in pins.values():
        assert sha(pin['path']) == pin['sha256'], pin['path']
    return manifest, pins


def load_profiles():
    manifest, pins = accepted_manifest()
    prepared = json.loads((CACHE / 'prepared-inputs.json').read_text())
    for item in prepared['frozen_helpers']:
        assert sha(CACHE / item['path']) == item['sha256'], item['path']
    assert sha(RECORDS / 'inputs.json') == prepared['original_instrument_manifest_sha256']
    spec = importlib.util.spec_from_file_location('profiles', ORIGINAL / 'profile-slices.py')
    profiles = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(profiles)
    profiles.GROUPS['q-r'] = ['q', 'r']
    profiles.PINS['r'] = R_PATH
    profiles.EXPECTED['r'] = pins['r']['sha256']
    original_verify = profiles.verify_inputs

    def verify():
        accepted_manifest()
        # The retained verifier reads the original immutable instrument manifests.
        current = profiles.RECORDS
        profiles.RECORDS = RECORDS
        try:
            original_verify()
        finally:
            profiles.RECORDS = current

    profiles.verify_inputs = verify
    profiles.RECORDS = OUTPUT
    profiles.__file__ = str(Path(__file__).resolve())
    OUTPUT.mkdir(exist_ok=True)
    (OUTPUT / 'r-inputs.json').write_text(json.dumps(manifest, indent=2) + '\n')
    verify()
    return profiles


def reuse_script(name, profiles, arguments):
    source = (ORIGINAL / name).read_text()
    start = source.index('ROOT = Path(__file__).resolve().parent\n')
    end = source.index('spec.loader.exec_module(profiles)\n', start) + len('spec.loader.exec_module(profiles)\n')
    source = source[:start] + 'ROOT = Path(' + repr(str(OUTPUT)) + ')\nprofiles = _profiles\n' + source[end:]
    if name == 'analyze-slices.py':
        profiles.GROUPS = {'q-r': ['q', 'r']}
        start = source.index('relations = ')
        end = source.index('partial = ', start)
        source = source[:start] + 'relations = {"q-r": [("8.8-final-fix", "q", "r")]}\n' + source[end:]
    if name == 'q-residual.py':
        source = source.replace('"q"', '"r"').replace('"p"', '"q"').replace('"p-q"', '"q-r"')
        source = source.replace('q_', 'r_').replace('q-vs-', 'r-vs-')
        source = source.replace('Report Q ', 'Report R ').replace('Q versus ', 'R versus ').replace('Q/P', 'R/Q')
    sys.argv = [str(ORIGINAL / name), *arguments]
    exec(compile(source, str(ORIGINAL / name), 'exec'), {'__name__': '__main__', '__file__': str(ORIGINAL / name), '_profiles': profiles})


if __name__ == '__main__':
    action, *arguments = sys.argv[1:]
    if action == 'status':
        print((CACHE / 'r-inputs.json').read_text())
        sys.exit(0)
    profiles = load_profiles()
    if action == 'run':
        group, label, maximum, jobs = arguments
        assert group == 'q-r'
        sys.exit(profiles.run_batch(group, label, int(maximum), int(jobs)))
    if action == 'analyze':
        reuse_script('analyze-slices.py', profiles, arguments)
    elif action == 'verify':
        assert len(arguments) == 1
        reuse_script('verify-profiles.py', profiles, [arguments[0], 'q-r'])
    elif action == 'residual':
        assert not arguments
        reuse_script('q-residual.py', profiles, [])
    else:
        raise SystemExit('Expected status, run, analyze, verify, or residual')
