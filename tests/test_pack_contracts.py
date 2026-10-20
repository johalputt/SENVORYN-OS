import json
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

class PackContractTests(unittest.TestCase):
    def test_all_json_files_parse(self):
        for p in ROOT.rglob("*.json"):
            with self.subTest(path=str(p.relative_to(ROOT))):
                json.loads(p.read_text(encoding="utf-8"))

    def test_change_capsule_state_machine_has_valid_endpoints(self):
        m = json.loads((ROOT / "state-machines" / "change-capsule-transitions.json").read_text())
        states = set(m["states"])
        for edge in m["transitions"]:
            self.assertIn(edge["from"], states)
            self.assertIn(edge["to"], states)
        self.assertIn("COMMITTED", m["terminal_states"])
        self.assertIn("No transition skips VALIDATED and AUTHORIZED.", m["global_invariants"])

    def test_denial_codes_are_unique(self):
        d = json.loads((ROOT / "errors" / "denial-codes.json").read_text())
        codes = [x["code"] for x in d["codes"]]
        self.assertEqual(len(codes), len(set(codes)))

    def test_operation_registry_sample_has_unique_operation_ids(self):
        r = json.loads((ROOT / "examples" / "operation-registry.sample.json").read_text())
        ids = [x["operation_id"] for x in r["operations"]]
        self.assertEqual(len(ids), len(set(ids)))
        delete = next(x for x in r["operations"] if x["operation_id"] == "delete")
        self.assertEqual(delete["risk_tier"], "R3")
        self.assertIn("STRONG_AUTH", delete["approval_requirements"])

if __name__ == "__main__":
    unittest.main()
