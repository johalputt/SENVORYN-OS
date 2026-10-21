"""Executes the shared Change Capsule vectors against the Python reference model.

The Rust implementation in ``crates/senvoryn-capsule`` executes the same
file. A disagreement between the two about any state or any reason fails
both suites, and CI runs both. That is the mechanism that keeps the
simulator and its implementation from drifting into two authorities on the
same question — and it is the mechanism the SCP's shared vectors use.

The vector file is the contract: it says what the executor must do, in data.
A change in either implementation that changes an outcome has to change the
file too, which is what makes the change reviewable — and which makes the
*other* implementation fail.
"""

import copy
import json
import os
import unittest

from capsule_reference import (
    Approval,
    Authorization,
    ChangeCapsule,
    Environment,
    Executor,
    Fault,
    FaultInjector,
    FaultPoint,
    IdempotencyLedger,
    OperationRegistry,
    State,
    load_capsule,
    load_registry,
    load_state_machine,
    read_decision_record,
)

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
VECTORS_PATH = os.path.join(ROOT, "conformance", "capsule-vectors.json")

#: The digest the base capsule carries.
BASE_DIGEST = "0000000000000000000000000000000000000000000000000000000000000002"

#: A digest that is not the base capsule's.
OTHER_DIGEST = "0000000000000000000000000000000000000000000000000000000000000009"

#: The version the base capsule expects its target to be at.
BASE_VERSION = "7"

#: A version the base capsule does not expect.
OTHER_VERSION = "6"


def _load(path):
    with open(path, encoding="utf-8") as handle:
        return json.load(handle)


VECTORS = _load(VECTORS_PATH)
BASE_CAPSULE = _load(os.path.join(ROOT, VECTORS["base_capsule"]))


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


def _base_environment():
    """The environment the base capsule runs in."""
    return Environment(
        current_version="7",
        policy_epoch=7,
        now_ms=1_767_225_600_000,
        satisfied_preconditions={"resource_version_matches"},
        observed_versions={"resource_version_matches": "7"},
        satisfied_postconditions={"write_accepted", "written_version_verified"},
        recovery_material_present=True,
        recovery_material_intact=True,
        audit_sink_durable=True,
    )


def _apply_environment(environment, overrides):
    """Applies the environment overrides of a case."""
    for key, value in (overrides or {}).items():
        if key == "current_version":
            environment.current_version = value
        elif key == "policy_epoch":
            environment.policy_epoch = value
        elif key == "now_ms":
            environment.now_ms = value
        elif key == "recovery_material_present":
            environment.recovery_material_present = value
        elif key == "recovery_material_intact":
            environment.recovery_material_intact = value
        elif key == "audit_sink_durable":
            environment.audit_sink_durable = value
        elif key == "satisfied_preconditions":
            environment.satisfied_preconditions = set(value)
        elif key == "satisfied_postconditions":
            environment.satisfied_postconditions = set(value)
        else:
            raise AssertionError(f"{key!r} is not an environment field")


def _decision_record(specification):
    """Builds the decision record a case is decided under.

    The record is built as a document and read back through the reference
    model's own reader, so every case exercises the reader as well as the
    executor. The executor is handed the authorization read out of that
    record, not a value the harness assembles.
    """
    digest = (specification or {}).get("digest", "matching")
    if digest == "matching":
        permitted = BASE_DIGEST
    elif digest == "other":
        permitted = OTHER_DIGEST
    elif digest == "none":
        permitted = None
    else:
        raise AssertionError(f"{digest!r} is not a digest selector")
    version = (specification or {}).get("version", "matching")
    if version == "matching":
        permitted_version = BASE_VERSION
    elif version == "other":
        permitted_version = OTHER_VERSION
    elif version == "none":
        permitted_version = None
    else:
        raise AssertionError(f"{version!r} is not a version selector")
    status = (specification or {}).get("status", "PERMIT")
    if status not in {"PERMIT", "DENY", "CHALLENGE", "DEFER", "QUARANTINE"}:
        raise AssertionError(f"{status!r} is not a decision status")
    return read_decision_record(
        {
            "schema": "senvoryn.decision-record.v1",
            "decision_id": "decision-0001",
            "request_id": "request-0001",
            "status": status,
            "reason_codes": ["ALL_POLICY_GATES_PASSED"],
            "policy_epoch": 7,
            "evaluated_at": "2026-01-01T00:00:00Z",
            "target_resource_id": "file:///srv/data/report.csv",
            "target_resource_version": permitted_version,
            "permitted_payload_digest": permitted,
            "capability_scope_ref": "capability-0001",
            "valid_until": None,
            "required_next_step": None,
            "effect_performed": False,
            "audit_ref": "audit-0001",
        }
    )


def _authorization(specification):
    """Builds the authorization a case is executed under.

    A case that says nothing about the decision is judged under the decision
    the sample capsule needs: the plan, the version and the one approval the
    sample capsule requires. A case that says something about the decision
    says everything about it, because a case that changed the approval while
    testing the digest would pass for the wrong reason.
    """
    if specification is None:
        approvals = (Approval("USER_CONFIRMATION", BASE_DIGEST),)
    else:
        approvals = []
        for entry in specification.get("approvals", []):
            binding = entry.get("binds")
            if binding == "capsule":
                binds = BASE_DIGEST
            elif binding == "other":
                binds = OTHER_DIGEST
            else:
                raise AssertionError(f"{binding!r} is not a binding selector")
            approvals.append(Approval(entry["type"], binds))
    return Authorization.from_record(_decision_record(specification), tuple(approvals))


def _fault_injector(specification):
    """Builds the fault injector of a case."""
    if specification is None:
        return None
    point = FaultPoint[specification["point"]]
    index = specification.get("index")
    return FaultInjector.at(point, Fault[specification["fault"]], index)


def _run_case(case, registry, machine):
    """Runs one case and returns the outcome."""
    document = copy.deepcopy(BASE_CAPSULE)
    for path, value in (case.get("capsule_overrides") or {}).items():
        _set_path(document, path, copy.deepcopy(value))
    capsule = load_capsule(document)

    environment = _base_environment()
    _apply_environment(environment, case.get("environment_overrides"))

    executor = Executor(
        capsule,
        registry,
        environment,
        machine=machine,
        ledger=IdempotencyLedger(),
        faults=_fault_injector(case.get("fault")),
    )
    return executor.run(_authorization(case.get("decision")))


class CapsuleVectorTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.registry = load_registry()
        cls.machine = load_state_machine()

    def test_every_vector_reaches_the_state_it_names(self):
        for case in VECTORS["cases"]:
            with self.subTest(case=case["id"]):
                outcome = _run_case(case, self.registry, self.machine)
                self.assertEqual(
                    outcome.final_state,
                    State(case["expected_state"]),
                    f"{case['id']} produced {outcome.final_state.value} / {outcome.reason} "
                    f"(the case says: {case.get('control')})",
                )
                expected_reason = case.get("expected_reason_contains")
                if expected_reason is not None:
                    self.assertIn(
                        expected_reason,
                        outcome.reason,
                        f"{case['id']} produced the wrong reason",
                    )

    def test_no_vector_reports_a_commit_it_did_not_earn(self):
        # The property the whole gate exists for, stated once more over the
        # vectors: only the case that expects a commit may report one.
        for case in VECTORS["cases"]:
            with self.subTest(case=case["id"]):
                if case["expected_state"] == "COMMITTED":
                    continue
                outcome = _run_case(case, self.registry, self.machine)
                self.assertNotEqual(
                    outcome.final_state,
                    State.COMMITTED,
                    f"{case['id']} does not expect a commit and the transaction reported one",
                )

    def test_every_vector_names_the_control_it_covers(self):
        for case in VECTORS["cases"]:
            with self.subTest(case=case["id"]):
                self.assertTrue(case.get("control"), "a case names no control")
                self.assertIn(case["expected_state"], {state.value for state in State})
                self.assertTrue(
                    case.get("expected_reason_contains"),
                    "a case names no reason, so nothing pins the wording",
                )

    def test_the_vector_file_is_large_enough_to_be_a_contract(self):
        """The count is checked here and in the Rust suite.

        That duplication is deliberate: the file is shared, so a case
        removed from it must fail in both implementations rather than in
        neither.
        """
        self.assertGreaterEqual(len(VECTORS["cases"]), 30)
        identifiers = [case["id"] for case in VECTORS["cases"]]
        self.assertEqual(len(identifiers), len(set(identifiers)), "a case identifier repeats")

    def test_the_two_implementations_agree_on_every_case(self):
        """The state table is the authority, and both implementations read it.

        If the two ever read different tables, the agreement they claim
        would be an agreement about two different machines. This checks that
        the table the Python model reads is the table in the repository.
        """
        with open(
            os.path.join(ROOT, "state-machines", "change-capsule-transitions.json"),
            encoding="utf-8",
        ) as handle:
            document = json.load(handle)
        self.assertEqual(self.machine.initial, State(document["initial_state"]))
        self.assertEqual(
            {state.value for state in self.machine.states},
            set(document["states"]),
        )
        for entry in document["transitions"]:
            source = State(entry["from"])
            destination = State(entry["to"])
            self.assertTrue(
                self.machine.permits(source, destination),
                f"the table lists {source.value} → {destination.value} and the model does not",
            )
            self.assertIsNotNone(self.machine.guard(source, destination))


if __name__ == "__main__":
    unittest.main()
