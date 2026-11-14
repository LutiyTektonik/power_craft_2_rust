"""Regressions for the source registry's comment and nested-recipe parser."""
import importlib.util
import json
import sys
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location('original_inventory', ROOT / 'scripts/inventory-original.py')
inventory = importlib.util.module_from_spec(spec)
sys.modules[spec.name] = inventory
spec.loader.exec_module(inventory)


class InventoryTests(unittest.TestCase):
    def test_comment_removal_preserves_literals_and_evidence_lines(self):
        source = '"https://example.test"; // disabled()\n/* hidden\n block */ "/* literal */"'
        result = inventory.without_comments(source)
        self.assertIn('"https://example.test"', result)
        self.assertIn('"/* literal */"', result)
        self.assertNotIn('disabled()', result)
        self.assertEqual(source.count('\n'), result.count('\n'))

    def test_nested_recipe_arguments_do_not_split_inside_arrays_or_strings(self):
        source = '(item(a, 2), new Object[] {"A,B", \'X\', other(1,2)}, 4);'
        values, end = inventory.arguments(source, 0)
        self.assertEqual(len(values), 3)
        self.assertEqual(values[0], 'item(a, 2)')
        self.assertEqual(values[2], '4')
        self.assertEqual(source[end:], ');')

    def test_registry_rejects_changed_original_source_before_indexing(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / 'docs').mkdir()
            (root / 'docs/ORIGINAL_SOURCE_INDEX.json').write_text(json.dumps({
                'revision': inventory.REVISION, 'tree_truncated': False,
                'files': [{'path': 'Changed.java', 'git_blob': '0' * 40, 'bytes': 1}]}))
            (root / 'Changed.java').write_text('modified source')
            with self.assertRaisesRegex(ValueError, 'Original source mismatch'):
                inventory.inventory(root, root)


if __name__ == '__main__':
    unittest.main()
