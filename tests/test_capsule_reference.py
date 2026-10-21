"""Unit tests for the Python Change Capsule reference model.

The shared vectors in ``tests/test_capsule_vectors.py`` cover the executor's
decisions. These cover the parts the vectors cannot reach: the timestamp
boundary, the registry's merge rule, the ledger's retry rule, and the state
table's authority — the controls that have to refuse, rather than the ones
that have to produce a particular answer.

The Rust crate has its own tests for the same controls, written against the
same contract. A control that is tested on one side only is a control the
other side might have got wrong.
"""

import copy
import json
import os
import unittest

from capsule_reference import (
    Approval,
    Authorization,
    CapsuleError,
    EffectState,
    Environment,
    Executor,
    Fault,
    FaultInjector,
    FaultPoint,
    IdempotencyLedger,
    OperationEntry,
    OperationRegistry,
    OutcomeKind,
    State,
    StateMachine,
    StepOutcome,
    load_capsule,
    load_registry,
    load_state_machine,
    parse_utc_millis,
    read_decision_record,
)

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

BASE_CAPSULE_PATH = os.path.join(ROOT, "examples", "capsule.sample.json")
TABLE_PATH = os.path.join(ROOT, "state-machines", "change-capsule-transitions.json")

BASE_DIGEST = "0000000000000000000000000000000000000000000000000000000000000002"


def _load(path):
    with open(path, encoding="utf-8") as handle:
        return json.load(handle)


def _set_path(document, path, value):
    """Sets a value at a dotted path, where a numeric part is an index."""
    parts = path.split(".")
    current = document
    for part in parts[:-1]:
        current = current[int(part)] if isinstance(current, list) else current[part]
    last = parts[-1]
    if isinstance(current, list):
        current[int(last)] = value
    else:
        current[last] = value


def _base_capsule():
    return load_capsule(_load(BASE_CAPSULE_PATH))


def _base_environment():
    return Environment(
        current_version="7",
        policy_epoch=7,
        now_ms=1_767_225_600_000,
        satisfied_preconditions={"resource_version_matches"},
        observed_versions={"resource_version_matches": "7"},
        satisfied_postconditions={"write_accepted", "written_version_verified"},
    )


def _decision_record_document(
    permitted_digest=BASE_DIGEST, permitted_resource_version="7", status="PERMIT"
):
    """The document a decision record is."""
    return {
        "schema": "senvoryn.decision-record.v1",
        "decision_id": "decision-0001",
        "request_id": "request-0001",
        "status": status,
        "reason_codes": ["ALL_POLICY_GATES_PASSED"],
        "policy_epoch": 7,
        "evaluated_at": "2026-01-01T00:00:00Z",
        "target_resource_id": "file:///srv/data/report.csv",
        "target_resource_version": permitted_resource_version,
        "permitted_payload_digest": permitted_digest,
        "capability_scope_ref": "capability-0001",
        "valid_until": None,
        "required_next_step": None,
        "effect_performed": False,
        "audit_ref": "audit-0001",
    }


def _decision_record(permitted_digest=BASE_DIGEST, permitted_resource_version="7", status="PERMIT"):
    """Builds a decision record and reads it back through the model's reader."""
    return read_decision_record(
        _decision_record_document(permitted_digest, permitted_resource_version, status)
    )


def _authorization():
    """The authorization that permits the sample capsule.

    It is read out of a decision record, the way the executor reads it in
    production: the plan and the version come from the decision, not from
    the value the test assembled.
    """
    return Authorization.from_record(
        _decision_record(), (Approval("USER_CONFIRMATION", BASE_DIGEST),)
    )


class TimestampTests(unittest.TestCase):
    """The timestamp boundary.

    Each guard is pinned by its own test, and each test names the guard it
    pins. The parser has several ways to say no to a malformed timestamp,
    and a test that only says "some refusal happened" cannot tell which of
    them is load-bearing. Where a case could be refused by two guards, the
    case here is chosen so that only one of them can refuse it.
    """

    def test_a_utc_timestamp_is_the_expected_number_of_milliseconds(self):
        self.assertEqual(parse_utc_millis("1970-01-01T00:00:00Z"), 0)
        self.assertEqual(parse_utc_millis("1970-01-01T00:00:01Z"), 1_000)
        self.assertEqual(parse_utc_millis("1970-01-01T00:00:01.001Z"), 1_001)
        self.assertEqual(parse_utc_millis("2026-01-01T00:00:00Z"), 1_767_225_600_000)

    def test_a_leap_day_in_a_leap_year_is_a_real_instant(self):
        # A refusal is reported as a failure with the guard's own words.
        # `assertEqual` never reaches its message when the parser raises,
        # and a test that cannot say which guard stopped the timestamp has
        # not tested the guard.
        for text, expected in (
            ("2024-02-29T00:00:00Z", 1_709_164_800_000),
            ("2000-02-29T00:00:00Z", 951_782_400_000),
        ):
            with self.subTest(text=text):
                try:
                    value = parse_utc_millis(text)
                except CapsuleError as error:
                    self.fail(
                        "a leap day in a leap year is a real instant, "
                        f"but it was refused: {error}"
                    )
                self.assertEqual(
                    value, expected, "a leap day in a leap year is a real instant"
                )
        self.assertEqual(parse_utc_millis("2024-03-01T00:00:00Z"), 1_709_251_200_000)

    def test_a_timestamp_with_an_offset_is_refused_rather_than_converted(self):
        for text in (
            "2026-01-01T00:00:00+00:00",
            "2026-01-01T05:30:00+05:30",
            "2026-01-01T00:00:00-08:00",
        ):
            with self.subTest(text=text):
                with self.assertRaises(CapsuleError):
                    parse_utc_millis(text)

    def test_a_timestamp_that_is_not_in_utc_is_refused(self):
        # A lower-case zone marker. Everything else about this timestamp is in
        # the strict form, so the zone marker is the only guard that can
        # refuse it -- which is what makes the guard testable.
        for text in ("2026-01-01T00:00:00z",):
            with self.subTest(text=text):
                with self.assertRaises(
                    CapsuleError,
                    msg="a timestamp that is not in UTC is refused rather than converted",
                ):
                    parse_utc_millis(text)

    def test_a_malformed_fraction_is_refused(self):
        # A fourth fractional digit is beyond the millisecond the profile
        # carries, so the timestamp is not in the form the profile accepts.
        for text in ("1970-01-01T00:00:01.1239Z",):
            with self.subTest(text=text):
                with self.assertRaises(
                    CapsuleError,
                    msg="a malformed fraction is refused: three digits at most",
                ):
                    parse_utc_millis(text)

    def test_a_date_that_does_not_exist_is_refused_rather_than_rolled_over(self):
        for text in (
            "2026-02-30T00:00:00Z",
            "2026-02-29T00:00:00Z",
            "2026-04-31T00:00:00Z",
            "1900-02-29T00:00:00Z",
            "2026-01-00T00:00:00Z",
            "2026-13-01T00:00:00Z",
        ):
            with self.subTest(text=text):
                with self.assertRaises(
                    CapsuleError,
                    msg="a date that does not exist is refused rather than rolled over",
                ):
                    parse_utc_millis(text)

    def test_a_time_that_does_not_exist_is_refused(self):
        for text in (
            "2026-01-01T24:00:00Z",
            "2026-01-01T00:00:60Z",
        ):
            with self.subTest(text=text):
                with self.assertRaises(
                    CapsuleError,
                    msg="a time that does not exist is refused",
                ):
                    parse_utc_millis(text)

    def test_a_year_before_one_is_refused(self):
        for text in ("0000-01-01T00:00:00Z",):
            with self.subTest(text=text):
                with self.assertRaises(
                    CapsuleError,
                    msg="the year must be one or later",
                ):
                    parse_utc_millis(text)

    def test_a_timestamp_that_is_not_in_the_strict_form_is_refused(self):
        for text in (
            "2026-01-01T00:00:00",
            "2026-01-01 00:00:00Z",
            "2026-01-01",
            "26-01-01T00:00:00Z",
            "",
        ):
            with self.subTest(text=text):
                with self.assertRaises(
                    CapsuleError,
                    msg="the strict form is one form",
                ):
                    parse_utc_millis(text)


class CapsuleReaderTests(unittest.TestCase):
    def test_a_capsule_without_an_expected_precondition_is_rejected(self):
        document = _load(BASE_CAPSULE_PATH)
        del document["preconditions"][0]["expected"]
        with self.assertRaises(CapsuleError):
            load_capsule(document)

    def test_a_capsule_with_a_non_utc_timestamp_is_rejected(self):
        document = _load(BASE_CAPSULE_PATH)
        document["expires_at"] = "2026-01-01T01:00:00+00:00"
        with self.assertRaises(CapsuleError):
            load_capsule(document)

    def test_the_capsule_keeps_the_wire_form_of_its_timestamps(self):
        capsule = _base_capsule()
        self.assertEqual(capsule.expires_at, "2026-01-01T01:00:00Z")
        self.assertEqual(capsule.expires_at_ms, 1_767_229_200_000)


class DecisionRecordTests(unittest.TestCase):
    """The decision record, read.

    The reader is a boundary, so it is closed: a record carrying a field it
    does not define is refused rather than read, and a record that claims an
    effect was performed is refused because the policy layer cannot have
    performed one.
    """

    def test_a_record_the_policy_layer_wrote_is_read_back(self):
        record = read_decision_record(_decision_record_document())
        self.assertEqual(record.status, "PERMIT")
        self.assertEqual(record.request_id, "request-0001")
        self.assertEqual(record.policy_epoch, 7)
        self.assertEqual(record.target_resource_version, "7")
        self.assertEqual(record.permitted_payload_digest, BASE_DIGEST)
        self.assertEqual(record.audit_ref, "audit-0001")

    def test_a_record_with_a_field_the_record_does_not_define_is_refused(self):
        document = _decision_record_document()
        document["effect_confirmed"] = True
        with self.assertRaises(ValueError):
            read_decision_record(document)

    def test_a_record_that_claims_an_effect_was_performed_is_refused(self):
        document = _decision_record_document()
        document["effect_performed"] = True
        with self.assertRaises(ValueError):
            read_decision_record(document)

    def test_a_record_with_another_schema_is_refused(self):
        document = _decision_record_document()
        document["schema"] = "senvoryn.decision-record.v2"
        with self.assertRaises(ValueError):
            read_decision_record(document)


class RegistryTests(unittest.TestCase):
    def test_the_registry_takes_the_most_restrictive_rule(self):
        lax = OperationEntry(
            operation_id="write",
            risk_tier="R0",
            reversibility="REVERSIBLE",
            approval_requirements=frozenset({"NONE"}),
            allowed_modes=frozenset({"NORMAL", "MAINTENANCE"}),
            resource_types=frozenset({"file_object"}),
            required_preconditions=frozenset(),
            required_postconditions=frozenset(),
            external_effects=frozenset(),
            idempotent=True,
            max_duration_ms=60_000,
            max_operation_count=5,
            audit_class="STANDARD",
        )
        strict = OperationEntry(
            operation_id="write",
            risk_tier="R3",
            reversibility="NON_REVERSIBLE",
            approval_requirements=frozenset({"STRONG_AUTH"}),
            allowed_modes=frozenset({"NORMAL"}),
            resource_types=frozenset({"file_object", "data_object"}),
            required_preconditions=frozenset({"resource_version_matches"}),
            required_postconditions=frozenset({"written_version_verified"}),
            external_effects=frozenset({"persistent_write"}),
            idempotent=False,
            max_duration_ms=1_000,
            max_operation_count=1,
            audit_class="SECURITY_CRITICAL",
        )
        registry = OperationRegistry([lax, strict])
        merged = registry.effective("write")

        self.assertIsNotNone(merged)
        # The most restrictive wins on every axis.
        self.assertEqual(merged.risk_tier, "R3")
        self.assertEqual(
            merged.reversibility, "NON_REVERSIBLE", "the least reversible rule wins"
        )
        self.assertEqual(merged.allowed_modes, frozenset({"NORMAL"}))
        self.assertEqual(
            merged.approval_requirements, frozenset({"NONE", "STRONG_AUTH"})
        )
        self.assertEqual(merged.max_duration_ms, 1_000)
        self.assertEqual(merged.max_operation_count, 1)
        self.assertEqual(merged.audit_class, "SECURITY_CRITICAL")
        self.assertFalse(merged.idempotent)
        self.assertFalse(merged.compensatable)

    def test_an_operation_the_registry_does_not_contain_has_no_rule(self):
        self.assertIsNone(load_registry().effective("teleport"))


class LedgerTests(unittest.TestCase):
    def test_only_an_outcome_known_not_to_have_applied_may_be_retried(self):
        # An effect that is not known may have landed, and repeating a
        # non-idempotent operation would apply it twice.
        self.assertTrue(
            StepOutcome(OutcomeKind.INTERRUPTED, EffectState.NOT_APPLIED).may_retry,
            "an operation that definitely did not run may be attempted again",
        )
        for outcome in (
            StepOutcome(OutcomeKind.INTERRUPTED, EffectState.UNKNOWN),
            StepOutcome(OutcomeKind.INTERRUPTED, EffectState.APPLIED),
            StepOutcome(OutcomeKind.SUCCEEDED),
            StepOutcome(OutcomeKind.FAILED),
        ):
            with self.subTest(outcome=outcome):
                self.assertFalse(outcome.may_retry)

    def test_the_ledger_remembers_what_an_attempt_did(self):
        ledger = IdempotencyLedger()
        self.assertIsNone(ledger.lookup("capsule:one", "idem:one"))
        ledger.record("capsule:one", "idem:one", StepOutcome(OutcomeKind.SUCCEEDED))
        self.assertIsNotNone(ledger.lookup("capsule:one", "idem:one"))
        # A different capsule's step is a different step.
        self.assertIsNone(ledger.lookup("capsule:two", "idem:one"))


class StateMachineTests(unittest.TestCase):
    def test_the_table_refuses_a_transition_it_does_not_contain(self):
        machine = load_state_machine()
        self.assertTrue(machine.permits(State.DRAFT, State.VALIDATED))
        # No transition skips VALIDATED or AUTHORIZED.
        self.assertFalse(
            machine.permits(State.DRAFT, State.APPLYING),
            "the table allowed a transition it does not contain",
        )
        self.assertFalse(
            machine.permits(State.DRAFT, State.COMMITTED),
            "the table allowed a transition it does not contain",
        )
        self.assertFalse(
            machine.permits(State.VALIDATED, State.STAGED),
            "the table allowed a transition it does not contain",
        )
        self.assertIsNone(machine.guard(State.DRAFT, State.APPLYING))

    def test_the_executor_only_enters_states_the_table_allows(self):
        machine = load_state_machine()
        executor = Executor(
            _base_capsule(), load_registry(), _base_environment(), machine=machine
        )
        outcome = executor.run(_authorization())
        self.assertEqual(outcome.final_state, State.COMMITTED)
        for state in outcome.history:
            self.assertIn(state, machine.states)

    def test_a_table_without_the_first_transition_stops_the_transaction(self):
        # The table is the authority, so a table that does not contain the
        # transition means the transaction does not take it. The executor
        # does not second-guess the table.
        document = _load(TABLE_PATH)
        document["transitions"] = [
            entry
            for entry in document["transitions"]
            if not (entry["from"] == "DRAFT" and entry["to"] == "VALIDATED")
        ]
        machine = StateMachine(document)
        executor = Executor(
            _base_capsule(), load_registry(), _base_environment(), machine=machine
        )
        outcome = executor.run(_authorization())
        self.assertEqual(outcome.final_state, State.DRAFT)
        self.assertIn("the state table does not allow", outcome.reason)

    def test_every_state_in_the_table_can_be_entered(self):
        # A state that no transition enters is a hole in the machine.
        machine = load_state_machine()
        reachable = {machine.initial}
        changed = True
        while changed:
            changed = False
            for state in list(reachable):
                for successor in machine.successors(state):
                    if successor not in reachable:
                        reachable.add(successor)
                        changed = True
        for state in machine.states:
            with self.subTest(state=state.value):
                self.assertIn(state, reachable)


class ExecutorTests(unittest.TestCase):
    def test_a_capsule_that_passes_every_check_is_committed(self):
        executor = Executor(
            _base_capsule(), load_registry(), _base_environment(), machine=load_state_machine()
        )
        outcome = executor.run(_authorization())
        self.assertEqual(outcome.final_state, State.COMMITTED)
        self.assertEqual(outcome.steps_executed, ("step.write", "step.verify"))

    def test_a_crash_leaves_the_effect_unknown_and_escalates_to_a_human(self):
        executor = Executor(
            _base_capsule(),
            load_registry(),
            _base_environment(),
            machine=load_state_machine(),
            faults=FaultInjector.at(FaultPoint.DURING_STEP, Fault.CRASH, 0),
        )
        outcome = executor.run(_authorization())
        self.assertEqual(outcome.final_state, State.NEEDS_HUMAN_RECOVERY)
        self.assertIn("effect is not known", outcome.reason)

    def test_an_io_error_during_a_step_is_compensated(self):
        executor = Executor(
            _base_capsule(),
            load_registry(),
            _base_environment(),
            machine=load_state_machine(),
            faults=FaultInjector.at(FaultPoint.DURING_STEP, Fault.IO, 0),
        )
        outcome = executor.run(_authorization())
        self.assertEqual(outcome.final_state, State.ROLLED_BACK)
        self.assertEqual(outcome.steps_executed, ())

    def test_the_executor_records_the_fault_points_it_reached(self):
        # A fault injected at a point the transaction never reaches is not a
        # fault the transaction survived.
        executor = Executor(
            _base_capsule(),
            load_registry(),
            _base_environment(),
            machine=load_state_machine(),
            faults=FaultInjector.at(FaultPoint.BEFORE_STEP, Fault.IO, 5),
        )
        executor.run(_authorization())
        self.assertEqual(executor.faults_fired, ())

    def test_a_resumed_run_does_not_repeat_a_step_that_already_ran(self):
        # The first run is interrupted after the first step landed. The
        # second run must not repeat it.
        machine = load_state_machine()
        registry = load_registry()

        first = Executor(
            _base_capsule(),
            registry,
            _base_environment(),
            machine=machine,
            faults=FaultInjector.at(FaultPoint.AFTER_STEP, Fault.IO, 0),
        )
        outcome = first.run(_authorization())
        self.assertEqual(outcome.final_state, State.ROLLED_BACK)

        # The step that ran is known to have succeeded, so the second run
        # reuses it rather than running it again.
        capsule = _base_capsule()
        resumed = Executor(
            capsule,
            registry,
            _base_environment(),
            machine=machine,
            ledger=first.ledger,
        )
        outcome = resumed.run(_authorization())
        self.assertEqual(outcome.final_state, State.COMMITTED, outcome.reason)
        self.assertIn("step.write", outcome.steps_reused)
        self.assertNotIn("step.write", outcome.steps_executed)

    def test_the_decision_is_the_only_authority_on_whether_a_capsule_may_run(self):
        # A decision about a different plan permits nothing here, whatever
        # the capsule itself says.
        executor = Executor(
            _base_capsule(), load_registry(), _base_environment(), machine=load_state_machine()
        )
        outcome = executor.run(
            Authorization.from_record(
                _decision_record(
                    permitted_digest="0000000000000000000000000000000000000000000000000000000000000009"
                ),
                (Approval("USER_CONFIRMATION", BASE_DIGEST),),
            )
        )
        self.assertEqual(outcome.final_state, State.REJECTED)
        self.assertIn("does not permit the capsule", outcome.reason)

    def test_a_decision_made_against_another_version_permits_nothing(self):
        # The decision names the version of the target it was made against.
        # A decision made against an earlier version says nothing about the
        # one the executor is about to act on.
        executor = Executor(
            _base_capsule(), load_registry(), _base_environment(), machine=load_state_machine()
        )
        outcome = executor.run(
            Authorization.from_record(
                _decision_record(permitted_resource_version="6"),
                (Approval("USER_CONFIRMATION", BASE_DIGEST),),
            )
        )
        self.assertEqual(outcome.final_state, State.REJECTED)
        self.assertIn("was made against version", outcome.reason)

    def test_a_decision_that_names_no_version_permits_nothing(self):
        # A decision made before the resource was resolved names no version.
        # That is a refusal, not a wildcard.
        executor = Executor(
            _base_capsule(), load_registry(), _base_environment(), machine=load_state_machine()
        )
        outcome = executor.run(
            Authorization.from_record(
                _decision_record(permitted_resource_version=None),
                (Approval("USER_CONFIRMATION", BASE_DIGEST),),
            )
        )
        self.assertEqual(outcome.final_state, State.REJECTED)
        self.assertIn("does not name the version", outcome.reason)

    def test_a_decision_that_granted_no_authority_permits_nothing(self):
        # A decision that refused the request authorises nothing, whatever
        # plan and version it names.
        executor = Executor(
            _base_capsule(), load_registry(), _base_environment(), machine=load_state_machine()
        )
        outcome = executor.run(
            Authorization.from_record(
                _decision_record(status="DENY"), (Approval("USER_CONFIRMATION", BASE_DIGEST),)
            )
        )
        self.assertEqual(outcome.final_state, State.REJECTED)
        self.assertIn("granted no authority", outcome.reason)

    def test_a_decision_that_names_no_plan_permits_nothing(self):
        # A request that named no plan produced a decision about an
        # operation rather than about a plan. That is a refusal, not a
        # wildcard.
        executor = Executor(
            _base_capsule(), load_registry(), _base_environment(), machine=load_state_machine()
        )
        outcome = executor.run(
            Authorization.from_record(
                _decision_record(permitted_digest=None),
                (Approval("USER_CONFIRMATION", BASE_DIGEST),),
            )
        )
        self.assertEqual(outcome.final_state, State.REJECTED)
        self.assertIn("does not name the plan", outcome.reason)


class ValidationTests(unittest.TestCase):
    """The controls a capsule must satisfy before anything is staged.

    The shared vectors say what the outcome is. These say which rule produced
    it, so a change to one rule cannot be absorbed by a different one.
    """

    def _run(self, replacements):
        document = _load(BASE_CAPSULE_PATH)
        for path, value in replacements:
            _set_path(document, path, value)
        executor = Executor(
            load_capsule(document), load_registry(), _base_environment(), machine=load_state_machine()
        )
        return executor.run(_authorization())

    def test_a_capsule_whose_budget_outlasts_its_window_is_refused(self):
        # The capsule's own claims have to agree with one another. It says the
        # plan takes at most `budget.duration_ms`, and it says it is only valid
        # until `expires_at`. A plan whose own bound outlasts its own window
        # cannot finish inside the authority it was issued under, so the
        # capsule contradicts itself -- and a capsule that contradicts itself
        # is refused before anything is staged.
        outcome = self._run([("budget.duration_ms", 7_200_000)])
        self.assertIn(
            "does not fit inside its validity window",
            outcome.reason,
            "a capsule whose budget outlasts its window must be refused, and the reason must say so",
        )
        self.assertEqual(outcome.final_state, State.REJECTED)

    def test_a_capsule_whose_budget_exactly_fills_its_window_is_admitted(self):
        # The boundary of the check above: a budget that exactly fills the
        # window is not a budget that outlasts it. A capsule that asks for
        # every millisecond it was granted has not asked for too much.
        #
        # The window is shortened to match the registry's combined allowance,
        # so that the budget is limited by the window rather than by the
        # registry.
        outcome = self._run(
            [
                ("expires_at", "2026-01-01T00:01:30Z"),
                ("budget.duration_ms", 90_000),
            ]
        )
        self.assertEqual(
            outcome.final_state,
            State.COMMITTED,
            "a capsule whose budget exactly fills its window is admitted, "
            "and the reason must not refuse it",
        )

    def test_a_capsule_whose_budget_does_not_cover_its_steps_is_refused(self):
        # The budget is the plan's own bound. A plan that asks for more time
        # than its budget allows is a plan that does not fit inside the
        # authority it asked for.
        outcome = self._run([("budget.duration_ms", 20_000)])
        self.assertIn(
            "does not cover its steps",
            outcome.reason,
            "a capsule whose budget does not cover its steps must be refused, "
            "and the reason must say so",
        )
        self.assertEqual(outcome.final_state, State.REJECTED)

    def test_a_capsule_whose_budget_exceeds_the_registry_allowance_is_refused(self):
        # The registry bounds how long the operations a plan performs may
        # run, and the plan's budget cannot outlast that. The allowance is the
        # sum of the per-operation bounds the plan actually uses.
        outcome = self._run([("budget.duration_ms", 100_000)])
        self.assertIn(
            "exceeds the 90000ms its operations are allowed",
            outcome.reason,
            "a capsule whose budget exceeds the registry's allowance must be refused, "
            "and the reason must say so",
        )
        self.assertEqual(outcome.final_state, State.REJECTED)

    def test_a_step_may_not_run_longer_than_the_registry_allows(self):
        # The registry's `max_duration_ms` bounds one invocation of one
        # operation. It is not the capsule's budget, and it is not the plan's
        # total.
        outcome = self._run([("steps.0.timeout_ms", 90_000)])
        self.assertIn(
            "but the registry allows 60000ms",
            outcome.reason,
            "a step longer than the registry allows must be refused, and the reason must say so",
        )
        self.assertEqual(outcome.final_state, State.REJECTED)

    def test_the_registry_counts_each_operation_and_the_capsule_counts_the_plan(self):
        # The registry's `max_operation_count` says how many times *one*
        # operation may be performed. The capsule's `budget.operation_count`
        # says how many operations the *plan* may perform. Neither number
        # means the other one's thing, so the two are not compared to each
        # other.
        outcome = self._run(
            [("steps.1.operation", "write"), ("steps.1.risk_tier", "R2")]
        )
        self.assertIn(
            "but the registry allows 1",
            outcome.reason,
            "an operation performed more often than the registry allows must be refused, "
            "and the reason must say so",
        )
        self.assertEqual(outcome.final_state, State.REJECTED)

        outcome = self._run([("budget.operation_count", 1)])
        self.assertIn(
            "the capsule's budget covers 1 operation(s) but the plan has 2 step(s)",
            outcome.reason,
            "a plan with more steps than the capsule's budget covers must be refused, "
            "and the reason must say so",
        )
        self.assertEqual(outcome.final_state, State.REJECTED)


class WorldChangeTests(unittest.TestCase):
    """The controls that have to notice the world moving under the transaction.

    A fault that interrupts the transaction is easy to reason about: it ends
    the transaction. A fault that changes the world is not, because the
    transaction has to find the change for itself, and a change it did not
    find is a change it acted on.
    """

    def test_a_full_disk_leaves_the_effect_unknown(self):
        # A write that could not complete is not a write that did not happen:
        # it may have been partially applied. "Partially applied" is not a
        # state the ledger can describe, so the effect is unknown.
        executor = Executor(
            _base_capsule(),
            load_registry(),
            _base_environment(),
            machine=load_state_machine(),
            faults=FaultInjector.at(FaultPoint.DURING_STEP, Fault.DISK_FULL, 0),
        )
        outcome = executor.run(_authorization())
        self.assertIn(
            "effect is not known",
            outcome.reason,
            "a full disk must leave the effect unknown, and the reason must say so",
        )
        self.assertEqual(outcome.final_state, State.NEEDS_HUMAN_RECOVERY)
        recorded = executor.ledger.lookup(
            "capsule:example:0001", "idem:capsule:example:0001:step.write"
        )
        self.assertIsNotNone(recorded)
        self.assertIs(recorded.effect, EffectState.UNKNOWN)

    def test_a_clock_that_moves_backwards_is_not_trusted(self):
        executor = Executor(
            _base_capsule(),
            load_registry(),
            _base_environment(),
            machine=load_state_machine(),
            faults=FaultInjector.at(FaultPoint.BEFORE_PRECHECK, Fault.CLOCK_ROLLBACK),
        )
        outcome = executor.run(_authorization())
        self.assertIn(
            "clock moved backwards",
            outcome.reason,
            "a clock that moves backwards must not be trusted, and the reason must say so",
        )
        self.assertEqual(outcome.final_state, State.FAILED)

    def test_a_capsule_cannot_be_executed_before_it_was_issued(self):
        # The only check that catches a clock that moved backwards before the
        # transaction had observed it at all: a capsule did not exist before it
        # was issued.
        environment = _base_environment()
        environment.now_ms = 1_767_225_600_000 - 1_000
        executor = Executor(
            _base_capsule(), load_registry(), environment, machine=load_state_machine()
        )
        outcome = executor.run(_authorization())
        self.assertIn(
            "not been issued yet",
            outcome.reason,
            "a capsule cannot be executed before it was issued, and the reason must say so",
        )
        self.assertEqual(outcome.final_state, State.REJECTED)

    def test_a_recovery_artifact_that_is_present_but_corrupt_is_not_recovery_material(self):
        # Presence is not integrity. Material that is present but corrupt is
        # worse than material that is missing, because a missing artefact
        # stops the transaction and a corrupt one is used.
        environment = _base_environment()
        environment.recovery_material_intact = False
        executor = Executor(
            _base_capsule(), load_registry(), environment, machine=load_state_machine()
        )
        outcome = executor.run(_authorization())
        self.assertIn(
            "recovery artifact is corrupt",
            outcome.reason,
            "a corrupt recovery artifact is not recovery material, and the reason must say so",
        )
        self.assertEqual(outcome.final_state, State.FAILED)
        self.assertEqual(environment.applied_steps, [])

    def test_recovery_material_that_becomes_corrupt_cannot_undo_the_effect(self):
        executor = Executor(
            _base_capsule(),
            load_registry(),
            _base_environment(),
            machine=load_state_machine(),
            faults=FaultInjector.at(
                FaultPoint.DURING_STEP, Fault.CORRUPTED_RECOVERY_ARTIFACT, 0
            ),
        )
        outcome = executor.run(_authorization())
        self.assertIn(
            "automatic recovery cannot safely complete",
            outcome.reason,
            "a compensation needs intact recovery material, and the reason must say so",
        )
        self.assertEqual(outcome.final_state, State.NEEDS_HUMAN_RECOVERY)

    def test_a_capsule_whose_window_closes_during_the_plan_does_not_land_the_next_effect(self):
        # The window is checked before every step, not only at the boundaries.
        # The capsule is entitled to an hour, and the plan starts thirty
        # seconds before that hour is over. A capsule that waited in a queue
        # looks exactly like this.
        environment = _base_environment()
        environment.now_ms = 1_767_225_600_000 + 3_570_000
        executor = Executor(
            _base_capsule(),
            load_registry(),
            environment,
            machine=load_state_machine(),
        )
        outcome = executor.run(_authorization())
        self.assertIn(
            "validity window closed while the plan was running",
            outcome.reason,
            "a capsule whose window closes during the plan must not land the next effect, "
            "and the reason must say so",
        )
        self.assertEqual(outcome.final_state, State.ROLLED_BACK)
        self.assertEqual(outcome.steps_executed, ("step.write",))

    def test_no_fault_at_any_point_produces_a_commit(self):
        # The property the whole gate exists for, over the whole fault space
        # including the faults that change the world rather than interrupting
        # the transaction.
        machine = load_state_machine()
        registry = load_registry()
        points = [
            (FaultPoint.BEFORE_VALIDATION, None),
            (FaultPoint.BEFORE_AUTHORIZATION, None),
            (FaultPoint.BEFORE_STAGING, None),
            (FaultPoint.AFTER_STAGING, None),
            (FaultPoint.BEFORE_PRECHECK, None),
            (FaultPoint.BEFORE_VERIFICATION, None),
            (FaultPoint.DURING_VERIFICATION, None),
        ]
        for index in (0, 1):
            points.extend(
                [
                    (FaultPoint.BEFORE_STEP, index),
                    (FaultPoint.DURING_STEP, index),
                    (FaultPoint.AFTER_STEP, index),
                ]
            )
        kinds = [
            Fault.IO,
            Fault.CRASH,
            Fault.STALE_PRECONDITION,
            Fault.POLICY_EPOCH_CHANGED,
            Fault.AUDIT_SINK_LOST,
            Fault.DISK_FULL,
            Fault.CLOCK_ROLLBACK,
            Fault.CORRUPTED_RECOVERY_ARTIFACT,
        ]
        checked = 0
        for point, index in points:
            for kind in kinds:
                executor = Executor(
                    _base_capsule(),
                    registry,
                    _base_environment(),
                    machine=machine,
                    faults=FaultInjector.at(point, kind, index),
                )
                outcome = executor.run(_authorization())
                self.assertNotEqual(
                    outcome.final_state,
                    State.COMMITTED,
                    f"a {kind.value} at {point.value} was reported as COMMITTED: {outcome.reason}",
                )
                checked += 1
        self.assertGreaterEqual(checked, 100)


if __name__ == "__main__":
    unittest.main()
