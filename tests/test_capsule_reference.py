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
)

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

BASE_CAPSULE_PATH = os.path.join(ROOT, "examples", "capsule.sample.json")
TABLE_PATH = os.path.join(ROOT, "state-machines", "change-capsule-transitions.json")

BASE_DIGEST = "0000000000000000000000000000000000000000000000000000000000000002"


def _load(path):
    with open(path, encoding="utf-8") as handle:
        return json.load(handle)


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


def _authorization():
    return Authorization(
        permitted_digest=BASE_DIGEST,
        approvals=(Approval("USER_CONFIRMATION", BASE_DIGEST),),
    )


class TimestampTests(unittest.TestCase):
    def test_a_utc_timestamp_is_the_expected_number_of_milliseconds(self):
        self.assertEqual(parse_utc_millis("1970-01-01T00:00:00Z"), 0)
        self.assertEqual(parse_utc_millis("1970-01-01T00:00:01Z"), 1_000)
        self.assertEqual(parse_utc_millis("1970-01-01T00:00:01.001Z"), 1_001)
        self.assertEqual(parse_utc_millis("2024-02-29T00:00:00Z"), 1_709_164_800_000)
        self.assertEqual(parse_utc_millis("2026-01-01T00:00:00Z"), 1_767_225_600_000)

    def test_a_timestamp_with_an_offset_is_refused_rather_than_converted(self):
        for text in (
            "2026-01-01T00:00:00+00:00",
            "2026-01-01T05:30:00+05:30",
            "2026-01-01T00:00:00-08:00",
        ):
            with self.subTest(text=text):
                with self.assertRaises(CapsuleError):
                    parse_utc_millis(text)

    def test_a_date_that_does_not_exist_is_refused_rather_than_rolled_over(self):
        for text in (
            "2026-02-30T00:00:00Z",
            "2026-02-29T00:00:00Z",
            "2026-04-31T00:00:00Z",
            "1900-02-29T00:00:00Z",
            "2026-01-00T00:00:00Z",
            "2026-13-01T00:00:00Z",
            "0000-01-01T00:00:00Z",
        ):
            with self.subTest(text=text):
                with self.assertRaises(CapsuleError):
                    parse_utc_millis(text)

    def test_a_timestamp_that_is_not_in_the_strict_form_is_refused(self):
        for text in (
            "2026-01-01T00:00:00",
            "2026-01-01 00:00:00Z",
            "2026-01-01",
            "26-01-01T00:00:00Z",
            "2026-01-01T00:00:60Z",
            "2026-01-01T24:00:00Z",
            "2026-01-01T00:00:01.1239Z",
            "",
        ):
            with self.subTest(text=text):
                with self.assertRaises(CapsuleError):
                    parse_utc_millis(text)

    def test_a_leap_day_is_a_real_instant(self):
        self.assertEqual(parse_utc_millis("2000-02-29T00:00:00Z"), 951_782_400_000)
        self.assertEqual(parse_utc_millis("2024-03-01T00:00:00Z"), 1_709_251_200_000)


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
            Authorization(
                permitted_digest="0000000000000000000000000000000000000000000000000000000000000009",
                approvals=(Approval("USER_CONFIRMATION", BASE_DIGEST),),
            )
        )
        self.assertEqual(outcome.final_state, State.REJECTED)
        self.assertIn("does not permit the capsule", outcome.reason)


if __name__ == "__main__":
    unittest.main()
