"""Senvoryn SCP reference decision model (prototype only).

This module demonstrates deterministic policy ordering for tests. It does NOT
verify cryptographic signatures, canonical byte encodings, operating-system
identities, clocks, hardware capabilities, or execute effects. Production code
must provide those boundaries and independent review.
"""
from dataclasses import dataclass, field
from enum import Enum
from typing import FrozenSet, Optional

class Status(str, Enum):
    PERMIT = "PERMIT"
    DENY = "DENY"
    CHALLENGE = "CHALLENGE"
    DEFER = "DEFER"
    QUARANTINE = "QUARANTINE"

@dataclass(frozen=True)
class Request:
    request_id: str
    principal_id: str
    recipient_id: str
    resource_id: str
    operation: str
    purpose: str
    policy_epoch: int
    issued_at: int
    expires_at: int
    nonce: str
    risk_tier: str
    required_approval_digest: Optional[str] = None
    provided_approval_digest: Optional[str] = None
    audit_ready: bool = True
    mode: str = "NORMAL"

@dataclass(frozen=True)
class Capability:
    capability_id: str
    subject_id: str
    resource_id: str
    operations: FrozenSet[str]
    purposes: FrozenSet[str]
    not_before: int
    expires_at: int
    policy_epoch: int
    revoked: bool = False
    issuer_verified: bool = True
    max_uses: Optional[int] = None
    delegation_depth: int = 0

@dataclass(frozen=True)
class PolicyContext:
    local_node_id: str
    current_policy_epoch: int
    now_monotonic: int
    known_principals: FrozenSet[str]
    revoked_nonces: FrozenSet[str] = frozenset()
    read_only_operations: FrozenSet[str] = frozenset({"read", "inspect", "status"})
    allowed_in_recovery: FrozenSet[str] = frozenset({"inspect", "recover", "verify"})
    approved_risk3: bool = False
    audit_ready: bool = True
    # Trusted local registry: operation name -> minimum canonical risk tier.
    operation_risk_tiers: tuple = (("read", "R0"), ("inspect", "R0"), ("status", "R0"), ("write", "R2"), ("delete", "R3"))

@dataclass(frozen=True)
class Decision:
    status: Status
    reason: str
    effect_performed: bool = False
    capability_id: Optional[str] = None


def evaluate(req: Request, cap: Optional[Capability], ctx: PolicyContext) -> Decision:
    """Evaluate a single request. This pure function has no I/O or side effects."""
    def deny(reason: str) -> Decision:
        return Decision(Status.DENY, reason, False, None)
    if not req.request_id or not req.nonce or not req.principal_id:
        return deny("MALFORMED_OR_MISSING_IDENTITY")
    if req.recipient_id != ctx.local_node_id:
        return deny("WRONG_RECIPIENT")
    if req.principal_id not in ctx.known_principals:
        return deny("UNKNOWN_PRINCIPAL")
    if req.nonce in ctx.revoked_nonces:
        return deny("REPLAY_OR_REVOKED_NONCE")
    if req.expires_at <= ctx.now_monotonic or req.issued_at > ctx.now_monotonic:
        return deny("REQUEST_NOT_FRESH")
    if req.policy_epoch != ctx.current_policy_epoch:
        return deny("POLICY_EPOCH_MISMATCH")
    risk_map = dict(ctx.operation_risk_tiers)
    canonical_risk = risk_map.get(req.operation)
    if canonical_risk is None:
        return deny("UNKNOWN_OPERATION_RISK")
    if req.mode in {"READ_ONLY", "RECOVERY", "QUARANTINED"}:
        allowed = ctx.allowed_in_recovery if req.mode == "RECOVERY" else ctx.read_only_operations
        if req.operation not in allowed:
            return deny("OPERATION_BLOCKED_BY_SYSTEM_MODE")
    if cap is None:
        return deny("MISSING_CAPABILITY")
    if not cap.issuer_verified:
        return deny("UNVERIFIED_CAPABILITY_ISSUER")
    if cap.revoked:
        return deny("CAPABILITY_REVOKED")
    if cap.subject_id != req.principal_id:
        return deny("CAPABILITY_SUBJECT_MISMATCH")
    if cap.resource_id != req.resource_id:
        return deny("CAPABILITY_RESOURCE_MISMATCH")
    if req.operation not in cap.operations:
        return deny("CAPABILITY_OPERATION_MISMATCH")
    if req.purpose not in cap.purposes:
        return deny("CAPABILITY_PURPOSE_MISMATCH")
    if ctx.now_monotonic < cap.not_before or ctx.now_monotonic >= cap.expires_at:
        return deny("CAPABILITY_NOT_CURRENT")
    if cap.policy_epoch != ctx.current_policy_epoch:
        return deny("CAPABILITY_POLICY_EPOCH_MISMATCH")
    if req.risk_tier != canonical_risk:
        return deny("RISK_CLAIM_MISMATCH")
    if canonical_risk == "R3":
        if not req.required_approval_digest or req.required_approval_digest != req.provided_approval_digest:
            return Decision(Status.CHALLENGE, "FRESH_ACTION_BOUND_APPROVAL_REQUIRED", False, None)
        if not ctx.approved_risk3:
            return Decision(Status.CHALLENGE, "STRONG_APPROVAL_REQUIRED", False, None)
    if canonical_risk not in {"R0", "R1", "R2", "R3"}:
        return deny("UNKNOWN_RISK_TIER")
    if not ctx.audit_ready and canonical_risk in {"R2", "R3"}:
        return deny("AUDIT_UNAVAILABLE_FOR_SENSITIVE_ACTION")
    return Decision(Status.PERMIT, "ALL_POLICY_GATES_PASSED", False, cap.capability_id)


def attenuate(parent: Capability, child: Capability) -> bool:
    """Return True only for an obviously non-broadening child in this toy model."""
    return (
        child.subject_id == parent.subject_id
        and child.resource_id == parent.resource_id
        and child.operations.issubset(parent.operations)
        and child.purposes.issubset(parent.purposes)
        and child.not_before >= parent.not_before
        and child.expires_at <= parent.expires_at
        and child.policy_epoch == parent.policy_epoch
        and child.delegation_depth < parent.delegation_depth
        and (parent.max_uses is None or (child.max_uses is not None and child.max_uses <= parent.max_uses))
    )
