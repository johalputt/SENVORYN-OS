import json
import unittest
from pathlib import Path

try:
    import jsonschema
except ImportError:
    jsonschema = None

ROOT = Path(__file__).resolve().parents[1]

@unittest.skipIf(jsonschema is None, "install requirements-dev.txt to run JSON Schema contract tests")
class SchemaContractTests(unittest.TestCase):
    def test_all_schemas_are_valid_draft_2020_12(self):
        for path in sorted((ROOT / "schemas").glob("*.schema.json")):
            with self.subTest(schema=path.name):
                schema = json.loads(path.read_text(encoding="utf-8"))
                jsonschema.Draft202012Validator.check_schema(schema)

    def test_valid_request_fixture_passes(self):
        schema = json.loads((ROOT / "schemas" / "request-envelope.schema.json").read_text(encoding="utf-8"))
        sample = json.loads((ROOT / "examples" / "valid-request-envelope.json").read_text(encoding="utf-8"))
        jsonschema.Draft202012Validator(schema, format_checker=jsonschema.FormatChecker()).validate(sample)

    def test_operation_registry_fixture_passes(self):
        schema = json.loads((ROOT / "schemas" / "operation-registry.schema.json").read_text(encoding="utf-8"))
        sample = json.loads((ROOT / "examples" / "operation-registry.sample.json").read_text(encoding="utf-8"))
        jsonschema.Draft202012Validator(schema, format_checker=jsonschema.FormatChecker()).validate(sample)

    def test_change_capsule_fixture_passes(self):
        # The capsule the Rust executor reads has to be a capsule the schema
        # accepts. The Rust reader enforces the fields it needs; the schema
        # enforces the rest. A fixture that drifts from the schema would be
        # read by one and rejected by the other.
        schema = json.loads((ROOT / "schemas" / "change-capsule.schema.json").read_text(encoding="utf-8"))
        sample = json.loads((ROOT / "examples" / "capsule.sample.json").read_text(encoding="utf-8"))
        jsonschema.Draft202012Validator(schema, format_checker=jsonschema.FormatChecker()).validate(sample)

    def test_a_capsule_without_an_expected_precondition_is_rejected(self):
        schema = json.loads((ROOT / "schemas" / "change-capsule.schema.json").read_text(encoding="utf-8"))
        sample = json.loads((ROOT / "examples" / "capsule.sample.json").read_text(encoding="utf-8"))
        del sample["preconditions"][0]["expected"]
        with self.assertRaises(jsonschema.ValidationError):
            jsonschema.Draft202012Validator(schema).validate(sample)

    def test_unknown_field_is_rejected(self):
        schema = json.loads((ROOT / "schemas" / "request-envelope.schema.json").read_text(encoding="utf-8"))
        sample = json.loads((ROOT / "examples" / "valid-request-envelope.json").read_text(encoding="utf-8"))
        sample["unexpected_privilege"] = "root"
        with self.assertRaises(jsonschema.ValidationError):
            jsonschema.Draft202012Validator(schema).validate(sample)

if __name__ == "__main__":
    unittest.main()
