"""Unit tests for the SCP reference decision model.

The shared conformance vectors in ``conformance/vectors/`` cover the gate
order and the reason codes. These tests cover what the vectors cannot: the
type-level guarantees, and the behaviour of the parts of the model that the
vectors exercise only indirectly.
"""

import unittest

from scp_reference import (
    ALL_GATES_PASSED,
    AuditState,
    Budget,
    Capability,
    OperationEntry,
    PolicyContext,
    Provenance,
    Request,
    Status,
    attenuate,
    evaluate,
)

REGISTRY = (
    OperationEntry(
        operation_id="read",
        risk_tier="R0",
        approval_requirements=frozenset({"NONE"}),
        allowed_modes=frozenset({"NORMAL", "READ_ONLY"}),
        resource_types=frozenset({"file_object"}),
        max_duration_ms=30000,
        max_operation_count=1,
        required_preconditions=frozenset({"resource_exists"}),
        audit_class="STANDARD",
    ),
    OperationEntry(
        operation_id="write",
        risk_tier="R2",
        approval_requirements=frozenset({"USER_CONFIRMATION"}),
        allowed_modes=frozenset({"NORMAL"}),
        resource_types=frozenset({"file_object"}),
        max_duration_ms=60000,
        max_operation_count=1,
        required_preconditions=frozenset({"resource_version_matches"}),
        audit_class="SENSITIVE",
    ),
    OperationEntry(
        operation_id="delete",
        risk_tier="R3",
        approval_requirements=frozenset({"STRONG_AUTH", "USER_CONFIRMATION"}),
        allowed_modes=frozenset({"NORMAL"}),
        resource_types=frozenset({"file_object"}),
        max_duration_ms=60000,
        max_operation_count=1,
        required_preconditions=frozenset({"exact_target_displayed"}),
        audit_class="SECURITY_CRITICAL",
    ),
)


def make_request(**overrides):
    base = dict(
        request_id="req-0000000001",
        principal_id="user:alice",
        recipient_id="node:local",
        resource_id="file:report",
        resource_type="file_object",
        operation="read",
        purpose="user-request",
        policy_epoch=7,
        issued_at=10,
        expires_at=50,
        nonce="nonce-unique-0001",
        risk_claim="R0",
    )
    base.update(overrides)
    return Request(**base)


def make_capability(**overrides):
    base = dict(
        capability_id="cap-00000001",
        subject_id="user:alice",
        resource_id="file:report",
        resource_type="file_object",
        operations=frozenset({"read"}),
        purposes=frozenset({"user-request"}),
        not_before=1,
        expires_at=60,
        policy_epoch=7,
        provenance=Provenance.VERIFIED,
        delegation_depth=3,
    )
    base.update(overrides)
    return Capability(**base)


def make_context(**overrides):
    base = dict(
        local_node_id="node:local",
        current_policy_epoch=7,
        now_monotonic=20,
        now_wall_ms=1_767_225_600_000,
        clock_state="TRUSTED",
        registry_valid_from_ms=1_767_225_600_000,
        registry_valid_until_ms=1_767_225_600_000 + 86_400_000,
        mode="NORMAL",
        known_principals=frozenset({"user:alice"}),
        resolved_resources={"file:report": "v1"},
        operations=REGISTRY,
        registry_epoch=7,
        audit_state=AuditState.READY,
        satisfied_preconditions=frozenset(
            {"resource_exists", "resource_version_matches", "exact_target_displayed"}
        ),
    )
    base.update(overrides)
    return PolicyContext(**base)


class SCPReferenceTests(unittest.TestCase):
    def test_valid_scoped_request_permits_without_effect(self):
        decision = evaluate(make_request(), make_capability(), make_context())
        self.assertEqual(decision.status, Status.PERMIT)
        self.assertEqual(decision.reason, ALL_GATES_PASSED)
        self.assertFalse(decision.effect_performed)

    def test_unknown_principal_denied(self):
        ctx = make_context(known_principals=frozenset())
        self.assertEqual(evaluate(make_request(), make_capability(), ctx).reason, "UNKNOWN_PRINCIPAL")

    def test_missing_capability_denied(self):
        self.assertEqual(evaluate(make_request(), None, make_context()).reason, "MISSING_CAPABILITY")

    def test_wrong_recipient_denied(self):
        req = make_request(recipient_id="node:other")
        self.assertEqual(evaluate(req, make_capability(), make_context()).reason, "WRONG_RECIPIENT")

    def test_wrong_subject_denied(self):
        cap = make_capability(subject_id="user:bob")
        self.assertEqual(evaluate(make_request(), cap, make_context()).reason, "CAPABILITY_SUBJECT_MISMATCH")

    def test_wrong_resource_denied(self):
        cap = make_capability(resource_id="file:other")
        self.assertEqual(evaluate(make_request(), cap, make_context()).reason, "CAPABILITY_RESOURCE_MISMATCH")

    def test_wrong_operation_denied(self):
        req = make_request(operation="write", risk_claim="R2")
        self.assertEqual(evaluate(req, make_capability(), make_context()).reason, "CAPABILITY_OPERATION_MISMATCH")

    def test_wrong_purpose_denied(self):
        req = make_request(purpose="training")
        self.assertEqual(evaluate(req, make_capability(), make_context()).reason, "CAPABILITY_PURPOSE_MISMATCH")

    def test_expired_capability_denied(self):
        cap = make_capability(expires_at=20)
        self.assertEqual(evaluate(make_request(), cap, make_context()).reason, "CAPABILITY_NOT_CURRENT")

    def test_capability_not_yet_valid_denied(self):
        cap = make_capability(not_before=30)
        self.assertEqual(evaluate(make_request(), cap, make_context()).reason, "CAPABILITY_NOT_CURRENT")

    def test_revoked_capability_denied(self):
        ctx = make_context(revoked_capabilities=frozenset({"cap-00000001"}))
        self.assertEqual(evaluate(make_request(), make_capability(), ctx).reason, "CAPABILITY_REVOKED")

    def test_a_registry_from_another_epoch_is_not_the_policy_in_force(self):
        # A registry carries the policy generation it was published for. If
        # the node is running a different generation, the risk tiers it
        # consults come from policy that is not the policy the request was
        # bound to, and the request's own epoch check cannot see it: the
        # request and the node agree about the epoch while the registry
        # disagrees with both.
        decision = evaluate(
            make_request(), make_capability(), make_context(registry_epoch=6)
        )
        self.assertEqual(
            decision.reason,
            "REGISTRY_EPOCH_MISMATCH",
            "a registry from another epoch is not the policy in force, "
            "and the reason must say so",
        )
        self.assertEqual(decision.status, Status.DENY)

    def test_the_registry_for_the_epoch_in_force_is_the_policy_in_force(self):
        # The other half of the rule, and the one that keeps the check
        # honest: a registry that belongs to the epoch in force decides
        # normally.
        decision = evaluate(make_request(), make_capability(), make_context())
        self.assertEqual(decision.status, Status.PERMIT, decision.reason)

    def test_policy_epoch_mismatch_denied(self):
        req = make_request(policy_epoch=6)
        self.assertEqual(
            evaluate(req, make_capability(), make_context()).reason, "POLICY_EPOCH_MISMATCH"
        )

    def test_capability_policy_epoch_mismatch_denied(self):
        cap = make_capability(policy_epoch=6)
        self.assertEqual(
            evaluate(make_request(), cap, make_context()).reason, "CAPABILITY_POLICY_EPOCH_MISMATCH"
        )

    def test_replayed_nonce_denied(self):
        ctx = make_context(revoked_nonces=frozenset({"nonce-unique-0001"}))
        self.assertEqual(evaluate(make_request(), make_capability(), ctx).reason, "REPLAY_OR_REVOKED_NONCE")

    def test_expired_request_denied(self):
        req = make_request(expires_at=20)
        self.assertEqual(evaluate(req, make_capability(), make_context()).reason, "REQUEST_NOT_FRESH")

    def test_request_from_the_future_denied(self):
        req = make_request(issued_at=21)
        self.assertEqual(evaluate(req, make_capability(), make_context()).reason, "REQUEST_NOT_FRESH")

    def test_unresolvable_resource_denied(self):
        req = make_request(resource_id="file:missing")
        self.assertEqual(evaluate(req, make_capability(), make_context()).reason, "RESOURCE_UNRESOLVABLE")

    def test_read_only_mode_blocks_write(self):
        req = make_request(operation="write", risk_claim="R2")
        cap = make_capability(operations=frozenset({"write"}))
        ctx = make_context(mode="READ_ONLY")
        self.assertEqual(evaluate(req, cap, ctx).reason, "OPERATION_BLOCKED_BY_SYSTEM_MODE")

    def test_r3_requires_action_bound_approval(self):
        req = make_request(
            operation="delete",
            risk_claim="R3",
            approval_digest="abc",
        )
        cap = make_capability(operations=frozenset({"delete"}))
        decision = evaluate(req, cap, make_context(action_digest="different"))
        self.assertEqual(decision.status, Status.CHALLENGE)
        self.assertEqual(decision.reason, "FRESH_ACTION_BOUND_APPROVAL_REQUIRED")

    def test_r3_requires_strong_approval(self):
        req = make_request(operation="delete", risk_claim="R3", approval_digest="abc")
        cap = make_capability(operations=frozenset({"delete"}))
        ctx = make_context(action_digest="abc")
        decision = evaluate(req, cap, ctx)
        self.assertEqual(decision.status, Status.CHALLENGE)
        self.assertEqual(decision.reason, "STRONG_APPROVAL_REQUIRED")

    def test_caller_cannot_downgrade_delete_risk(self):
        req = make_request(operation="delete", risk_claim="R0")
        cap = make_capability(operations=frozenset({"delete"}))
        self.assertEqual(evaluate(req, cap, make_context()).reason, "RISK_CLAIM_MISMATCH")

    def test_caller_cannot_escalate_by_over_claiming(self):
        req = make_request(operation="write", risk_claim="R3", approval_digest="abc")
        cap = make_capability(operations=frozenset({"write"}))
        ctx = make_context(action_digest="abc")
        decision = evaluate(req, cap, ctx)
        self.assertEqual(decision.status, Status.PERMIT)

    def test_sensitive_action_denied_if_audit_unavailable(self):
        req = make_request(operation="write", risk_claim="R2", approval_digest="abc")
        cap = make_capability(operations=frozenset({"write"}))
        ctx = make_context(audit_state=AuditState.UNAVAILABLE, action_digest="abc")
        self.assertEqual(
            evaluate(req, cap, ctx).reason, "AUDIT_UNAVAILABLE_FOR_SENSITIVE_ACTION"
        )

    def test_standard_audit_class_is_not_blocked_by_an_unavailable_sink(self):
        ctx = make_context(audit_state=AuditState.UNAVAILABLE)
        decision = evaluate(make_request(), make_capability(), ctx)
        self.assertEqual(decision.status, Status.PERMIT)

    def test_unknown_operation_risk_is_denied(self):
        req = make_request(operation="mystery.execute")
        cap = make_capability(operations=frozenset({"mystery.execute"}))
        self.assertEqual(evaluate(req, cap, make_context()).reason, "UNKNOWN_OPERATION_RISK")

    def test_budget_over_the_registry_limit_is_denied(self):
        req = make_request(
            operation="write",
            risk_claim="R2",
            approval_digest="abc",
            budget=Budget(duration_ms=60001),
        )
        cap = make_capability(operations=frozenset({"write"}))
        ctx = make_context(action_digest="abc")
        self.assertEqual(evaluate(req, cap, ctx).reason, "RESOURCE_BUDGET_EXCEEDED")

    def test_budget_within_the_registry_limit_is_permitted(self):
        req = make_request(
            operation="write",
            risk_claim="R2",
            approval_digest="abc",
            budget=Budget(duration_ms=60000, operation_count=1),
        )
        cap = make_capability(operations=frozenset({"write"}))
        ctx = make_context(action_digest="abc")
        self.assertEqual(evaluate(req, cap, ctx).status, Status.PERMIT)

    def test_unsatisfied_precondition_is_denied(self):
        req = make_request(operation="write", risk_claim="R2", approval_digest="abc")
        cap = make_capability(operations=frozenset({"write"}))
        ctx = make_context(
            action_digest="abc",
            satisfied_preconditions=frozenset({"resource_exists"}),
        )
        self.assertEqual(evaluate(req, cap, ctx).reason, "PRECONDITION_FAILED")

    def test_unverified_issuer_is_denied(self):
        cap = make_capability(provenance=Provenance.UNCHECKED)
        self.assertEqual(evaluate(make_request(), cap, make_context()).reason, "UNVERIFIED_CAPABILITY_ISSUER")

    def test_invalid_signature_is_denied(self):
        cap = make_capability(provenance=Provenance.INVALID)
        self.assertEqual(evaluate(make_request(), cap, make_context()).reason, "SIGNATURE_INVALID")

    def test_evaluator_never_performs_effect(self):
        decision = evaluate(make_request(), make_capability(), make_context())
        self.assertFalse(decision.effect_performed)

    def test_effect_performed_cannot_be_set(self):
        # The property, not a field: no caller can construct a decision that
        # claims an effect happened.
        with self.assertRaises(TypeError):
            Decision = type(evaluate(make_request(), make_capability(), make_context()))
            Decision(Status.PERMIT, ALL_GATES_PASSED, None, 7, True)

    def test_attenuation_accepts_narrower_child(self):
        child = make_capability(not_before=10, expires_at=40, delegation_depth=2)
        self.assertTrue(attenuate(make_capability(), child))

    def test_attenuation_rejects_broader_operation(self):
        child = make_capability(operations=frozenset({"read", "write"}), delegation_depth=2)
        self.assertFalse(attenuate(make_capability(), child))

    def test_attenuation_rejects_longer_lifetime(self):
        child = make_capability(expires_at=70, delegation_depth=2)
        self.assertFalse(attenuate(make_capability(), child))

    def test_attenuation_rejects_more_uses(self):
        parent = make_capability(max_uses=5)
        child = make_capability(max_uses=6, delegation_depth=2)
        self.assertFalse(attenuate(parent, child))

    def test_attenuation_rejects_a_different_subject(self):
        child = make_capability(subject_id="user:bob", delegation_depth=2)
        self.assertFalse(attenuate(make_capability(), child))


if __name__ == "__main__":
    unittest.main()
