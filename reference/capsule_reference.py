"""Senvoryn Change Capsule reference executor (prototype only).

This module is the specification's *reference Change Capsule executor*: it
implements the Change Capsule transaction of Core Contract Specification §5.3
against the machine-readable state machine in
``state-machines/change-capsule-transitions.json``.

It is a PROTOTYPE and a SIMULATOR. It performs no effect, verifies no
signature, resolves no resource, keeps no trust store, and touches no
storage. The world it acts on is a model in this file.

The Rust implementation in ``crates/senvoryn-capsule`` implements the same
contract, in a different language and from a different set of files. The two
are kept in step by the shared capsule vectors in
``conformance/capsule-vectors.json``, which both test suites execute; a
disagreement fails CI. Two implementations of one transaction is the point:
one implementation can only be checked against its own idea of what it
should do.

Two rules shape the code here, and both exist because the alternative has
already caused a real defect elsewhere in this project:

* Nothing that means "a security property was established" is a boolean with
  a safe-looking default. The simulated environment's fields are explicit and
  each is checked where the specification says it must be checked.
* The state table is the authority on which transitions are legal. The
  executor asks the table and never decides legality itself, so the table and
  the code cannot drift apart without the contract pack noticing.
"""

from __future__ import annotations

import enum
import json
import re
from dataclasses import dataclass, field
from datetime import datetime, timedelta, timezone
from pathlib import Path
from typing import Any, Mapping, NoReturn, Optional, Sequence

#: Risk tiers, in ascending order of consequence.
RISK_TIERS = ("R0", "R1", "R2", "R3")

#: Audit classes, in ascending order of required assurance.
AUDIT_CLASSES = ("STANDARD", "SENSITIVE", "SECURITY_CRITICAL")

#: Reversibilities, in ascending order of how hard the effect is to undo.
#: The least reversible sorts last, so that merging two rules for the same
#: operation keeps the stricter one. ``UNKNOWN`` sorts last too, because an
#: operation whose reversibility the registry does not state is treated as the
#: worst case rather than the best one.
REVERSIBILITIES = ("REVERSIBLE", "COMPENSATABLE", "NON_REVERSIBLE", "UNKNOWN")

#: The recovery strategies an executor can carry out without a human.
AUTOMATIC_RECOVERY_STRATEGIES = frozenset({"ROLLBACK", "COMPENSATE", "RESTORE_SNAPSHOT"})

#: The strict UTC timestamp form: ``YYYY-MM-DDTHH:MM:SS[.fff]Z``.
#:
#: A numeric offset is refused rather than converted. Converting one needs a
#: time-zone database, this project has no dependency to provide one, and a
#: capsule whose clock is stated in UTC is not ambiguous. A capsule's validity
#: window is a security boundary, and two readers that disagree about it by
#: hours is a stale capsule that runs.
_TIMESTAMP = re.compile(
    r"^(\d{4})-(\d{2})-(\d{2})T(\d{2}):(\d{2}):(\d{2})(?:\.(\d{1,3}))?Z$"
)

#: The epoch, for the conversion from a civil date to a number of days.
_EPOCH = datetime(1970, 1, 1, tzinfo=timezone.utc)


class CapsuleError(ValueError):
    """A capsule document that cannot be read."""


class State(str, enum.Enum):
    """The states of the Change Capsule transaction.

    A ``str`` enum, so a state's name is the name the state table and the
    vector file use, with no second vocabulary to keep in step.
    """

    DRAFT = "DRAFT"
    VALIDATED = "VALIDATED"
    AUTHORIZED = "AUTHORIZED"
    STAGED = "STAGED"
    PRECHECKED = "PRECHECKED"
    APPLYING = "APPLYING"
    VERIFYING = "VERIFYING"
    COMMITTED = "COMMITTED"
    REJECTED = "REJECTED"
    EXPIRED = "EXPIRED"
    CANCELLED = "CANCELLED"
    FAILED = "FAILED"
    COMPENSATING = "COMPENSATING"
    ROLLED_BACK = "ROLLED_BACK"
    PARTIAL = "PARTIAL"
    NEEDS_HUMAN_RECOVERY = "NEEDS_HUMAN_RECOVERY"


class EffectState(str, enum.Enum):
    """What is known about a step's effect."""

    NOT_APPLIED = "NOT_APPLIED"
    APPLIED = "APPLIED"
    UNKNOWN = "UNKNOWN"


class OutcomeKind(str, enum.Enum):
    """What a step's earlier attempt did."""

    SUCCEEDED = "SUCCEEDED"
    FAILED = "FAILED"
    INTERRUPTED = "INTERRUPTED"


class FaultPoint(enum.Enum):
    """Where in the transaction a fault can be injected."""

    BEFORE_VALIDATION = "BEFORE_VALIDATION"
    BEFORE_AUTHORIZATION = "BEFORE_AUTHORIZATION"
    BEFORE_STAGING = "BEFORE_STAGING"
    AFTER_STAGING = "AFTER_STAGING"
    BEFORE_PRECHECK = "BEFORE_PRECHECK"
    BEFORE_STEP = "BEFORE_STEP"
    DURING_STEP = "DURING_STEP"
    AFTER_STEP = "AFTER_STEP"
    BEFORE_VERIFICATION = "BEFORE_VERIFICATION"
    DURING_VERIFICATION = "DURING_VERIFICATION"


class Fault(str, enum.Enum):
    """What a fault does."""

    IO = "IO"
    CRASH = "CRASH"
    STALE_PRECONDITION = "STALE_PRECONDITION"
    POLICY_EPOCH_CHANGED = "POLICY_EPOCH_CHANGED"
    AUDIT_SINK_LOST = "AUDIT_SINK_LOST"
    DISK_FULL = "DISK_FULL"
    CLOCK_ROLLBACK = "CLOCK_ROLLBACK"
    CORRUPTED_RECOVERY_ARTIFACT = "CORRUPTED_RECOVERY_ARTIFACT"

    @property
    def interrupts(self) -> bool:
        """Whether the fault stops the transaction where it happened.

        An interruption is a moment the transaction does not continue past. A
        change to the world is not: the clock moved backwards, or the recovery
        material turned out to be corrupt, and the transaction carries on and
        finds the change for itself.
        """
        return self not in (Fault.CLOCK_ROLLBACK, Fault.CORRUPTED_RECOVERY_ARTIFACT)


# ---------------------------------------------------------------------------
# The state machine.
# ---------------------------------------------------------------------------


class StateMachine:
    """The Change Capsule state machine, read from the repository's own table.

    The table in ``state-machines/`` is the authority on which transitions
    are legal. The executor asks this object and never decides for itself, so
    a transition the table does not list is not taken whatever the code would
    prefer.
    """

    def __init__(self, document: Mapping[str, Any]) -> None:
        self.initial = State(document["initial_state"])
        self.states = tuple(State(name) for name in document["states"])
        self.terminal_states = frozenset(
            State(name) for name in document["terminal_states"]
        )
        self._guards: dict[tuple[State, State], str] = {}
        for entry in document["transitions"]:
            self._guards[(State(entry["from"]), State(entry["to"]))] = entry["guard"]

    def permits(self, source: State, destination: State) -> bool:
        """Whether the table allows this transition."""
        return (source, destination) in self._guards

    def guard(self, source: State, destination: State) -> Optional[str]:
        """The guard the table names for this transition, if it lists one."""
        return self._guards.get((source, destination))

    def successors(self, source: State) -> tuple[State, ...]:
        """The states the table allows from here."""
        return tuple(
            destination for (origin, destination) in self._guards if origin == source
        )


def load_state_machine(path: Optional[Path] = None) -> StateMachine:
    """Reads the state machine from the repository's contract pack."""
    if path is None:
        path = Path(__file__).resolve().parents[1] / "state-machines" / "change-capsule-transitions.json"
    with open(path, encoding="utf-8") as handle:
        return StateMachine(json.load(handle))


# ---------------------------------------------------------------------------
# The capsule.
# ---------------------------------------------------------------------------


@dataclass(frozen=True)
class Target:
    """What a capsule acts on."""

    target_id: str
    target_type: str
    expected_version: str


@dataclass(frozen=True)
class Precondition:
    """A condition that must hold before the capsule may run."""

    predicate_id: str
    observed_version: Optional[str]


@dataclass(frozen=True)
class Postcondition:
    """A condition that must hold after the capsule has run."""

    predicate_id: str
    mandatory: bool


@dataclass(frozen=True)
class Step:
    """One step of a capsule."""

    step_id: str
    operation: str
    idempotency_key: str
    parameters_digest: str
    risk_tier: str
    timeout_ms: int
    success_predicates: frozenset[str]
    recovery_action_id: Optional[str]


@dataclass(frozen=True)
class ChangeCapsule:
    """An immutable plan, its evidence, and the scoped authority it requests.

    A capsule is **not** a script. It names operations from the trusted
    Operation Registry, states the preconditions that must hold, the
    postconditions that must be true afterwards, the budget it may spend, and
    how its effects are undone.
    """

    capsule_id: str
    originator_id: str
    target: Target
    intent_id: str
    policy_epoch: int
    risk_tier: str
    reversibility: str
    preconditions: tuple[Precondition, ...]
    steps: tuple[Step, ...]
    budget_duration_ms: int
    budget_operation_count: int
    postconditions: tuple[Postcondition, ...]
    recovery_strategy: str
    recovery_artifact_ref: Optional[str]
    irreversible_effects: frozenset[str]
    human_instructions_ref: Optional[str]
    issued_at: str
    expires_at: str
    issued_at_ms: int
    expires_at_ms: int
    approval_requirements: tuple[tuple[str, int], ...]
    payload_digest: str
    signature_refs: tuple[str, ...]

    @property
    def claimed_risk_tier(self) -> str:
        """The highest risk tier any of the capsule's steps claims.

        The registry is the authority on risk, so this is what the capsule
        *claims*; the executor compares it with what the registry says.
        """
        if not self.steps:
            return self.risk_tier
        return max((step.risk_tier for step in self.steps), key=RISK_TIERS.index)

    @property
    def mandatory_postconditions(self) -> tuple[str, ...]:
        """The postconditions the capsule may not be committed without."""
        return tuple(
            postcondition.predicate_id
            for postcondition in self.postconditions
            if postcondition.mandatory
        )

    @property
    def automatically_recoverable(self) -> bool:
        """Whether the recovery plan can be carried out without a human.

        A plan whose strategy asks for human recovery, or which names no
        strategy at all, cannot be carried out by the executor. Neither can
        one whose recovery artifact is missing.
        """
        return (
            self.recovery_strategy in AUTOMATIC_RECOVERY_STRATEGIES
            and self.recovery_artifact_ref is not None
        )


def parse_utc_millis(text: str) -> int:
    """Parses a strict UTC timestamp into milliseconds since the Unix epoch.

    Only ``YYYY-MM-DDTHH:MM:SSZ`` and ``YYYY-MM-DDTHH:MM:SS.fffZ`` are
    accepted. A numeric offset is refused rather than converted.

    The value has to be a real instant. A date that does not exist, such as
    the thirtieth of February, is refused rather than rolled over into the
    next month: a capsule that names a time which never happened is
    malformed, and silently moving it would mean the executor compared the
    capsule against an instant its issuer never stated.
    """
    match = _TIMESTAMP.fullmatch(text)
    if match is None:
        raise CapsuleError(f"{text!r} is not a UTC timestamp")
    year, month, day, hour, minute, second = (
        int(part) for part in match.group(1, 2, 3, 4, 5, 6)
    )
    if year < 1:
        raise CapsuleError(f"{text!r} is not a date")
    if not 1 <= month <= 12:
        raise CapsuleError(f"{text!r} is not a date")
    if not 1 <= day <= _days_in_month(year, month):
        raise CapsuleError(f"{text!r} is not a date")
    if hour > 23 or minute > 59 or second > 59:
        raise CapsuleError(f"{text!r} is not a time")
    moment = datetime(year, month, day, hour, minute, second, tzinfo=timezone.utc)
    seconds = (moment - _EPOCH) // timedelta(seconds=1)
    millis = int((match.group(7) or "").ljust(3, "0"))
    return seconds * 1000 + millis


def _days_in_month(year: int, month: int) -> int:
    """How many days a month has, in the proleptic Gregorian calendar."""
    lengths = (31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31)
    if month == 2 and _is_leap_year(year):
        return 29
    return lengths[month - 1]


def _is_leap_year(year: int) -> bool:
    return year % 4 == 0 and (year % 100 != 0 or year % 400 == 0)


def _require(document: Mapping[str, Any], field_name: str) -> Any:
    if field_name not in document:
        raise CapsuleError(f"capsule field {field_name!r} is absent")
    return document[field_name]


def _require_str(document: Mapping[str, Any], field_name: str) -> str:
    value = _require(document, field_name)
    if not isinstance(value, str):
        raise CapsuleError(f"capsule field {field_name!r} is not a string")
    return value


def _require_int(document: Mapping[str, Any], field_name: str) -> int:
    value = _require(document, field_name)
    if not isinstance(value, int) or isinstance(value, bool) or value < 0:
        raise CapsuleError(f"capsule field {field_name!r} is not a non-negative integer")
    return value


def _optional_str(document: Mapping[str, Any], field_name: str) -> Optional[str]:
    value = document.get(field_name)
    if value is None:
        return None
    if not isinstance(value, str):
        raise CapsuleError(f"capsule field {field_name!r} is not a string")
    return value


def _string_set(document: Mapping[str, Any], field_name: str) -> frozenset[str]:
    value = document.get(field_name, [])
    if not isinstance(value, list) or not all(isinstance(item, str) for item in value):
        raise CapsuleError(f"capsule field {field_name!r} is not an array of strings")
    return frozenset(value)


def load_capsule(document: Mapping[str, Any]) -> ChangeCapsule:
    """Reads a capsule from a document."""
    schema = _require_str(document, "schema")
    if schema != "senvoryn.change-capsule.v1":
        raise CapsuleError(f"{schema!r} is not a change capsule")

    target = _require(document, "target")
    if not isinstance(target, Mapping):
        raise CapsuleError("capsule field 'target' is not an object")

    issued_at = _require_str(document, "issued_at")
    expires_at = _require_str(document, "expires_at")
    try:
        issued_at_ms = parse_utc_millis(issued_at)
    except CapsuleError as error:
        raise CapsuleError(f"capsule field 'issued_at' is not a UTC timestamp: {error}") from error
    try:
        expires_at_ms = parse_utc_millis(expires_at)
    except CapsuleError as error:
        raise CapsuleError(f"capsule field 'expires_at' is not a UTC timestamp: {error}") from error

    risk_tier = _require_str(document, "risk_tier")
    if risk_tier not in RISK_TIERS:
        raise CapsuleError(f"capsule field 'risk_tier' is not a risk tier")
    reversibility = _require_str(document, "reversibility")
    if reversibility not in REVERSIBILITIES:
        raise CapsuleError(f"capsule field 'reversibility' is not a reversibility")

    preconditions = []
    for entry in _require(document, "preconditions"):
        if not isinstance(entry, Mapping):
            raise CapsuleError("capsule field 'preconditions' is not an array of objects")
        # The schema requires `expected`: a precondition that does not say
        # what it expects is not a precondition. The executor's environment
        # models preconditions by predicate name, so the value itself is
        # carried for the evidence rather than evaluated here, but its
        # absence is a malformed capsule.
        if "expected" not in entry:
            raise CapsuleError(
                "capsule field 'preconditions' holds a precondition that does not "
                "say what it expects"
            )
        preconditions.append(
            Precondition(
                predicate_id=_require_str(entry, "predicate_id"),
                observed_version=_optional_str(entry, "observed_version"),
            )
        )

    steps = []
    for entry in _require(document, "steps"):
        if not isinstance(entry, Mapping):
            raise CapsuleError("capsule field 'steps' is not an array of objects")
        step_risk = _require_str(entry, "risk_tier")
        if step_risk not in RISK_TIERS:
            raise CapsuleError("capsule field 'steps[].risk_tier' is not a risk tier")
        steps.append(
            Step(
                step_id=_require_str(entry, "step_id"),
                operation=_require_str(entry, "operation"),
                idempotency_key=_require_str(entry, "idempotency_key"),
                parameters_digest=_require_str(entry, "parameters_digest"),
                risk_tier=step_risk,
                timeout_ms=_require_int(entry, "timeout_ms"),
                success_predicates=_string_set(entry, "success_predicates"),
                recovery_action_id=_optional_str(entry, "recovery_action_id"),
            )
        )

    budget = _require(document, "budget")
    if not isinstance(budget, Mapping):
        raise CapsuleError("capsule field 'budget' is not an object")

    postconditions = []
    for entry in _require(document, "postconditions"):
        if not isinstance(entry, Mapping):
            raise CapsuleError("capsule field 'postconditions' is not an array of objects")
        mandatory = entry.get("mandatory")
        if not isinstance(mandatory, bool):
            raise CapsuleError("capsule field 'postconditions[].mandatory' is not a boolean")
        postconditions.append(
            Postcondition(
                predicate_id=_require_str(entry, "predicate_id"),
                mandatory=mandatory,
            )
        )

    recovery = _require(document, "recovery_plan")
    if not isinstance(recovery, Mapping):
        raise CapsuleError("capsule field 'recovery_plan' is not an object")

    approval_requirements = []
    for entry in _require(document, "approval_requirements"):
        if not isinstance(entry, Mapping):
            raise CapsuleError(
                "capsule field 'approval_requirements' is not an array of objects"
            )
        approval_requirements.append(
            (
                _require_str(entry, "approval_type"),
                _require_int(entry, "count"),
            )
        )

    return ChangeCapsule(
        capsule_id=_require_str(document, "capsule_id"),
        originator_id=_require_str(document, "originator_id"),
        target=Target(
            target_id=_require_str(target, "target_id"),
            target_type=_require_str(target, "target_type"),
            expected_version=_require_str(target, "expected_version"),
        ),
        intent_id=_require_str(document, "intent_id"),
        policy_epoch=_require_int(document, "policy_epoch"),
        risk_tier=risk_tier,
        reversibility=reversibility,
        preconditions=tuple(preconditions),
        steps=tuple(steps),
        budget_duration_ms=_require_int(budget, "duration_ms"),
        budget_operation_count=_require_int(budget, "operation_count"),
        postconditions=tuple(postconditions),
        recovery_strategy=_require_str(recovery, "strategy"),
        recovery_artifact_ref=_optional_str(recovery, "recovery_artifact_ref"),
        irreversible_effects=_string_set(recovery, "irreversible_effects"),
        human_instructions_ref=_optional_str(recovery, "human_instructions_ref"),
        issued_at=issued_at,
        expires_at=expires_at,
        issued_at_ms=issued_at_ms,
        expires_at_ms=expires_at_ms,
        approval_requirements=tuple(approval_requirements),
        payload_digest=_require_str(document, "payload_digest"),
        signature_refs=tuple(sorted(_string_set(document, "signature_refs"))),
    )


# ---------------------------------------------------------------------------
# The operation registry.
# ---------------------------------------------------------------------------


@dataclass(frozen=True)
class OperationEntry:
    """What the registry says about one operation."""

    operation_id: str
    risk_tier: str
    reversibility: str
    approval_requirements: frozenset[str]
    allowed_modes: frozenset[str]
    resource_types: frozenset[str]
    required_preconditions: frozenset[str]
    required_postconditions: frozenset[str]
    external_effects: frozenset[str]
    idempotent: bool
    max_duration_ms: Optional[int]
    max_operation_count: Optional[int]
    audit_class: str

    @property
    def compensatable(self) -> bool:
        """Whether the operation's effect can be undone."""
        return self.reversibility in ("REVERSIBLE", "COMPENSATABLE")


class OperationRegistry:
    """The trusted Operation Registry.

    Where more than one rule applies to the same operation, the most
    restrictive one wins, on every axis: the highest risk tier, the narrowest
    mode set, the union of the approval requirements, the smallest budgets,
    the highest audit class, and the least reversible behaviour. A caller
    cannot escape a strict rule by also matching a lax one.
    """

    def __init__(self, entries: Sequence[OperationEntry]) -> None:
        self._entries = list(entries)

    def entries(self) -> tuple[OperationEntry, ...]:
        """Every entry in the registry."""
        return tuple(self._entries)

    def effective(self, operation_id: str) -> Optional[OperationEntry]:
        """The merged rule for an operation, or ``None`` if it has none."""
        matching = [
            entry for entry in self._entries if entry.operation_id == operation_id
        ]
        if not matching:
            return None
        merged = matching[0]
        for entry in matching[1:]:
            merged = _merge_entries(merged, entry)
        return merged


def _merge_entries(left: OperationEntry, right: OperationEntry) -> OperationEntry:
    """Merges two rules for the same operation, most restrictive first."""
    return OperationEntry(
        operation_id=left.operation_id,
        risk_tier=max(left.risk_tier, right.risk_tier, key=RISK_TIERS.index),
        reversibility=max(left.reversibility, right.reversibility, key=REVERSIBILITIES.index),
        approval_requirements=left.approval_requirements | right.approval_requirements,
        allowed_modes=left.allowed_modes & right.allowed_modes,
        resource_types=left.resource_types | right.resource_types,
        required_preconditions=left.required_preconditions | right.required_preconditions,
        required_postconditions=left.required_postconditions | right.required_postconditions,
        external_effects=left.external_effects | right.external_effects,
        idempotent=left.idempotent and right.idempotent,
        max_duration_ms=_min_option(left.max_duration_ms, right.max_duration_ms),
        max_operation_count=_min_option(left.max_operation_count, right.max_operation_count),
        audit_class=max(left.audit_class, right.audit_class, key=AUDIT_CLASSES.index),
    )


def _min_option(left: Optional[int], right: Optional[int]) -> Optional[int]:
    if left is None:
        return right
    if right is None:
        return left
    return min(left, right)


def load_registry(path: Optional[Path] = None) -> OperationRegistry:
    """Reads the operation registry from the repository's contract pack."""
    if path is None:
        path = Path(__file__).resolve().parents[1] / "examples" / "operation-registry.sample.json"
    with open(path, encoding="utf-8") as handle:
        document = json.load(handle)
    entries = []
    for entry in document["operations"]:
        entries.append(
            OperationEntry(
                operation_id=entry["operation_id"],
                risk_tier=entry["risk_tier"],
                reversibility=entry["reversibility"],
                approval_requirements=frozenset(entry["approval_requirements"]),
                allowed_modes=frozenset(entry["allowed_modes"]),
                resource_types=frozenset(entry["resource_types"]),
                required_preconditions=frozenset(entry.get("required_preconditions", [])),
                required_postconditions=frozenset(entry.get("required_postconditions", [])),
                external_effects=frozenset(entry.get("external_effects", [])),
                idempotent=entry.get("idempotent", False),
                max_duration_ms=entry.get("max_duration_ms"),
                max_operation_count=entry.get("max_operation_count"),
                audit_class=entry.get("audit_class", "STANDARD"),
            )
        )
    return OperationRegistry(entries)


# ---------------------------------------------------------------------------
# The environment, the ledger, the faults.
# ---------------------------------------------------------------------------


@dataclass
class Environment:
    """The simulated system a capsule acts on.

    Every field has an explicit value because a test has to be able to move
    the world: a precondition to stop holding, a version to change, an effect
    to appear.
    """

    current_version: str = ""
    policy_epoch: int = 0
    now_ms: int = 0
    satisfied_preconditions: set[str] = field(default_factory=set)
    observed_versions: dict[str, str] = field(default_factory=dict)
    satisfied_postconditions: set[str] = field(default_factory=set)
    staged: bool = False
    applied_steps: list[str] = field(default_factory=list)
    compensated_steps: list[str] = field(default_factory=list)
    outstanding_effects: set[str] = field(default_factory=set)
    recovery_material_present: bool = True
    recovery_material_intact: bool = True
    audit_sink_durable: bool = True

    def precondition_holds(self, predicate_id: str) -> bool:
        """Whether a precondition currently holds."""
        return predicate_id in self.satisfied_preconditions

    def observed_version(self, predicate_id: str) -> Optional[str]:
        """The version a precondition observed, if it observed one."""
        return self.observed_versions.get(predicate_id)

    def postcondition_holds(self, predicate_id: str) -> bool:
        """Whether a postcondition currently holds."""
        return predicate_id in self.satisfied_postconditions

    def apply_step(self, step_id: str) -> None:
        """Records that a step's effect landed."""
        self.applied_steps.append(step_id)
        self.outstanding_effects.add(step_id)

    def compensate_step(self, step_id: str) -> None:
        """Records that a step's effect was undone."""
        self.compensated_steps.append(step_id)
        self.outstanding_effects.discard(step_id)


@dataclass(frozen=True)
class StepOutcome:
    """What an earlier attempt at a step did."""

    kind: OutcomeKind
    effect: EffectState = EffectState.NOT_APPLIED

    @property
    def may_retry(self) -> bool:
        """Whether the step may be attempted again.

        Only an outcome that is *known* not to have applied may be retried.
        An effect that is not known may have landed, and repeating a
        non-idempotent operation would apply it twice.
        """
        return self.kind is OutcomeKind.INTERRUPTED and self.effect is EffectState.NOT_APPLIED


class IdempotencyLedger:
    """What is known about the steps of a capsule that has been attempted.

    The ledger is what stops a repeated attempt from repeating a step that
    already ran, and what stops it from repeating one whose effect is not
    known.
    """

    def __init__(self) -> None:
        self._outcomes: dict[tuple[str, str], StepOutcome] = {}

    def lookup(self, capsule_id: str, idempotency_key: str) -> Optional[StepOutcome]:
        """The outcome of an earlier attempt, if there was one."""
        return self._outcomes.get((capsule_id, idempotency_key))

    def record(
        self, capsule_id: str, idempotency_key: str, outcome: StepOutcome
    ) -> None:
        """Records what an attempt did."""
        self._outcomes[(capsule_id, idempotency_key)] = outcome

    def __len__(self) -> int:
        return len(self._outcomes)


class FaultInjector:
    """The faults to inject, by point.

    A fault is not consumed when it is read: the same fault is there at every
    reading of its point, which is what makes "the step was interrupted" and
    "the step's effect landed" two separate readings of one interruption.
    """

    def __init__(
        self, faults: Optional[Mapping[tuple[FaultPoint, Optional[int]], Fault]] = None
    ) -> None:
        self._faults = dict(faults or {})

    @classmethod
    def at(
        cls, point: FaultPoint, fault: Fault, index: Optional[int] = None
    ) -> "FaultInjector":
        """An injector that fails at one point."""
        return cls({(point, index): fault})

    def fault_at(
        self, point: FaultPoint, index: Optional[int] = None
    ) -> Optional[Fault]:
        """The fault at a point, if there is one."""
        return self._faults.get((point, index))


# ---------------------------------------------------------------------------
# The authorization.
# ---------------------------------------------------------------------------


@dataclass(frozen=True)
class Approval:
    """An approval given for a capsule."""

    approval_type: str
    binds_digest: str


#: The fields a decision record defines. The form is closed: a record that
#: carries a field this list does not name is refused rather than read, for
#: the same reason the policy request form is closed (ADR-019).
DECISION_RECORD_FIELDS = frozenset(
    {
        "schema",
        "decision_id",
        "request_id",
        "status",
        "reason_codes",
        "policy_epoch",
        "evaluated_at",
        "target_resource_id",
        "target_resource_version",
        "permitted_payload_digest",
        "capability_scope_ref",
        "valid_until",
        "required_next_step",
        "effect_performed",
        "audit_ref",
    }
)

#: The decision statuses that grant authority. Only ``PERMIT`` does.
AUTHORITY_GRANTING_STATUSES = frozenset({"PERMIT"})


@dataclass(frozen=True)
class DecisionRecord:
    """A decision record, read.

    The record is the policy layer's own output. The executor is handed one
    of these rather than a value the host assembled, so that the plan the
    decision was made about and the version of the target it was made
    against come from the decision instead of from the caller.
    """

    decision_id: str
    request_id: str
    status: str
    reason_codes: tuple[str, ...]
    policy_epoch: int
    evaluated_at: str
    target_resource_id: str
    target_resource_version: Optional[str]
    permitted_payload_digest: Optional[str]
    capability_scope_ref: Optional[str]
    valid_until: Optional[str]
    required_next_step: Optional[str]
    audit_ref: str


def read_decision_record(document: dict) -> DecisionRecord:
    """Read a decision record, or raise ``ValueError``.

    The reader enforces the form and nothing else. It refuses a field the
    record does not define, a record that declares a schema this node does
    not support, and a record that claims an effect was performed: the
    record's schema fixes ``effect_performed`` to false, and the policy layer
    cannot have performed one.
    """
    if not isinstance(document, dict):
        raise ValueError(f"expected a decision record object, found {type(document).__name__}")
    unknown = sorted(set(document) - DECISION_RECORD_FIELDS)
    if unknown:
        raise ValueError(f"the decision record carries a field it does not define: {unknown[0]!r}")
    schema = document.get("schema")
    if schema != "senvoryn.decision-record.v1":
        raise ValueError(f"the decision record declares an unsupported schema: {schema!r}")
    if document.get("effect_performed") is not False:
        raise ValueError("the decision record claims an effect was performed")
    status = document.get("status")
    if status not in AUTHORITY_GRANTING_STATUSES and not isinstance(status, str):
        raise ValueError(f"the decision record's status is not a string: {status!r}")

    def required_text(field: str) -> str:
        value = document.get(field)
        if not isinstance(value, str) or not value:
            raise ValueError(f"the decision record's {field!r} is absent or not a string")
        return value

    def optional_text(field: str) -> Optional[str]:
        value = document.get(field)
        if value is None:
            return None
        if not isinstance(value, str) or not value:
            raise ValueError(f"the decision record's {field!r} is not a non-empty string")
        return value

    reason_codes = document.get("reason_codes")
    if not isinstance(reason_codes, list) or not all(
        isinstance(code, str) for code in reason_codes
    ):
        raise ValueError("the decision record's 'reason_codes' is absent or not a list of strings")
    policy_epoch = document.get("policy_epoch")
    if not isinstance(policy_epoch, int) or isinstance(policy_epoch, bool) or policy_epoch < 0:
        raise ValueError("the decision record's 'policy_epoch' is absent or not an integer")

    return DecisionRecord(
        decision_id=required_text("decision_id"),
        request_id=required_text("request_id"),
        status=status,
        reason_codes=tuple(reason_codes),
        policy_epoch=policy_epoch,
        evaluated_at=required_text("evaluated_at"),
        target_resource_id=required_text("target_resource_id"),
        target_resource_version=optional_text("target_resource_version"),
        permitted_payload_digest=optional_text("permitted_payload_digest"),
        capability_scope_ref=optional_text("capability_scope_ref"),
        valid_until=optional_text("valid_until"),
        required_next_step=optional_text("required_next_step"),
        audit_ref=required_text("audit_ref"),
    )


@dataclass(frozen=True)
class Authorization:
    """The System Control Plane's decision about a capsule.

    The SCP is the authority on whether a capsule may run. This model reads
    the decision it made and checks the thing the decision has to bind — the
    exact digest of the plan that was authorized, and the version of the
    target the decision was made against — plus the approvals the capsule
    asked for. It does not decide policy.
    """

    decision_status: str
    permitted_digest: Optional[str]
    permitted_resource_version: Optional[str]
    approvals: tuple[Approval, ...]

    @classmethod
    def from_record(
        cls, record: DecisionRecord, approvals: tuple[Approval, ...]
    ) -> "Authorization":
        """Read the authorization out of a decision record."""
        return cls(
            decision_status=record.status,
            permitted_digest=record.permitted_payload_digest,
            permitted_resource_version=record.target_resource_version,
            approvals=approvals,
        )


def check_authorization(
    capsule: ChangeCapsule, authorization: Authorization
) -> Optional[str]:
    """The reason the authorization does not permit the capsule, or ``None``.

    The status comes first: a decision that granted no authority authorises
    nothing, and no amount of matching digests makes a refusal into a grant.
    The digest check comes next because it is the one that cannot be worked
    around: an approval for a different plan is not an approval for this
    one, however many there are of them. The version check comes after it: a
    decision names the version of the target it was made against, and the
    target may have moved since (ADR-018, threat model AS-19).
    """
    if authorization.decision_status not in AUTHORITY_GRANTING_STATUSES:
        return (
            f"the decision granted no authority: its status is "
            f"{authorization.decision_status}"
        )
    if authorization.permitted_digest is None:
        return "the decision does not name the plan it was made about"
    if authorization.permitted_digest != capsule.payload_digest:
        return (
            f"the decision permits digest {authorization.permitted_digest!r}, "
            f"not the capsule's {capsule.payload_digest!r}"
        )
    permitted_version = authorization.permitted_resource_version
    if permitted_version is None:
        return "the decision does not name the version of the target it was made against"
    if permitted_version != capsule.target.expected_version:
        return (
            f"the decision was made against version {permitted_version!r} of the "
            f"target, not the version {capsule.target.expected_version!r} the "
            "capsule expects"
        )
    given: dict[str, int] = {}
    for approval in authorization.approvals:
        if approval.binds_digest != capsule.payload_digest:
            return f"a {approval.approval_type} approval was given for a different plan"
        given[approval.approval_type] = given.get(approval.approval_type, 0) + 1
    for approval_type, count in capsule.approval_requirements:
        supplied = given.get(approval_type, 0)
        if supplied == 0:
            return f"the capsule requires a {approval_type} approval and none was given"
        if supplied < count:
            return (
                f"the capsule requires {count} {approval_type} approval(s) and "
                f"{supplied} were given"
            )
    return None


# ---------------------------------------------------------------------------
# The executor.
# ---------------------------------------------------------------------------


@dataclass(frozen=True)
class ExecutionOutcome:
    """What the executor did."""

    final_state: State
    reason: str
    steps_executed: tuple[str, ...]
    steps_reused: tuple[str, ...]
    history: tuple[State, ...]


class _Halt(Exception):
    """Raised when the transaction ends where it is."""

    def __init__(self, state: State, reason: str) -> None:
        super().__init__(reason)
        self.state = state
        self.reason = reason


class Executor:
    """The Change Capsule executor.

    The executor's job is narrow and it is the whole of it: **move the
    transaction from one state to the next only when the table allows it and
    the guard holds.** It does not decide policy, and it does not decide
    whether a capsule is acceptable. What it does is refuse to report a
    result it cannot prove.

    The property this exists for: **an interrupted operation never falsely
    reports ``COMMITTED``.** Every path to ``COMMITTED`` requires that every
    step has an outcome, that every mandatory postcondition holds, and that
    the evidence is durable. A fault at any point breaks one of those, so a
    fault at any point ends the transaction somewhere other than
    ``COMMITTED``.
    """

    def __init__(
        self,
        capsule: ChangeCapsule,
        registry: OperationRegistry,
        environment: Environment,
        machine: Optional[StateMachine] = None,
        ledger: Optional[IdempotencyLedger] = None,
        faults: Optional[FaultInjector] = None,
    ) -> None:
        self._capsule = capsule
        self._registry = registry
        self._environment = environment
        self._machine = machine if machine is not None else load_state_machine()
        self._ledger = ledger if ledger is not None else IdempotencyLedger()
        self._faults = faults if faults is not None else FaultInjector()
        self._state = self._machine.initial
        self._history: list[State] = [self._state]
        self._executed: list[str] = []
        self._reused: list[str] = []
        self._fired: list[tuple[FaultPoint, Optional[int]]] = []
        # The highest time the transaction has observed. A clock that moves
        # backwards makes the capsule's validity window meaningless, so the
        # transaction refuses to act on a clock that has gone behind it.
        self._highest_now_ms: Optional[int] = None

    # -- the transaction ----------------------------------------------------

    def run(self, authorization: Authorization) -> ExecutionOutcome:
        """Runs the transaction to wherever it ends."""
        self._executed = []
        self._reused = []
        try:
            self._validate()
            self._authorize(authorization)
            self._stage()
            self._precheck()
            self._apply()
            self._verify()
        except _Halt as halt:
            return self._outcome(halt.state, halt.reason)
        return self._outcome(
            self._state, "every mandatory postcondition held and the evidence is durable"
        )

    def _validate(self) -> None:
        """``DRAFT → VALIDATED``: the capsule is a plan the registry accepts."""
        fault = self._fault_at(FaultPoint.BEFORE_VALIDATION)
        if fault is not None and fault.interrupts:
            raise self._fail(
                f"the capsule was interrupted before it was validated ({fault.value})"
            )
        now = self._now()
        # A capsule cannot be executed before it was issued, so a clock that
        # reads earlier than the issue time is a clock that cannot be
        # trusted. This is the only check that catches a clock that moved
        # backwards before the transaction had observed it at all.
        if now < self._capsule.issued_at_ms:
            raise self._reject("the capsule has not been issued yet")
        if self._capsule.expires_at_ms <= now:
            raise self._expire("the capsule's validity window has closed")
        # The capsule's own claims have to agree with one another. It says the
        # plan takes at most `budget.duration_ms`, and it says it is only valid
        # until `expires_at`. A plan whose own bound outlasts its own window
        # cannot finish inside the authority it was issued under, so the
        # capsule contradicts itself. This is a consistency check, not a policy
        # one: it needs no clock, and it is checked at validation so the
        # operator learns before anything is staged.
        if (
            self._capsule.issued_at_ms + self._capsule.budget_duration_ms
            > self._capsule.expires_at_ms
        ):
            raise self._reject(
                f"the capsule's budget of {self._capsule.budget_duration_ms}ms does not fit "
                f"inside its validity window of "
                f"{self._capsule.expires_at_ms - self._capsule.issued_at_ms}ms"
            )

        for step in self._capsule.steps:
            entry = self._registry.effective(step.operation)
            if entry is None:
                raise self._reject(
                    f"the operation {step.operation!r} is not in the operation registry"
                )
            # The registry is the authority on risk. A capsule that claims
            # less risk than the registry assigns is refused; a capsule that
            # claims more is not punished for it, exactly as in the SCP.
            if RISK_TIERS.index(step.risk_tier) < RISK_TIERS.index(entry.risk_tier):
                raise self._reject(
                    f"the step {step.step_id!r} claims {step.risk_tier!r} but the "
                    f"registry assigns {entry.risk_tier!r}"
                )
            # The capsule may not ask for less authority than the registry
            # demands of the operation it performs. The System Control Plane
            # is supposed to have caught this already, but the executor is
            # the last gate before the effect, and a plan that understates the
            # authority it needs must not run.
            for required in sorted(entry.approval_requirements):
                if required == "NONE":
                    continue
                if required not in {
                    approval_type for approval_type, _ in self._capsule.approval_requirements
                }:
                    raise self._reject(
                        f"the capsule does not require the {required} approval the "
                        f"operation {step.operation!r} demands"
                    )
            if entry.max_duration_ms is not None and step.timeout_ms > entry.max_duration_ms:
                raise self._reject(
                    f"the step {step.step_id!r} may run for {step.timeout_ms}ms but the "
                    f"registry allows {entry.max_duration_ms}ms"
                )
            if entry.max_operation_count is not None:
                # The registry's count is per operation: how many times that
                # operation may be performed. The capsule's own count is a
                # total across the plan, so the two are not compared to each
                # other.
                uses = sum(1 for other in self._capsule.steps if other.operation == step.operation)
                if uses > entry.max_operation_count:
                    raise self._reject(
                        f"the capsule performs the operation {step.operation!r} {uses} "
                        f"time(s) but the registry allows {entry.max_operation_count}"
                    )

        if RISK_TIERS.index(self._capsule.risk_tier) < RISK_TIERS.index(
            self._capsule.claimed_risk_tier
        ):
            raise self._reject("the capsule's own risk tier is below the risk of its steps")
        if self._capsule.budget_operation_count < len(self._capsule.steps):
            raise self._reject(
                f"the capsule's budget covers {self._capsule.budget_operation_count} "
                f"operation(s) but the plan has {len(self._capsule.steps)} step(s)"
            )
        planned = sum(step.timeout_ms for step in self._capsule.steps)
        if self._capsule.budget_duration_ms < planned:
            raise self._reject(
                f"the capsule's budget of {self._capsule.budget_duration_ms}ms does not "
                f"cover its steps, which ask for {planned}ms"
            )
        # The registry bounds how long each operation may run. A transaction
        # cannot outlast the operations it performs, so its budget is bounded
        # by their combined allowance.
        allowance = 0
        for step in self._capsule.steps:
            entry = self._registry.effective(step.operation)
            if entry is not None and entry.max_duration_ms is not None:
                allowance += entry.max_duration_ms
        if self._capsule.budget_duration_ms > allowance:
            raise self._reject(
                f"the capsule's budget of {self._capsule.budget_duration_ms}ms exceeds "
                f"the {allowance}ms its operations are allowed"
            )
        if self._capsule.policy_epoch != self._environment.policy_epoch:
            raise self._reject("the capsule was made under a stale policy epoch")
        if self._capsule.target.expected_version != self._environment.current_version:
            raise self._reject("the target is not at the version the capsule expects")
        for precondition in self._capsule.preconditions:
            if not self._environment.precondition_holds(precondition.predicate_id):
                raise self._reject(
                    f"the precondition {precondition.predicate_id!r} does not hold"
                )
        self._transition(State.VALIDATED)

    def _authorize(self, authorization: Authorization) -> None:
        """``VALIDATED → AUTHORIZED``: the decision permits this exact plan."""
        fault = self._fault_at(FaultPoint.BEFORE_AUTHORIZATION)
        if fault is not None and fault.interrupts:
            raise self._fail(
                f"the capsule was interrupted before it was authorized ({fault.value})"
            )
        problem = check_authorization(self._capsule, authorization)
        if problem is not None:
            raise self._reject(f"the decision does not permit the capsule: {problem}")
        self._transition(State.AUTHORIZED)

    def _stage(self) -> None:
        """``AUTHORIZED → STAGED``: the artifacts are staged outside the active storage."""
        fault = self._fault_at(FaultPoint.BEFORE_STAGING)
        if fault is not None and fault.interrupts:
            raise self._fail(f"the capsule was interrupted before staging ({fault.value})")
        self._environment.staged = True
        fault = self._fault_at(FaultPoint.AFTER_STAGING)
        if fault is not None and fault.interrupts:
            raise self._fail(f"the capsule was interrupted after staging ({fault.value})")
        self._transition(State.STAGED)

    def _precheck(self) -> None:
        """``STAGED → PRECHECKED``: the world is rechecked, because it moved."""
        fault = self._fault_at(FaultPoint.BEFORE_PRECHECK)
        if fault is not None and fault.interrupts:
            raise self._fail(f"the capsule was interrupted before the precheck ({fault.value})")
        # The clock is read again, and a clock that has gone backwards is not
        # a clock the transaction can act on. The validity window is *not*
        # rechecked here: the simulator cannot move the clock forward between
        # the validation and the precheck, so a check here could never fire,
        # and a control that has never been seen to fail has proved nothing.
        # The window is checked before every step, which is where the clock
        # does move.
        self._now()
        if self._capsule.target.expected_version != self._environment.current_version:
            raise self._fail("the target version changed under the capsule")
        if self._capsule.policy_epoch != self._environment.policy_epoch:
            raise self._fail("the policy epoch moved on under the capsule")
        for precondition in self._capsule.preconditions:
            if not self._environment.precondition_holds(precondition.predicate_id):
                raise self._fail(
                    f"the precondition {precondition.predicate_id!r} no longer holds"
                )
        if self._capsule.recovery_artifact_ref is not None:
            if not self._environment.recovery_material_present:
                raise self._fail("the recovery artifact is missing")
            # Presence is not integrity. Material that is present but corrupt
            # would be used to undo the effect.
            if not self._environment.recovery_material_intact:
                raise self._fail("the recovery artifact is corrupt")
        self._transition(State.PRECHECKED)

    def _apply(self) -> None:
        """``PRECHECKED → APPLYING → VERIFYING``: the steps run.

        The ledger is consulted before every step. A step whose outcome is
        already known is not run again, and a step whose effect may have
        landed is never run again at all.
        """
        self._transition(State.APPLYING)
        capsule_id = self._capsule.capsule_id

        for index, step in enumerate(self._capsule.steps):
            earlier = self._ledger.lookup(capsule_id, step.idempotency_key)
            if earlier is not None:
                if earlier.kind is OutcomeKind.SUCCEEDED:
                    self._reused.append(step.step_id)
                    continue
                if earlier.kind is OutcomeKind.FAILED:
                    self._reused.append(step.step_id)
                    raise self._fail(f"the step {step.step_id!r} failed on an earlier attempt")
                if earlier.effect is not EffectState.NOT_APPLIED:
                    # The effect may have landed. Repeating a non-idempotent
                    # operation would apply it twice.
                    self._compensate(
                        f"the step {step.step_id!r} was interrupted and its effect is not known"
                    )

            fault = self._fault_at(FaultPoint.BEFORE_STEP, index)
            if fault is not None and fault.interrupts:
                self._ledger.record(
                    capsule_id,
                    step.idempotency_key,
                    StepOutcome(OutcomeKind.INTERRUPTED, EffectState.NOT_APPLIED),
                )
                raise self._fail(
                    f"the step {step.step_id!r} was interrupted before it ran ({fault.value})"
                )

            # The validity window is checked before every step, and not only
            # at the boundaries. A step that lands an effect after the window
            # closed is the case the window exists to prevent.
            now = self._now()
            if self._capsule.expires_at_ms <= now:
                reason = "the capsule's validity window closed while the plan was running"
                # Nothing has landed yet, so the transaction failed. Something
                # has landed, so the transaction has to undo it before it can
                # report anything.
                if not self._environment.applied_steps:
                    raise self._fail(reason)
                self._compensate(reason)

            fault = self._fault_at(FaultPoint.DURING_STEP, index)
            if fault in (Fault.CRASH, Fault.DISK_FULL):
                # The machine stopped, or the write could not complete.
                # Whether the effect landed is not known, and an effect that is
                # not known cannot be undone automatically. A write that fails
                # may have been partially applied, and "partially applied" is
                # not a state the ledger can describe.
                self._ledger.record(
                    capsule_id,
                    step.idempotency_key,
                    StepOutcome(OutcomeKind.INTERRUPTED, EffectState.UNKNOWN),
                )
                self._compensate(
                    f"the step {step.step_id!r} was interrupted and its effect is not known"
                )
            if fault is not None:
                # The step ran and its effect landed; the interruption came
                # after it.
                self._environment.apply_step(step.step_id)
                self._ledger.record(
                    capsule_id,
                    step.idempotency_key,
                    StepOutcome(OutcomeKind.INTERRUPTED, EffectState.APPLIED),
                )
                self._compensate(
                    f"the step {step.step_id!r} was interrupted after its effect landed "
                    f"({fault.value})"
                )

            self._environment.apply_step(step.step_id)
            if not all(
                self._environment.postcondition_holds(predicate)
                for predicate in step.success_predicates
            ):
                self._ledger.record(
                    capsule_id, step.idempotency_key, StepOutcome(OutcomeKind.FAILED)
                )
                self._compensate(
                    f"the step {step.step_id!r} did not meet its success predicates"
                )
            self._ledger.record(
                capsule_id, step.idempotency_key, StepOutcome(OutcomeKind.SUCCEEDED)
            )
            self._executed.append(step.step_id)
            # The step took the time it declared. The clock moves with the
            # work, which is what makes a validity window close *during* a
            # plan rather than only before it.
            self._environment.now_ms += step.timeout_ms

            fault = self._fault_at(FaultPoint.AFTER_STEP, index)
            if fault is not None and fault.interrupts:
                self._compensate(
                    f"the capsule was interrupted after the step {step.step_id!r} "
                    f"({fault.value})"
                )

        self._transition(State.VERIFYING)

    def _verify(self) -> None:
        """``VERIFYING → COMMITTED``: the outcome is proven, or it is not reported."""
        fault = self._fault_at(FaultPoint.BEFORE_VERIFICATION)
        if fault is not None and fault.interrupts:
            self._compensate(f"the capsule was interrupted before verification ({fault.value})")
        for step in self._capsule.steps:
            if self._ledger.lookup(self._capsule.capsule_id, step.idempotency_key) is None:
                self._compensate(f"the step {step.step_id!r} has no outcome")
        fault = self._fault_at(FaultPoint.DURING_VERIFICATION)
        if fault is not None and fault.interrupts:
            self._compensate(f"the verification was interrupted ({fault.value})")
        unmet = [
            predicate
            for predicate in self._capsule.mandatory_postconditions
            if not self._environment.postcondition_holds(predicate)
        ]
        if unmet:
            self._compensate(f"the mandatory postcondition {unmet} did not hold")
        if not self._environment.audit_sink_durable:
            self._compensate("the audit sink cannot record the evidence durably")
        # The last read of the clock, and the last look at the validity
        # window. A commit is the moment the operator stops watching, and it
        # is the last moment at which the transaction can refuse.
        now = self._now()
        if self._capsule.expires_at_ms <= now:
            self._compensate("the capsule's validity window closed before the commit")
        if (
            self._capsule.recovery_artifact_ref is not None
            and not self._environment.recovery_material_intact
        ):
            self._compensate("the recovery artifact was corrupted before the commit")
        self._transition(State.COMMITTED)

    # -- the ways a transaction ends ----------------------------------------

    def _compensate(self, reason: str) -> NoReturn:
        """Undoes what landed, or escalates to a human.

        Never returns. The transaction has ended either way, and a caller
        that carried on would be continuing a transaction whose outcome is
        already decided.
        """
        if not self._machine.permits(self._state, State.COMPENSATING):
            # The table does not allow compensation from here, so the
            # transaction stays where it is rather than claiming a result it
            # cannot prove.
            raise _Halt(self._state, reason)
        self._state = State.COMPENSATING
        self._history.append(State.COMPENSATING)

        if not self._capsule.automatically_recoverable:
            raise self._escalate(reason)
        # A step whose effect is not known cannot be undone: the executor does
        # not know whether there is anything to undo, and undoing the wrong
        # thing is worse than stopping.
        if self._has_unknown_effect():
            raise self._escalate(reason)
        # The recovery material is checked **here**, and not only at the
        # precheck. Material that was present before the steps ran may be
        # gone or corrupt now, and a compensation that claims to have undone
        # an effect without material to undo it with is a false claim.
        if self._capsule.recovery_artifact_ref is not None:
            if not self._environment.recovery_material_present:
                raise self._escalate(reason)
            if not self._environment.recovery_material_intact:
                raise self._escalate(reason)
        for step_id in self._environment.applied_steps:
            entry = self._registry.effective(self._operation_of(step_id))
            if entry is None or not entry.compensatable:
                raise self._escalate(reason)
        for step_id in reversed(self._environment.applied_steps):
            self._environment.compensate_step(step_id)
        if not self._environment.outstanding_effects:
            self._state = State.ROLLED_BACK
            self._history.append(State.ROLLED_BACK)
            raise _Halt(State.ROLLED_BACK, reason)
        self._state = State.PARTIAL
        self._history.append(State.PARTIAL)
        raise _Halt(State.PARTIAL, reason)

    def _escalate(self, reason: str) -> _Halt:
        """Automatic recovery cannot safely complete."""
        if self._machine.permits(self._state, State.NEEDS_HUMAN_RECOVERY):
            self._state = State.NEEDS_HUMAN_RECOVERY
            self._history.append(State.NEEDS_HUMAN_RECOVERY)
        return _Halt(
            self._state, f"{reason}; automatic recovery cannot safely complete"
        )

    def _reject(self, reason: str) -> _Halt:
        """The capsule was refused before it could act."""
        if self._machine.permits(self._state, State.REJECTED):
            self._state = State.REJECTED
            self._history.append(State.REJECTED)
        return _Halt(self._state, reason)

    def _expire(self, reason: str) -> _Halt:
        """The capsule's validity window closed."""
        if self._machine.permits(self._state, State.EXPIRED):
            self._state = State.EXPIRED
            self._history.append(State.EXPIRED)
        return _Halt(self._state, reason)

    def _fail(self, reason: str) -> _Halt:
        """The capsule failed before it mutated anything."""
        if self._machine.permits(self._state, State.FAILED):
            self._state = State.FAILED
            self._history.append(State.FAILED)
        return _Halt(self._state, reason)

    def _transition(self, destination: State) -> None:
        """Moves to a state the table allows, or ends where it is."""
        if not self._machine.permits(self._state, destination):
            raise _Halt(
                self._state,
                f"the state table does not allow {self._state.value} → {destination.value}",
            )
        self._state = destination
        self._history.append(destination)

    # -- the helpers --------------------------------------------------------

    def _apply_fault(self, fault: Fault) -> None:
        """Lets a fault change the simulated world.

        A fault is not only an event the transaction observes. Some of them
        are changes to the world the transaction is acting on, and a change
        has to be found by the transaction's own checks rather than announced
        to it: a clock that moves backwards does not announce itself, and
        neither does a recovery artefact that turns out to be corrupt.
        """
        if fault is Fault.CLOCK_ROLLBACK:
            # An hour is enough to reopen a validity window that had just
            # closed, and small enough that the budget does not notice.
            self._environment.now_ms -= 3_600_000
        elif fault is Fault.CORRUPTED_RECOVERY_ARTIFACT:
            self._environment.recovery_material_intact = False

    def _now(self) -> int:
        """The current time, if the clock can be trusted.

        Every read of the clock goes through here. A transaction that read the
        clock once and trusted it again later is a transaction whose expiry
        check can be defeated by moving the clock back.
        """
        now = self._environment.now_ms
        if self._highest_now_ms is not None and now < self._highest_now_ms:
            raise self._untrusted_clock("the clock moved backwards under the capsule")
        self._highest_now_ms = now
        return now

    def _untrusted_clock(self, reason: str) -> _Halt:
        """The transaction cannot trust the clock, so it stops.

        Nothing has landed, so the transaction failed. Something has landed,
        so the transaction has to undo it before it can report anything.
        """
        if not self._environment.applied_steps:
            raise self._fail(reason)
        raise self._compensate(reason)

    def _fault_at(self, point: FaultPoint, index: Optional[int] = None) -> Optional[Fault]:
        """The fault at a point, recording that the transaction reached it,
        and letting the fault change the world.

        A fault injected at a point the transaction never reaches is not a
        fault the transaction survived, and a test that counted it as one
        would be proving nothing.
        """
        fault = self._faults.fault_at(point, index)
        if fault is not None:
            self._fired.append((point, index))
            self._apply_fault(fault)
        return fault

    def _has_unknown_effect(self) -> bool:
        """Whether any step's effect is not known."""
        return any(
            outcome is not None
            and outcome.kind is OutcomeKind.INTERRUPTED
            and outcome.effect is EffectState.UNKNOWN
            for outcome in (
                self._ledger.lookup(self._capsule.capsule_id, step.idempotency_key)
                for step in self._capsule.steps
            )
        )

    def _operation_of(self, step_id: str) -> str:
        """The operation an applied step performed.

        The simulated environment records step identifiers; the registry is
        keyed by operation. A step the capsule knows is looked up by its
        identifier. One it does not know has its operation recovered from the
        part of the identifier before the first dot, which is how the
        simulator's own fixtures name their steps.
        """
        for step in self._capsule.steps:
            if step.step_id == step_id:
                return step.operation
        return step_id.split(".", 1)[0]

    def _outcome(self, final_state: State, reason: str) -> ExecutionOutcome:
        return ExecutionOutcome(
            final_state=final_state,
            reason=reason,
            steps_executed=tuple(self._executed),
            steps_reused=tuple(self._reused),
            history=tuple(self._history),
        )

    # -- what a test can see -----------------------------------------------

    @property
    def state(self) -> State:
        """The state the transaction is in."""
        return self._state

    @property
    def ledger(self) -> IdempotencyLedger:
        """What the executor knows about the steps it has attempted."""
        return self._ledger

    @property
    def environment(self) -> Environment:
        """The simulated system."""
        return self._environment

    @property
    def faults_fired(self) -> tuple[tuple[FaultPoint, Optional[int]], ...]:
        """The fault points the transaction actually reached."""
        return tuple(self._fired)
