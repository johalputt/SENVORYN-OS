"""Senvoryn SCP reference decision model (prototype only).

This module is the specification's *reference policy evaluator*: it implements
the sixteen validation gates of Core Contract Specification §3.2 in the order
the specification gives them, and it emits only reason codes that exist in the
denial-code registry.

It is a PROTOTYPE. It does not verify cryptographic signatures, canonical byte
encodings beyond JSON well-formedness, operating-system identities, clocks, or
hardware capabilities, and it performs no effects. Production code must provide
those boundaries and independent review.

The Rust implementation in ``crates/senvoryn-scp`` implements the same gates
against the same reason codes. The two are kept in step by the shared
conformance vectors in ``conformance/vectors/``, which both test suites
execute; a disagreement fails CI.

Two rules shape the types here, and both exist because the alternative has
already caused a real defect elsewhere in this project:

* Nothing that means "a security property was established" is a boolean with a
  safe-looking default. ``Provenance`` and ``AuditState`` are enums, and
  ``Capability`` has no way to be constructed without stating its provenance.
* Nothing that is always true is a settable field. ``Decision.effect_performed``
  is a property, not a field, so no caller can set it.
"""

from __future__ import annotations

import enum
from dataclasses import dataclass
from typing import FrozenSet, Mapping, Optional

#: The reason a permitted decision carries. Not a denial, so it is not in the
#: denial-code registry, but it is a reason code like any other and is
#: reported in the decision record's ``reason_codes`` array.
ALL_GATES_PASSED = "ALL_POLICY_GATES_PASSED"

#: Risk tiers, in ascending order of consequence.
RISK_TIERS = ("R0", "R1", "R2", "R3")

#: Audit classes, in ascending order of required assurance.
AUDIT_CLASSES = ("STANDARD", "SENSITIVE", "SECURITY_CRITICAL")

#: System modes.
SYSTEM_MODES = (
    "NORMAL",
    "DEGRADED",
    "READ_ONLY",
    "RECOVERY",
    "QUARANTINED",
    "MAINTENANCE",
    "OFFLINE",
)

#: Every reason code this model can produce. The denial-code registry is the
#: authority on which codes exist; ``tests/test_scp_reference.py`` checks this
#: list against it in both directions, and the Rust suite checks its own list
#: against the same registry.
ALL_REASON_CODES = frozenset(
    {
        ALL_GATES_PASSED,
        "MALFORMED_OR_MISSING_IDENTITY",
        "WRONG_RECIPIENT",
        "UNKNOWN_PRINCIPAL",
        "REQUEST_NOT_FRESH",
        "REPLAY_OR_REVOKED_NONCE",
        "RESOURCE_UNRESOLVABLE",
        "UNKNOWN_OPERATION_RISK",
        "RISK_CLAIM_MISMATCH",
        "POLICY_EPOCH_MISMATCH",
        "MISSING_CAPABILITY",
        "UNVERIFIED_CAPABILITY_ISSUER",
        "CAPABILITY_REVOKED",
        "CAPABILITY_SUBJECT_MISMATCH",
        "CAPABILITY_RESOURCE_MISMATCH",
        "CAPABILITY_OPERATION_MISMATCH",
        "CAPABILITY_PURPOSE_MISMATCH",
        "CAPABILITY_NOT_CURRENT",
        "CAPABILITY_POLICY_EPOCH_MISMATCH",
        "OPERATION_BLOCKED_BY_SYSTEM_MODE",
        "FRESH_ACTION_BOUND_APPROVAL_REQUIRED",
        "STRONG_APPROVAL_REQUIRED",
        "AUDIT_UNAVAILABLE_FOR_SENSITIVE_ACTION",
        "RESOURCE_BUDGET_EXCEEDED",
        "PRECONDITION_FAILED",
        "UNSUPPORTED_CRITICAL_SCHEMA",
        "SIGNATURE_INVALID",
        "ANTI_ROLLBACK_VIOLATION",
    }
)


class Status(str, enum.Enum):
    """The decision vocabulary of specification §3.3."""

    PERMIT = "PERMIT"
    DENY = "DENY"
    CHALLENGE = "CHALLENGE"
    DEFER = "DEFER"
    QUARANTINE = "QUARANTINE"


class Provenance(str, enum.Enum):
    """How the issuer's authority was established.

    Not a boolean. A boolean defaulting to ``True`` lets a capability that
    nobody verified be treated as verified by omission, which is the failure
    this enum exists to make impossible. A capability cannot be constructed
    without stating which of these it has.
    """

    VERIFIED = "VERIFIED"
    UNCHECKED = "UNCHECKED"
    INVALID = "INVALID"


class AuditState(str, enum.Enum):
    """Whether the audit sink can record the required audit class."""

    READY = "READY"
    UNAVAILABLE = "UNAVAILABLE"


@dataclass(frozen=True)
class Budget:
    """The budget a request asks for. Absent fields are unbounded by the request."""

    duration_ms: Optional[int] = None
    operation_count: Optional[int] = None


@dataclass(frozen=True)
class Request:
    """One request envelope, in the shape the policy layer sees it.

    Times are monotonic milliseconds since boot. The wire format carries
    date-times; converting them to monotonic values is the transport layer's
    job, and the policy layer compares against ``PolicyContext.now_monotonic``
    so that a wall-clock change cannot extend or cut short a capability's life.
    """

    request_id: str
    principal_id: str
    recipient_id: str
    resource_id: str
    resource_type: str
    operation: str
    purpose: str
    policy_epoch: int
    issued_at: int
    expires_at: int
    nonce: str
    risk_claim: str
    approval_digest: Optional[str] = None
    budget: Optional[Budget] = None


@dataclass(frozen=True)
class Capability:
    """A capability grant.

    ``provenance`` has no default. A capability that nobody checked cannot be
    built without saying so.
    """

    capability_id: str
    subject_id: str
    resource_id: str
    resource_type: str
    operations: FrozenSet[str]
    purposes: FrozenSet[str]
    not_before: int
    expires_at: int
    policy_epoch: int
    provenance: Provenance
    delegation_depth: int = 0
    max_uses: Optional[int] = None


@dataclass(frozen=True)
class OperationEntry:
    """One entry of the trusted Operation Registry."""

    operation_id: str
    risk_tier: str
    approval_requirements: FrozenSet[str]
    allowed_modes: FrozenSet[str]
    resource_types: FrozenSet[str]
    max_duration_ms: Optional[int] = None
    max_operation_count: Optional[int] = None
    required_preconditions: FrozenSet[str] = frozenset()
    audit_class: str = "STANDARD"


@dataclass(frozen=True)
class PolicyContext:
    """The state the policy environment provides.

    Every field here is something the environment *knows*, never something the
    caller asserts. ``audit_state`` in particular has no default: a context
    that never says whether the audit sink is up must not be able to imply
    that it is.

    ``operations`` is a sequence rather than a mapping because the registry
    may hold more than one rule for the same operation, and the strictest
    rule is the one that applies.
    """

    local_node_id: str
    current_policy_epoch: int
    now_monotonic: int
    mode: str
    known_principals: FrozenSet[str]
    resolved_resources: Mapping[str, str]
    operations: tuple[OperationEntry, ...]
    audit_state: AuditState
    action_digest: Optional[str] = None
    revoked_nonces: FrozenSet[str] = frozenset()
    revoked_capabilities: FrozenSet[str] = frozenset()
    strong_approval_granted: bool = False
    satisfied_preconditions: FrozenSet[str] = frozenset()


@dataclass(frozen=True)
class Decision:
    """The result of evaluating one request."""

    status: Status
    reason: str
    capability_id: Optional[str] = None
    policy_epoch: int = 0

    @property
    def effect_performed(self) -> bool:
        """Always false.

        A property rather than a field, so that no caller can set it. The
        effect separation gate is the sixteenth of the sixteen, and the whole
        point of it is that the evaluator cannot perform an effect.
        """
        return False


def _deny(reason: str, policy_epoch: int = 0) -> Decision:
    return Decision(Status.DENY, reason, None, policy_epoch)


def _challenge(reason: str, policy_epoch: int = 0) -> Decision:
    return Decision(Status.CHALLENGE, reason, None, policy_epoch)


def _risk_rank(tier: str) -> int:
    return RISK_TIERS.index(tier)


def _audit_rank(audit_class: str) -> int:
    return AUDIT_CLASSES.index(audit_class)


def _merge_entries(entries: list[OperationEntry]) -> OperationEntry:
    """Fold every rule that applies to one operation into the strictest one.

    "When multiple rules apply, the most restrictive risk tier wins" —
    specification §3.2. The same principle is applied to the other dimensions:
    the narrowest mode set, the union of approval requirements, the smallest
    budgets, and the highest audit class. A caller therefore cannot escape a
    strict rule by being matched by a lax one.
    """
    first = entries[0]
    risk = max(entries, key=lambda entry: _risk_rank(entry.risk_tier))
    audit = max(entries, key=lambda entry: _audit_rank(entry.audit_class))
    allowed_modes = set(first.allowed_modes)
    resource_types: set[str] = set()
    approval_requirements: set[str] = set()
    preconditions: set[str] = set()
    for entry in entries:
        allowed_modes &= set(entry.allowed_modes)
        resource_types |= set(entry.resource_types)
        approval_requirements |= set(entry.approval_requirements)
        preconditions |= set(entry.required_preconditions)
    durations = [entry.max_duration_ms for entry in entries if entry.max_duration_ms is not None]
    counts = [entry.max_operation_count for entry in entries if entry.max_operation_count is not None]
    return OperationEntry(
        operation_id=first.operation_id,
        risk_tier=risk.risk_tier,
        approval_requirements=frozenset(approval_requirements),
        allowed_modes=frozenset(allowed_modes),
        resource_types=frozenset(resource_types),
        max_duration_ms=min(durations) if durations else None,
        max_operation_count=min(counts) if counts else None,
        required_preconditions=frozenset(preconditions),
        audit_class=audit.audit_class,
    )


def evaluate(req: Request, cap: Optional[Capability], ctx: PolicyContext) -> Decision:
    """Evaluate one request against the policy context.

    This is a pure function. It reads no clock, performs no I/O, mutates
    nothing, and returns a decision that grants nothing by itself. The gates
    are evaluated in the order specification §3.2 gives them, and the first
    one that fails is the reason reported.
    """
    epoch = ctx.current_policy_epoch

    # Gate 2 - canonical parse. The structural half: every field a later gate
    # relies on must be present and well-typed before any of them is read.
    if not (
        req.request_id
        and req.principal_id
        and req.recipient_id
        and req.resource_id
        and req.resource_type
        and req.operation
        and req.purpose
        and req.nonce
    ):
        return _deny("MALFORMED_OR_MISSING_IDENTITY", epoch)
    if req.risk_claim not in RISK_TIERS:
        return _deny("MALFORMED_OR_MISSING_IDENTITY", epoch)

    # Gate 3 - recipient. Checked before identity so that a request addressed
    # elsewhere does not learn whether the principal exists.
    if req.recipient_id != ctx.local_node_id:
        return _deny("WRONG_RECIPIENT", epoch)

    # Gate 4 - identity.
    if req.principal_id not in ctx.known_principals:
        return _deny("UNKNOWN_PRINCIPAL", epoch)

    # Gate 5 - freshness: nonce, replay cache, expiry.
    if req.nonce in ctx.revoked_nonces:
        return _deny("REPLAY_OR_REVOKED_NONCE", epoch)
    if req.issued_at > ctx.now_monotonic or req.expires_at <= ctx.now_monotonic:
        return _deny("REQUEST_NOT_FRESH", epoch)

    # Gate 6 - object resolution. The resource must resolve to a canonical
    # object; an identifier that resolves to nothing is denied rather than
    # dereferenced.
    if req.resource_id not in ctx.resolved_resources:
        return _deny("RESOURCE_UNRESOLVABLE", epoch)

    # Gate 7 - revocation. Principal revocation is expressed by removing the
    # principal from `known_principals`; capability revocation is listed here.
    if cap is not None and cap.capability_id in ctx.revoked_capabilities:
        return _deny("CAPABILITY_REVOKED", epoch)

    # Gate 8 - mode. The registry declares the modes each operation is
    # designed for, and the registry is the signed policy, so it is the only
    # authority consulted. An operation the registry does not know is left to
    # gate 11, which is where the unknown operation is reported.
    matching = [entry for entry in ctx.operations if entry.operation_id == req.operation]
    entry = _merge_entries(matching) if matching else None
    if entry is not None and ctx.mode not in entry.allowed_modes:
        return _deny("OPERATION_BLOCKED_BY_SYSTEM_MODE", epoch)

    # Gate 9 - policy epoch.
    if req.policy_epoch != ctx.current_policy_epoch:
        return _deny("POLICY_EPOCH_MISMATCH", epoch)

    # Gate 10 - capability.
    if cap is None:
        return _deny("MISSING_CAPABILITY", epoch)
    if cap.provenance is Provenance.INVALID:
        return _deny("SIGNATURE_INVALID", epoch)
    if cap.provenance is not Provenance.VERIFIED:
        return _deny("UNVERIFIED_CAPABILITY_ISSUER", epoch)
    if cap.subject_id != req.principal_id:
        return _deny("CAPABILITY_SUBJECT_MISMATCH", epoch)
    if cap.resource_id != req.resource_id or cap.resource_type != req.resource_type:
        return _deny("CAPABILITY_RESOURCE_MISMATCH", epoch)
    if req.operation not in cap.operations:
        return _deny("CAPABILITY_OPERATION_MISMATCH", epoch)
    if req.purpose not in cap.purposes:
        return _deny("CAPABILITY_PURPOSE_MISMATCH", epoch)
    if ctx.now_monotonic < cap.not_before or ctx.now_monotonic >= cap.expires_at:
        return _deny("CAPABILITY_NOT_CURRENT", epoch)
    if cap.policy_epoch != ctx.current_policy_epoch:
        return _deny("CAPABILITY_POLICY_EPOCH_MISMATCH", epoch)

    # Gate 11 - risk and consent. The tier comes from the registry. The
    # caller's claim is never authoritative: a lower claim is a mismatch, and
    # a higher claim is ignored in favour of the registry's tier.
    if entry is None:
        return _deny("UNKNOWN_OPERATION_RISK", epoch)
    if req.resource_type not in entry.resource_types:
        return _deny("UNKNOWN_OPERATION_RISK", epoch)
    if _risk_rank(req.risk_claim) < _risk_rank(entry.risk_tier):
        return _deny("RISK_CLAIM_MISMATCH", epoch)
    if entry.approval_requirements and entry.approval_requirements != frozenset({"NONE"}):
        if not req.approval_digest or req.approval_digest != ctx.action_digest:
            return _challenge("FRESH_ACTION_BOUND_APPROVAL_REQUIRED", epoch)
        if "STRONG_AUTH" in entry.approval_requirements and not ctx.strong_approval_granted:
            return _challenge("STRONG_APPROVAL_REQUIRED", epoch)

    # Gate 12 - budget. The registry states the limits; where it states none,
    # there is no policy to check against, and the gate does not invent one.
    if req.budget is not None:
        if entry.max_duration_ms is not None and req.budget.duration_ms is not None:
            if req.budget.duration_ms > entry.max_duration_ms:
                return _deny("RESOURCE_BUDGET_EXCEEDED", epoch)
        if entry.max_operation_count is not None and req.budget.operation_count is not None:
            if req.budget.operation_count > entry.max_operation_count:
                return _deny("RESOURCE_BUDGET_EXCEEDED", epoch)

    # Gate 13 - precondition. The registry states the preconditions the
    # operation needs; the environment reports which currently hold.
    for precondition in entry.required_preconditions:
        if precondition not in ctx.satisfied_preconditions:
            return _deny("PRECONDITION_FAILED", epoch)

    # Gate 14 - audit readiness. Absent audit storage is not absent risk.
    if _audit_rank(entry.audit_class) >= _audit_rank("SENSITIVE"):
        if ctx.audit_state is not AuditState.READY:
            return _deny("AUDIT_UNAVAILABLE_FOR_SENSITIVE_ACTION", epoch)

    # Gate 15 - decision construction. Gate 16 - effect separation: nothing
    # here performs an effect, and the decision says so.
    return Decision(Status.PERMIT, ALL_GATES_PASSED, cap.capability_id, epoch)


def attenuate(parent: Capability, child: Capability) -> bool:
    """Return True only when the child is equal to or narrower than the parent.

    Specification §4.2. The delegation-depth rule follows the specification's
    wording literally - "child delegation depth must decrease" - even though
    the capability schema's ``delegation_depth`` also reads as a chain depth,
    which would make it increase. The two readings cannot both hold, and the
    specification is the normative text, so the ambiguity is recorded as an
    open decision rather than resolved silently here.
    """
    return (
        child.subject_id == parent.subject_id
        and child.resource_id == parent.resource_id
        and child.resource_type == parent.resource_type
        and child.operations <= parent.operations
        and child.purposes <= parent.purposes
        and child.not_before >= parent.not_before
        and child.expires_at <= parent.expires_at
        and child.policy_epoch == parent.policy_epoch
        and child.delegation_depth < parent.delegation_depth
        and (
            parent.max_uses is None
            or (child.max_uses is not None and child.max_uses <= parent.max_uses)
        )
    )
