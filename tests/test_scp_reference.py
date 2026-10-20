import unittest
from scp_reference import Request, Capability, PolicyContext, Status, evaluate, attenuate

class SCPReferenceTests(unittest.TestCase):
    def setUp(self):
        self.req = Request("req-0000000001", "user:alice", "node:local", "file:report", "read", "user-request", 7, 10, 50, "nonce-unique-0001", "R0")
        self.cap = Capability("cap-00000001", "user:alice", "file:report", frozenset({"read"}), frozenset({"user-request"}), 1, 60, 7, delegation_depth=3)
        self.ctx = PolicyContext("node:local", 7, 20, frozenset({"user:alice"}))
    def test_valid_scoped_request_permits_without_effect(self):
        d = evaluate(self.req, self.cap, self.ctx)
        self.assertEqual(d.status, Status.PERMIT)
        self.assertFalse(d.effect_performed)
    def test_unknown_principal_denied(self):
        ctx = PolicyContext("node:local", 7, 20, frozenset())
        self.assertEqual(evaluate(self.req, self.cap, ctx).reason, "UNKNOWN_PRINCIPAL")
    def test_missing_capability_denied(self):
        self.assertEqual(evaluate(self.req, None, self.ctx).reason, "MISSING_CAPABILITY")
    def test_wrong_recipient_denied(self):
        req = Request(**{**self.req.__dict__, "recipient_id":"node:other"})
        self.assertEqual(evaluate(req, self.cap, self.ctx).reason, "WRONG_RECIPIENT")
    def test_wrong_subject_denied(self):
        cap = Capability(**{**self.cap.__dict__, "subject_id":"user:bob"})
        self.assertEqual(evaluate(self.req, cap, self.ctx).reason, "CAPABILITY_SUBJECT_MISMATCH")
    def test_wrong_resource_denied(self):
        cap = Capability(**{**self.cap.__dict__, "resource_id":"file:other"})
        self.assertEqual(evaluate(self.req, cap, self.ctx).reason, "CAPABILITY_RESOURCE_MISMATCH")
    def test_wrong_operation_denied(self):
        req = Request(**{**self.req.__dict__, "operation":"write"})
        self.assertEqual(evaluate(req, self.cap, self.ctx).reason, "CAPABILITY_OPERATION_MISMATCH")
    def test_wrong_purpose_denied(self):
        req = Request(**{**self.req.__dict__, "purpose":"training"})
        self.assertEqual(evaluate(req, self.cap, self.ctx).reason, "CAPABILITY_PURPOSE_MISMATCH")
    def test_expired_capability_denied(self):
        cap = Capability(**{**self.cap.__dict__, "expires_at":20})
        self.assertEqual(evaluate(self.req, cap, self.ctx).reason, "CAPABILITY_NOT_CURRENT")
    def test_revoked_capability_denied(self):
        cap = Capability(**{**self.cap.__dict__, "revoked":True})
        self.assertEqual(evaluate(self.req, cap, self.ctx).reason, "CAPABILITY_REVOKED")
    def test_policy_epoch_mismatch_denied(self):
        req = Request(**{**self.req.__dict__, "policy_epoch":6})
        self.assertEqual(evaluate(req, self.cap, self.ctx).reason, "POLICY_EPOCH_MISMATCH")
    def test_replayed_nonce_denied(self):
        ctx = PolicyContext("node:local", 7, 20, frozenset({"user:alice"}), frozenset({"nonce-unique-0001"}))
        self.assertEqual(evaluate(self.req, self.cap, ctx).reason, "REPLAY_OR_REVOKED_NONCE")
    def test_read_only_mode_blocks_write(self):
        req = Request(**{**self.req.__dict__, "operation":"write", "mode":"READ_ONLY"})
        cap = Capability(**{**self.cap.__dict__, "operations":frozenset({"write"})})
        self.assertEqual(evaluate(req, cap, self.ctx).reason, "OPERATION_BLOCKED_BY_SYSTEM_MODE")
    def test_r3_requires_action_bound_approval(self):
        req = Request(**{**self.req.__dict__, "operation":"delete", "risk_tier":"R3", "required_approval_digest":"abc", "provided_approval_digest":"different"})
        cap = Capability(**{**self.cap.__dict__, "operations":frozenset({"delete"})})
        self.assertEqual(evaluate(req, cap, self.ctx).status, Status.CHALLENGE)
    def test_caller_cannot_downgrade_delete_risk(self):
        req = Request(**{**self.req.__dict__, "operation":"delete", "risk_tier":"R0"})
        cap = Capability(**{**self.cap.__dict__, "operations":frozenset({"delete"})})
        self.assertEqual(evaluate(req, cap, self.ctx).reason, "RISK_CLAIM_MISMATCH")
    def test_sensitive_action_denied_if_audit_unavailable(self):
        req = Request(**{**self.req.__dict__, "operation":"write", "risk_tier":"R2"})
        cap = Capability(**{**self.cap.__dict__, "operations":frozenset({"write"})})
        ctx = PolicyContext("node:local", 7, 20, frozenset({"user:alice"}), audit_ready=False)
        self.assertEqual(evaluate(req, cap, ctx).reason, "AUDIT_UNAVAILABLE_FOR_SENSITIVE_ACTION")
    def test_unknown_operation_risk_is_denied(self):
        req = Request(**{**self.req.__dict__, "operation":"mystery.execute"})
        self.assertEqual(evaluate(req, self.cap, self.ctx).reason, "UNKNOWN_OPERATION_RISK")
    def test_evaluator_never_performs_effect(self):
        self.assertFalse(evaluate(self.req, self.cap, self.ctx).effect_performed)
    def test_attenuation_accepts_narrower_child(self):
        child = Capability("cap-child-01", "user:alice", "file:report", frozenset({"read"}), frozenset({"user-request"}), 10, 40, 7, delegation_depth=2)
        self.assertTrue(attenuate(self.cap, child))
    def test_attenuation_rejects_broader_operation(self):
        child = Capability("cap-child-02", "user:alice", "file:report", frozenset({"read","write"}), frozenset({"user-request"}), 10, 40, 7, delegation_depth=2)
        self.assertFalse(attenuate(self.cap, child))
    def test_attenuation_rejects_longer_lifetime(self):
        child = Capability("cap-child-03", "user:alice", "file:report", frozenset({"read"}), frozenset({"user-request"}), 10, 70, 7, delegation_depth=2)
        self.assertFalse(attenuate(self.cap, child))

if __name__ == "__main__":
    unittest.main()
