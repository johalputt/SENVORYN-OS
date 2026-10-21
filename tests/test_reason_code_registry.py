"""Checks the Python model's reason codes against the denial-code registry.

The registry in ``errors/denial-codes.json`` is the authority on which reason
codes exist. The Rust implementation in ``crates/senvoryn-scp`` checks its
own list against the same registry. These tests check the model's list in
both directions, so a code added to one side and not the other fails the
build rather than surviving to a release.
"""

import json
import os
import unittest

from scp_reference import ALL_GATES_PASSED, ALL_REASON_CODES

REGISTRY_PATH = os.path.join(
    os.path.dirname(os.path.dirname(os.path.abspath(__file__))),
    "errors",
    "denial-codes.json",
)


def _registry_codes():
    with open(REGISTRY_PATH, encoding="utf-8") as handle:
        document = json.load(handle)
    return {entry["code"]: entry for entry in document["codes"]}


class ReasonCodeRegistryTests(unittest.TestCase):
    def test_every_code_the_model_can_produce_is_in_the_registry(self):
        registry = _registry_codes()
        for code in ALL_REASON_CODES:
            if code == ALL_GATES_PASSED:
                # The permitting reason is not a denial, so it is not in a
                # denial registry. It is still a reason code, and it is still
                # reported.
                continue
            self.assertIn(code, registry, f"{code} is not in the denial-code registry")

    def test_every_code_in_the_registry_is_known_to_the_model(self):
        for code in _registry_codes():
            self.assertIn(
                code,
                ALL_REASON_CODES,
                f"{code} is in the denial-code registry but the model cannot produce it",
            )

    def test_the_registry_has_no_duplicate_codes(self):
        with open(REGISTRY_PATH, encoding="utf-8") as handle:
            document = json.load(handle)
        codes = [entry["code"] for entry in document["codes"]]
        self.assertEqual(len(codes), len(set(codes)), "a code appears twice in the registry")

    def test_every_code_declares_a_known_default_status(self):
        for code, entry in _registry_codes().items():
            self.assertIn(
                entry["default"],
                {"DENY", "CHALLENGE", "DEFER", "QUARANTINE"},
                f"{code} declares an unknown default status",
            )
            self.assertTrue(entry["class"], f"{code} declares no class")

    def test_the_two_approval_codes_default_to_a_challenge(self):
        registry = _registry_codes()
        for code in ("FRESH_ACTION_BOUND_APPROVAL_REQUIRED", "STRONG_APPROVAL_REQUIRED"):
            self.assertEqual(
                registry[code]["default"],
                "CHALLENGE",
                f"{code} should default to CHALLENGE",
            )
        for code, entry in registry.items():
            if code in ("FRESH_ACTION_BOUND_APPROVAL_REQUIRED", "STRONG_APPROVAL_REQUIRED"):
                continue
            if code == "POLICY_EPOCH_NOT_YET_IN_FORCE":
                continue
            self.assertEqual(
                entry["default"],
                "DENY",
                f"{code} should default to DENY",
            )

    def test_the_deferral_code_is_the_only_one_that_postpones(self):
        # The one code that postpones rather than refuses. A request made
        # under a policy epoch this node has not reached yet is not "no", and
        # a reader of the registry has to be told which it is.
        registry = _registry_codes()
        self.assertEqual(
            registry["POLICY_EPOCH_NOT_YET_IN_FORCE"]["default"],
            "DEFER",
            "POLICY_EPOCH_NOT_YET_IN_FORCE should default to DEFER",
        )
        postponed = [code for code, entry in registry.items() if entry["default"] == "DEFER"]
        self.assertEqual(
            postponed,
            ["POLICY_EPOCH_NOT_YET_IN_FORCE"],
            "a code other than the deferral code postpones",
        )


if __name__ == "__main__":
    unittest.main()
