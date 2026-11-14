"""Meaningful packaging/structure regressions; these do not replace Rust tests."""
import json
import shutil
import subprocess
import sys
import tempfile
import unittest
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / 'scripts'))
import audit
from package import package_content


class ProjectTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        for name in ('content', 'native', 'docs'):
            shutil.copytree(ROOT / name, self.root / name,
                            ignore=shutil.ignore_patterns('target', '__pycache__'))
        for name in ('Cargo.toml', 'upstream.lock.json'):
            shutil.copy2(ROOT / name, self.root / name)

    def change_json(self, relative, change):
        path = self.root / relative
        data = json.loads(path.read_text())
        change(data)
        path.write_text(json.dumps(data))

    def test_project_structure_and_official_tool_provenance(self):
        self.assertEqual(audit.validate(self.root)['structural_audit'], 'passed')

    def test_mismatched_horizon_pin_fails(self):
        self.change_json('native/powercraft2/compatibility.json',
                         lambda d: d.update(horizon_revision='0' * 40))
        with self.assertRaisesRegex(audit.AuditError, 'pin mismatch'):
            audit.validate(self.root)

    def test_missing_facing_model_fails(self):
        self.change_json('content/assets/powercraft2/blockstates/conveyor.json',
                         lambda d: d['variants'].pop('facing=east'))
        with self.assertRaisesRegex(audit.AuditError, 'model variant'):
            audit.validate(self.root)

    def test_collision_drift_fails(self):
        self.change_json('content/data/powercraft2/horizon/block/conveyor.json',
                         lambda d: d['collision_shapes'].update(default=[]))
        with self.assertRaisesRegex(audit.AuditError, 'surface mismatch'):
            audit.validate(self.root)

    def test_foreign_namespace_fails(self):
        path = self.root / 'content/data/other_mod'
        path.mkdir()
        (path / 'anything.json').write_text('{}')
        with self.assertRaisesRegex(audit.AuditError, 'foreign namespace'):
            audit.validate(self.root)

    def test_duplicate_json_keys_fail(self):
        path = self.root / 'content/data/powercraft2/recipe/conveyor.json'
        path.write_text('{"type":"a","type":"b"}')
        with self.assertRaisesRegex(audit.AuditError, 'duplicate key'):
            audit.validate(self.root)

    def test_content_archive_reproducibility_and_payload(self):
        out = package_content(self.root)
        first = out.read_bytes()
        self.assertEqual(package_content(self.root).read_bytes(), first)
        with zipfile.ZipFile(out) as archive:
            self.assertIsNone(archive.testzip())
            names = archive.namelist()
            self.assertEqual(len(names), len(set(names)))
            self.assertIn('data/powercraft2/horizon/block/conveyor.json', names)
            self.assertIn('assets/powercraft2/textures/block/conveyor_top.png', names)
            for name in names:
                self.assertFalse(name.startswith('/') or '..' in Path(name).parts)

    def test_native_archive_reproducibility_and_pins(self):
        command = [sys.executable, str(self.root / 'native/tools/native.py'), 'package', 'powercraft2']
        subprocess.run(command, check=True, capture_output=True)
        out = self.root / 'native/target/powercraft2-0.1.0.hnmod'
        first = out.read_bytes()
        subprocess.run(command, check=True, capture_output=True)
        self.assertEqual(out.read_bytes(), first)
        with zipfile.ZipFile(out) as archive:
            self.assertIsNone(archive.testzip())
            manifest = json.loads(archive.read('horizon-native.json'))
            self.assertEqual(manifest['horizon'], json.loads((self.root / 'upstream.lock.json').read_text())['horizon']['revision'])
            self.assertEqual(manifest['hooks'], 381)
            self.assertEqual(archive.read('mixins/powercraft2/transport.rs'),
                             (self.root / 'native/powercraft2/mixins/powercraft2/transport.rs').read_bytes())
            self.assertIn('mixins/powercraft2/LICENSE', archive.namelist())

    def test_modified_official_tool_fails(self):
        with (self.root / 'native/tools/native.py').open('a') as file:
            file.write('\n# unexpected modification\n')
        with self.assertRaisesRegex(audit.AuditError, 'modified official tool'):
            audit.validate(self.root)

    def test_native_packager_rejects_inconsistent_pin(self):
        self.change_json('native/powercraft2/compatibility.json',
                         lambda d: d.update(horizon_revision='0' * 40))
        result = subprocess.run([sys.executable, str(self.root / 'native/tools/native.py'),
                                 'package', 'powercraft2'], capture_output=True, text=True)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn('pins', result.stderr)


if __name__ == '__main__':
    unittest.main()
