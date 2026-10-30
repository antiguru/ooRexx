"""Controls for the test entry preflight."""

from pathlib import Path
import tempfile
import unittest

import test_preflight as preflight


class Prerequisites(unittest.TestCase):
    def test_empty_checkout_reports_native_suite_and_docs_paths(self):
        with tempfile.TemporaryDirectory() as directory:
            missing = preflight.missing_inputs(Path(directory))
        self.assertIn("build/lib/librxregexp.so", missing)
        self.assertIn("ootest/ooTest.frm", missing)
        self.assertIn("oodocs/rexxref/en-US/fundclasses.xml", missing)

    def test_complete_inputs_pass_and_missing_leaf_is_reported(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            for path in (*preflight.NATIVE, *preflight.SUITE_FILES, *preflight.DOCS):
                target = root / path
                target.parent.mkdir(parents=True, exist_ok=True)
                target.touch()
            for path in preflight.SUITE_DIRS:
                (root / path).mkdir(parents=True, exist_ok=True)
            self.assertEqual(preflight.missing_inputs(root), [])
            (root / "build/lib/librxmath.so").unlink()
            self.assertEqual(preflight.missing_inputs(root), ["build/lib/librxmath.so"])

    def test_oracle_sibling_cannot_satisfy_checkout_local_inputs(self):
        with tempfile.TemporaryDirectory() as directory:
            parent = Path(directory)
            library = parent / "oracle/build/lib/librxregexp.so"
            library.parent.mkdir(parents=True)
            library.touch()
            root = parent / "worktree"
            root.mkdir()
            self.assertIn("build/lib/librxregexp.so", preflight.missing_inputs(root))


if __name__ == "__main__":
    unittest.main()
