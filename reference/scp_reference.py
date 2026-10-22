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
from dataclasses import dataclass, replace
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

#: Reversibilities, in ascending order of how reversible the operation is.
#: ``UNKNOWN`` sorts last because an operation whose reversibility the
#: registry does not state is treated as the worst case, not as the best one.
REVERSIBILITIES = ("REVERSIBLE", "COMPENSATABLE", "NON_REVERSIBLE", "UNKNOWN")

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
        "PRINCIPAL_QUARANTINED",
        "REQUEST_NOT_FRESH",
        "REPLAY_OR_REVOKED_NONCE",
        "RESOURCE_UNRESOLVABLE",
        "RESOURCE_TYPE_MISMATCH",
        "UNKNOWN_OPERATION_RISK",
        "RISK_CLAIM_MISMATCH",
        "POLICY_EPOCH_MISMATCH",
        "POLICY_EPOCH_NOT_YET_IN_FORCE",
        "REGISTRY_EPOCH_MISMATCH",
        "CLOCK_UNTRUSTED",
        "REGISTRY_NOT_YET_VALID",
        "REGISTRY_EXPIRED",
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
    reversibility: str = "UNKNOWN"
    external_effects: FrozenSet[str] = frozenset()


@dataclass(frozen=True)
class ResolvedResource:
    """The canonical identity of a resource, as the environment resolved it.

    The environment resolves; this model does not. It has no filesystem and no
    resolver, and it is not the component that knows what objects exist. What
    arrives here is the result of that resolution: the identifier the rest of
    the system knows the object by, the type the object has, and the version
    it was at when it was resolved.

    The distinction the record draws is between the *name* a caller used and
    the *identity* the name means. A caller may address an object by an alias,
    and the alias is legitimate; what the decision path compares against is
    the canonical identity, because that is what a grant is over and what the
    registry's rules are keyed by.
    """

    canonical_id: str
    resource_type: str
    version: str


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
    now_wall_ms: int
    clock_state: str
    mode: str
    known_principals: FrozenSet[str]
    resolved_resources: Mapping[str, ResolvedResource]
    operations: tuple[OperationEntry, ...]
    registry_epoch: int
    registry_valid_from_ms: int
    registry_valid_until_ms: int
    audit_state: AuditState
    action_digest: Optional[str] = None
    revoked_nonces: FrozenSet[str] = frozenset()
    revoked_capabilities: FrozenSet[str] = frozenset()
    quarantined_principals: FrozenSet[str] = frozenset()
    strong_approval_granted: bool = False
    satisfied_preconditions: FrozenSet[str] = frozenset()


@dataclass(frozen=True)
class Deferral:
    """What a deferred decision is waiting for.

    A deferral is a decision that says "not yet", so it has to say what it is
    waiting for. A deferral that waits for nothing is a decision that never
    arrives, and the record would publish a next step that names no event.
    """

    policy_epoch: int

    def required_next_step(self) -> str:
        """The next step a caller must take, as the decision record publishes it."""
        return f"wait for policy epoch {self.policy_epoch} to come into force"


@dataclass(frozen=True)
class Isolation:
    """What a quarantined decision is holding apart.

    A quarantine is a state of the subject rather than a state of the
    request, so it names the subject: the decision says which principal is
    held, and the record publishes that, so that the subject can ask for the
    isolation to be lifted.

    The type is named for the state rather than for the status, because the
    status is ``QUARANTINE`` and a type and a function cannot both be called
    ``quarantine`` in the same module.
    """

    principal: str

    def required_next_step(self) -> str:
        """The next step a caller must take, as the decision record publishes it."""
        return (
            f"the principal {self.principal} is quarantined; "
            "a policy decision must lift the isolation"
        )


@dataclass(frozen=True)
class Decision:
    """The result of evaluating one request."""

    status: Status
    reason: str
    capability_id: Optional[str] = None
    policy_epoch: int = 0
    resolved: Optional[ResolvedResource] = None
    deferral: Optional[Deferral] = None
    isolation: Optional[Isolation] = None

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


def _defer(reason: str, deferral: Deferral, policy_epoch: int = 0) -> Decision:
    """A decision that waits for a policy window to open.

    The deferral names what it waits for, because a deferral that waits for
    nothing is a decision that never arrives. It grants no authority: the
    capability scope stays empty.
    """
    return Decision(Status.DEFER, reason, None, policy_epoch, None, deferral)


def _quarantine(reason: str, isolation: Isolation, policy_epoch: int = 0) -> Decision:
    """A decision that holds the principal apart.

    The isolation names the principal it holds, because a quarantine that
    names nobody is a penalty with no subject. It grants no authority: the
    capability scope stays empty.
    """
    return Decision(Status.QUARANTINE, reason, None, policy_epoch, None, None, isolation)


def _risk_rank(tier: str) -> int:
    return RISK_TIERS.index(tier)


def _audit_rank(audit_class: str) -> int:
    return AUDIT_CLASSES.index(audit_class)


def _reversibility_rank(reversibility: str) -> int:
    """The rank of a reversibility.

    A value this model does not know is ranked as ``UNKNOWN`` rather than
    raising: the reversibility comes from the registry, and an operation whose
    reversibility the registry does not state is the worst case, not an
    error.
    """
    if reversibility in REVERSIBILITIES:
        return REVERSIBILITIES.index(reversibility)
    return REVERSIBILITIES.index("UNKNOWN")


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
    reversibility = max(entries, key=lambda entry: _reversibility_rank(entry.reversibility))
    allowed_modes = set(first.allowed_modes)
    resource_types: set[str] = set()
    approval_requirements: set[str] = set()
    preconditions: set[str] = set()
    external_effects: set[str] = set()
    for entry in entries:
        allowed_modes &= set(entry.allowed_modes)
        resource_types |= set(entry.resource_types)
        approval_requirements |= set(entry.approval_requirements)
        preconditions |= set(entry.required_preconditions)
        external_effects |= set(entry.external_effects)
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
        reversibility=reversibility.reversibility,
        external_effects=frozenset(external_effects),
    )


def _is_non_destructive(entry: Optional[OperationEntry]) -> bool:
    """Whether an operation may be deferred rather than denied.

    Specification §3.3 qualifies ``DEFER`` with "operation is non-destructive",
    and a deferral is a promise of a later answer — so it is a promise this
    project may only make about an operation whose effect can be undone.

    The registry in force is the only policy this node holds, so the registry
    in force is what answers. An operation it does not know is not shown to be
    non-destructive, and an external effect settles it the other way: an
    operation that reaches outside the node is not undone by restoring local
    state, whatever the registry says about reversibility.
    """
    if entry is None:
        return False
    return entry.reversibility == "REVERSIBLE" and not entry.external_effects


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

    # The identity gate, continued - the quarantine of a principal.
    #
    # A quarantined principal is *known*: it is held apart, not let go, and
    # the node says so, so that the subject can ask for the isolation to be
    # lifted. The check is here, beside the known-principal check, because
    # both are facts about the subject - and here, before the resource is
    # resolved and before the capability is examined, so that a quarantined
    # principal learns that it is quarantined and nothing else.
    #
    # A quarantine is a state of the subject and not a penalty for an error:
    # no other failure falls back to it. ADR-023.
    if req.principal_id in ctx.quarantined_principals:
        return _quarantine(
            "PRINCIPAL_QUARANTINED", Isolation(req.principal_id), epoch
        )

    # Gate 5 - freshness: nonce, replay cache, expiry.
    if req.nonce in ctx.revoked_nonces:
        return _deny("REPLAY_OR_REVOKED_NONCE", epoch)
    if req.issued_at > ctx.now_monotonic or req.expires_at <= ctx.now_monotonic:
        return _deny("REQUEST_NOT_FRESH", epoch)

    # Gate 6 - object resolution. The name the caller used is resolved to the
    # canonical identity of an object: the identifier the rest of the system
    # knows it by, its type, and its version. A name that resolves to nothing
    # is denied rather than dereferenced. A name that resolves to an object
    # whose type is not the type the request claimed is denied rather than
    # corrected: the type in the envelope is the caller's claim about its own
    # target, and a caller that misdescribes its target is either confused or
    # hostile, and the boundary does not decide which.
    resolved = ctx.resolved_resources.get(req.resource_id)
    if resolved is None:
        return _deny("RESOURCE_UNRESOLVABLE", epoch)
    if resolved.resource_type != req.resource_type:
        return replace(_deny("RESOURCE_TYPE_MISMATCH", epoch), resolved=resolved)

    # Everything from here on is judged against the canonical identity rather
    # than against what the caller wrote, and the decision carries that
    # identity so that the record and the audit name the object the effect
    # will be applied to.
    return replace(_decide(req, cap, ctx, resolved, epoch), resolved=resolved)


def _decide(
    req: Request,
    cap: Optional[Capability],
    ctx: PolicyContext,
    resolved: ResolvedResource,
    epoch: int,
) -> Decision:
    """The gates that follow the resolution.

    These are the gates that need the canonical identity, and they are a
    separate function so that the identity reaches all of them from one place:
    a gate that could read the caller's strings instead would be one line away
    from doing so.
    """
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

    # Gate 9 - policy epoch. The request's epoch, the node's epoch, and the
    # epoch of the registry that will decide it must be the same generation.
    # The registry's own consistency is settled first: a deferral is a
    # decision, and no decision may be made under a registry whose window has
    # not opened or has closed.
    if ctx.registry_epoch != ctx.current_policy_epoch:
        return _deny("REGISTRY_EPOCH_MISMATCH", epoch)
    # The registry's own validity window. A registry with no window is a
    # policy that never expires, and a wall clock that has moved backwards
    # would make an expired registry look current.
    if ctx.clock_state != "TRUSTED":
        return _deny("CLOCK_UNTRUSTED", epoch)
    if ctx.now_wall_ms < ctx.registry_valid_from_ms:
        return _deny("REGISTRY_NOT_YET_VALID", epoch)
    if ctx.now_wall_ms >= ctx.registry_valid_until_ms:
        return _deny("REGISTRY_EXPIRED", epoch)

    # The request's own epoch. A request under a **future** epoch names a
    # policy that does not exist yet, so the answer is neither yes nor no: it
    # is "not yet", and the node defers it. A request under a **stale** epoch
    # names a policy that is gone and will not come back, so the node denies
    # it. ADR-022.
    #
    # The deferral is bounded by the request itself: gate 5 has already
    # refused a request that has expired, so a deferred request becomes a
    # denial once it expires.
    if req.policy_epoch != ctx.current_policy_epoch:
        if req.policy_epoch > ctx.current_policy_epoch and _is_non_destructive(entry):
            return _defer(
                "POLICY_EPOCH_NOT_YET_IN_FORCE",
                Deferral(policy_epoch=req.policy_epoch),
                epoch,
            )
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
    # The grant is over an object, so it is compared with the canonical
    # identity of the object the name resolved to and with nothing the caller
    # wrote. A grant for one object cannot be aimed at another by addressing
    # it differently, and an alias still matches the grant that names the
    # object the alias means.
    if cap.resource_id != resolved.canonical_id or cap.resource_type != resolved.resource_type:
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
    # a higher claim is ignored in favour of the registry's tier. The
    # registry's rules are keyed by canonical resource type, so the type that
    # is looked up is the one the environment resolved.
    if entry is None:
        return _deny("UNKNOWN_OPERATION_RISK", epoch)
    if resolved.resource_type not in entry.resource_types:
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
