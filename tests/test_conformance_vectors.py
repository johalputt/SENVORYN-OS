"""Executes the shared conformance vectors against the Python reference model.

The Rust implementation in ``crates/senvoryn-scp`` executes the same file. A
disagreement between the two about any decision or reason code fails both
suites, and CI runs both. That is the mechanism that keeps the prototype and
the implementation from drifting into two authorities on the same question.
"""

import json
import os
import re
import unittest

from scp_reference import (
    ALL_GATES_PASSED,
    ALL_REASON_CODES,
    AuditState,
    Budget,
    Capability,
    OperationEntry,
    PolicyContext,
    Provenance,
    Request,
    ResolvedResource,
    Status,
    attenuate,
    evaluate,
)

VECTORS_PATH = os.path.join(
    os.path.dirname(os.path.dirname(os.path.abspath(__file__))),
    "conformance",
    "vectors",
    "vectors.json",
)


def _load():
    with open(VECTORS_PATH, encoding="utf-8") as handle:
        return json.load(handle)


VECTORS = _load()


def _operation_entry(raw):
    return OperationEntry(
        operation_id=raw["operation_id"],
        risk_tier=raw["risk_tier"],
        approval_requirements=frozenset(raw["approval_requirements"]),
        allowed_modes=frozenset(raw["allowed_modes"]),
        resource_types=frozenset(raw["resource_types"]),
        max_duration_ms=raw.get("max_duration_ms"),
        max_operation_count=raw.get("max_operation_count"),
        required_preconditions=frozenset(raw.get("required_preconditions", [])),
        audit_class=raw.get("audit_class", "STANDARD"),
        reversibility=raw.get("reversibility", "UNKNOWN"),
        external_effects=frozenset(raw.get("external_effects", [])),
    )


def _capability(raw):
    return Capability(
        capability_id=raw["capability_id"],
        subject_id=raw["subject_id"],
        resource_id=raw["resource_id"],
        resource_type=raw["resource_type"],
        operations=frozenset(raw["operations"]),
        purposes=frozenset(raw["purposes"]),
        not_before=raw["not_before"],
        expires_at=raw["expires_at"],
        policy_epoch=raw["policy_epoch"],
        provenance=Provenance(raw["provenance"]),
        delegation_depth=raw.get("delegation_depth", 0),
        max_uses=raw.get("max_uses"),
    )


def _request(raw):
    budget = raw.get("budget")
    return Request(
        request_id=raw["request_id"],
        principal_id=raw["principal_id"],
        recipient_id=raw["recipient_id"],
        resource_id=raw["resource_id"],
        resource_type=raw["resource_type"],
        operation=raw["operation"],
        purpose=raw["purpose"],
        policy_epoch=raw["policy_epoch"],
        issued_at=raw["issued_at"],
        expires_at=raw["expires_at"],
        nonce=raw["nonce"],
        risk_claim=raw["risk_claim"],
        approval_digest=raw.get("approval_digest"),
        budget=None if budget is None else Budget(**budget),
    )


def _context(raw, registry, registry_epoch):
    return PolicyContext(
        local_node_id=raw["local_node_id"],
        current_policy_epoch=raw["current_policy_epoch"],
        now_monotonic=raw["now_monotonic"],
        now_wall_ms=raw["now_wall_ms"],
        clock_state=raw["clock_state"],
        registry_valid_from_ms=1_767_225_600_000,
        registry_valid_until_ms=1_767_225_600_000 + 86_400_000,
        mode=raw["mode"],
        known_principals=frozenset(raw["known_principals"]),
        resolved_resources={
            name: ResolvedResource(
                canonical_id=entry["canonical_id"],
                resource_type=entry["resource_type"],
                version=entry["version"],
            )
            for name, entry in raw["resolved_resources"].items()
        },
        operations=tuple(_operation_entry(entry) for entry in registry),
        registry_epoch=registry_epoch,
        audit_state=AuditState(raw["audit_state"]),
        action_digest=raw.get("action_digest"),
        revoked_nonces=frozenset(raw.get("revoked_nonces", [])),
        revoked_capabilities=frozenset(raw.get("revoked_capabilities", [])),
        quarantined_principals=frozenset(raw.get("quarantined_principals", [])),
        strong_approval_granted=raw.get("strong_approval_granted", False),
        satisfied_preconditions=frozenset(raw.get("satisfied_preconditions", [])),
    )


def _merged(base, overrides):
    """Replace the named fields wholesale. No nesting, so there is no merge
    rule for this implementation and the Rust one to disagree about."""
    merged = dict(base)
    merged.update(overrides or {})
    return merged


class ConformanceVectorTests(unittest.TestCase):
    def test_every_case_matches_the_expected_decision(self):
        for case in VECTORS["cases"]:
            with self.subTest(case=case["id"]):
                registry = case.get("registry", VECTORS["registry"])
                request = _request(_merged(VECTORS["request"], case.get("request")))
                if "capability" in case:
                    capability = (
                        None
                        if case["capability"] is None
                        else _capability(_merged(VECTORS["capability"], case["capability"]))
                    )
                else:
                    capability = _capability(VECTORS["capability"])
                registry_epoch = case.get("registry_epoch", VECTORS["registry_epoch"])
                context = _context(
                    _merged(VECTORS["context"], case.get("context")), registry, registry_epoch
                )

                decision = evaluate(request, capability, context)

                self.assertEqual(
                    decision.status,
                    Status(case["expected"]["status"]),
                    f"{case['id']} produced {decision.status} / {decision.reason}",
                )
                self.assertEqual(
                    decision.reason,
                    case["expected"]["reason"],
                    f"{case['id']} produced the wrong reason",
                )
                self.assertFalse(
                    decision.effect_performed,
                    f"{case['id']} reported an effect",
                )
                # A deferral names the thing it waits for, and nothing else
                # does. The epoch is the deferral's substance, so the two
                # implementations have to agree about which epoch it is.
                expected_epoch = case["expected"].get("waits_for_policy_epoch")
                if expected_epoch is None:
                    self.assertIsNone(
                        decision.deferral,
                        f"{case['id']} produced a deferral the case does not expect",
                    )
                else:
                    self.assertIsNotNone(
                        decision.deferral,
                        f"{case['id']} produced no deferral",
                    )
                    self.assertEqual(
                        decision.deferral.policy_epoch,
                        expected_epoch,
                        f"{case['id']} waits for the wrong policy epoch",
                    )
                # A quarantine names the principal it holds, and nothing
                # else does. The principal is the isolation's substance, so
                # the two implementations have to agree about which
                # principal it is.
                expected_principal = case["expected"].get("quarantined_principal")
                if expected_principal is None:
                    self.assertIsNone(
                        decision.isolation,
                        f"{case['id']} produced a quarantine the case does not expect",
                    )
                else:
                    self.assertIsNotNone(
                        decision.isolation,
                        f"{case['id']} produced no quarantine",
                    )
                    self.assertEqual(
                        decision.isolation.principal,
                        expected_principal,
                        f"{case['id']} holds the wrong principal",
                    )
                if case["expected"]["status"] != "PERMIT":
                    self.assertIsNone(
                        decision.capability_id,
                        f"{case['id']} scoped a capability it did not permit",
                    )

    def test_every_attenuation_case_matches(self):
        for case in VECTORS["attenuation_cases"]:
            with self.subTest(case=case["id"]):
                parent = _capability(_merged(VECTORS["capability"], case.get("parent")))
                child = _capability(_merged(VECTORS["capability"], case.get("child")))
                self.assertEqual(
                    attenuate(parent, child),
                    case["expected"],
                    f"{case['id']} produced the wrong attenuation result",
                )

    def test_the_permitted_case_reports_the_permitting_reason(self):
        permitted = [case for case in VECTORS["cases"] if case["expected"]["status"] == "PERMIT"]
        self.assertTrue(permitted, "the vector set contains no permitted case")
        for case in permitted:
            self.assertEqual(case["expected"]["reason"], ALL_GATES_PASSED)

    def test_the_vector_file_covers_the_specification(self):
        """The file must carry at least one case per conformance requirement,
        and every case must have its own identifier.

        The count is checked here and in the Rust suite. That duplication is
        deliberate: the file is shared, so a case removed from it must fail
        in both implementations rather than in neither.
        """
        self.assertGreaterEqual(len(VECTORS["cases"]), 30)
        self.assertGreaterEqual(len(VECTORS["attenuation_cases"]), 11)
        identifiers = [case["id"] for case in VECTORS["cases"]]
        identifiers += [case["id"] for case in VECTORS["attenuation_cases"]]
        self.assertEqual(len(identifiers), len(set(identifiers)), "a case identifier repeats")

    def test_every_registry_code_is_produced_by_a_case_or_is_ahead_of_the_code(self):
        # A code in the registry that nothing produces is a promise the
        # registry cannot keep: a reader consulting it would expect a refusal
        # that cannot happen. The two codes named here name refusals the
        # decision path does not produce. UNSUPPORTED_CRITICAL_SCHEMA names
        # the transport's refusal of a document whose schema this node does
        # not support - a refusal that is not a decision, because the request
        # never existed - and ANTI_ROLLBACK_VIOLATION names the updater's
        # refusal of a downgrade, and there is no updater.
        ahead_of_the_code = {"UNSUPPORTED_CRITICAL_SCHEMA", "ANTI_ROLLBACK_VIOLATION"}
        produced = {case["expected"]["reason"] for case in VECTORS["cases"]}
        for code in ALL_REASON_CODES:
            if code in produced:
                continue
            self.assertIn(
                code,
                ahead_of_the_code,
                f"{code} is in the denial-code registry and no case produces it. Either give it "
                "a case, or list it as a code that names a refusal the decision path does not "
                "produce.",
            )
        for code in ahead_of_the_code:
            self.assertNotIn(
                code,
                produced,
                f"{code} is produced by a case now, so it no longer belongs to the list of "
                "codes that are ahead of the code",
            )

    def test_every_case_names_the_requirement_it_covers(self):
        # A case cites either a constitutional invariant (INV-nn) or the
        # conformance test identifier it exercises (SCP-nnn and the rest).
        pattern = re.compile(r"^(INV|SCP|CAP|CC|OC|AI|INS|UPD|PRIV|FED|EVID)-\d{2,3}$")
        for case in VECTORS["cases"]:
            with self.subTest(case=case["id"]):
                self.assertRegex(case["requirement"], pattern)
                self.assertIn(case["gate"], range(1, 17), "the gate is outside the sixteen")
                self.assertIn(
                    case["expected"]["status"],
                    {"PERMIT", "DENY", "CHALLENGE", "DEFER", "QUARANTINE"},
                )
                self.assertIn(case["expected"]["reason"], ALL_REASON_CODES)


if __name__ == "__main__":
    unittest.main()
