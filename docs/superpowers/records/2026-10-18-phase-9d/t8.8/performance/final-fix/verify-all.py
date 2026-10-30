#!/usr/bin/env python3
"""Apply the frozen verifier to all historical and final immutable cohorts."""

import importlib.util
from pathlib import Path

adapter_path = Path(__file__).resolve().parent / 'q-r.py'
spec = importlib.util.spec_from_file_location('adapter', adapter_path)
adapter = importlib.util.module_from_spec(spec)
spec.loader.exec_module(adapter)
profiles = adapter.load_profiles()
adapter.reuse_script('verify-profiles.py', profiles,
                     ['final-all-complete', *profiles.GROUPS])
