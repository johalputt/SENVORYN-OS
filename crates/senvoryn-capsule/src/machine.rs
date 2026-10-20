//! The Change Capsule state machine, read from the repository's own table.
//!
//! The table in `state-machines/change-capsule-transitions.json` is the
//! authority. The executor does not decide which transitions are legal; it
//! asks this table, and the answer is what it does. That is the whole point of
//! keeping the table in the repository instead of in the code: a transition
//! that is not in the table does not exist, and the table is covered by the
//! checksum manifest.

use std::collections::{BTreeMap, BTreeSet};

use senvoryn_canonical::Value;

/// The states of the Change Capsule transaction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum State {
    /// The capsule is a plan and nothing more.
    Draft,
    /// The capsule passed structural validation.
    Validated,
    /// The capsule was permitted and its approvals bind its digest.
    Authorized,
    /// The artifacts are staged outside the active storage.
    Staged,
    /// The target state, epoch, resources and recovery materials were rechecked.
    Prechecked,
    /// The steps are running.
    Applying,
    /// The steps are done and their outcomes are being checked.
    Verifying,
    /// Every mandatory postcondition passed and the evidence is durable.
    Committed,
    /// The capsule was refused before it could act.
    Rejected,
    /// The capsule's validity window closed.
    Expired,
    /// The capsule was cancelled before it mutated anything.
    Cancelled,
    /// The capsule failed before it mutated anything.
    Failed,
    /// The capsule's effects are being undone.
    Compensating,
    /// The effects were undone and the postconditions were verified.
    RolledBack,
    /// Some effects remain, or the compensation was incomplete.
    Partial,
    /// Automatic recovery cannot safely complete.
    NeedsHumanRecovery,
}

impl State {
    /// The state as it appears in the state table.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "DRAFT",
            Self::Validated => "VALIDATED",
            Self::Authorized => "AUTHORIZED",
            Self::Staged => "STAGED",
            Self::Prechecked => "PRECHECKED",
            Self::Applying => "APPLYING",
            Self::Verifying => "VERIFYING",
            Self::Committed => "COMMITTED",
            Self::Rejected => "REJECTED",
            Self::Expired => "EXPIRED",
            Self::Cancelled => "CANCELLED",
            Self::Failed => "FAILED",
            Self::Compensating => "COMPENSATING",
            Self::RolledBack => "ROLLED_BACK",
            Self::Partial => "PARTIAL",
            Self::NeedsHumanRecovery => "NEEDS_HUMAN_RECOVERY",
        }
    }

    /// Reads a state from its serialized form.
    pub fn parse(text: &str) -> Option<Self> {
        match text {
            "DRAFT" => Some(Self::Draft),
            "VALIDATED" => Some(Self::Validated),
            "AUTHORIZED" => Some(Self::Authorized),
            "STAGED" => Some(Self::Staged),
            "PRECHECKED" => Some(Self::Prechecked),
            "APPLYING" => Some(Self::Applying),
            "VERIFYING" => Some(Self::Verifying),
            "COMMITTED" => Some(Self::Committed),
            "REJECTED" => Some(Self::Rejected),
            "EXPIRED" => Some(Self::Expired),
            "CANCELLED" => Some(Self::Cancelled),
            "FAILED" => Some(Self::Failed),
            "COMPENSATING" => Some(Self::Compensating),
            "ROLLED_BACK" => Some(Self::RolledBack),
            "PARTIAL" => Some(Self::Partial),
            "NEEDS_HUMAN_RECOVERY" => Some(Self::NeedsHumanRecovery),
            _ => None,
        }
    }

    /// Whether the transaction can end in this state.
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            Self::Committed
                | Self::Rejected
                | Self::Expired
                | Self::Cancelled
                | Self::Failed
                | Self::RolledBack
                | Self::Partial
                | Self::NeedsHumanRecovery
        )
    }
}

/// One allowed transition, with the guard that has to hold for it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Transition {
    /// The state the transition leaves.
    pub from: State,
    /// The state the transition enters.
    pub to: State,
    /// The name of the guard, as the table states it.
    pub guard: String,
}

/// Why a state table could not be read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MachineError {
    /// A required field was missing or malformed.
    Malformed {
        /// The field at fault.
        field: String,
        /// What was wrong with it.
        detail: String,
    },
    /// The table names a state that does not exist.
    UnknownState(String),
}

impl core::fmt::Display for MachineError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Malformed { field, detail } => {
                write!(f, "state table field {field:?} is malformed: {detail}")
            }
            Self::UnknownState(name) => write!(f, "{name:?} is not a state of the machine"),
        }
    }
}

impl std::error::Error for MachineError {}

/// The Change Capsule state table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransitionTable {
    /// The state the transaction starts in.
    pub initial: State,
    /// The states the transaction may end in.
    pub terminal: BTreeSet<State>,
    /// The allowed transitions, keyed by where they go.
    transitions: BTreeMap<(State, State), Transition>,
}

impl TransitionTable {
    /// Reads a table from a canonical document.
    pub fn from_value(document: &Value) -> Result<Self, MachineError> {
        let initial_text =
            document.get("initial_state").and_then(Value::as_str).ok_or_else(|| {
                MachineError::Malformed {
                    field: "initial_state".to_owned(),
                    detail: "absent or not a string".to_owned(),
                }
            })?;
        let initial = State::parse(initial_text)
            .ok_or_else(|| MachineError::UnknownState(initial_text.to_owned()))?;
        let terminal = document
            .get("terminal_states")
            .and_then(Value::as_array)
            .ok_or_else(|| MachineError::Malformed {
                field: "terminal_states".to_owned(),
                detail: "absent or not an array".to_owned(),
            })?
            .iter()
            .filter_map(Value::as_str)
            .map(|name| {
                State::parse(name).ok_or_else(|| MachineError::UnknownState(name.to_owned()))
            })
            .collect::<Result<BTreeSet<_>, _>>()?;

        let raw = document.get("transitions").and_then(Value::as_array).ok_or_else(|| {
            MachineError::Malformed {
                field: "transitions".to_owned(),
                detail: "absent or not an array".to_owned(),
            }
        })?;
        let mut transitions = BTreeMap::new();
        for entry in raw {
            let from_text = entry.get("from").and_then(Value::as_str).ok_or_else(|| {
                MachineError::Malformed {
                    field: "transitions[].from".to_owned(),
                    detail: "absent or not a string".to_owned(),
                }
            })?;
            let to_text =
                entry.get("to").and_then(Value::as_str).ok_or_else(|| MachineError::Malformed {
                    field: "transitions[].to".to_owned(),
                    detail: "absent or not a string".to_owned(),
                })?;
            let guard = entry.get("guard").and_then(Value::as_str).ok_or_else(|| {
                MachineError::Malformed {
                    field: "transitions[].guard".to_owned(),
                    detail: "absent or not a string".to_owned(),
                }
            })?;
            let from = State::parse(from_text)
                .ok_or_else(|| MachineError::UnknownState(from_text.to_owned()))?;
            let to = State::parse(to_text)
                .ok_or_else(|| MachineError::UnknownState(to_text.to_owned()))?;
            transitions.insert((from, to), Transition { from, to, guard: guard.to_owned() });
        }
        Ok(Self { initial, terminal, transitions })
    }

    /// The allowed transitions.
    pub fn transitions(&self) -> impl Iterator<Item = &Transition> {
        self.transitions.values()
    }

    /// The guard for a transition, if the table allows it.
    pub fn guard(&self, from: State, to: State) -> Option<&str> {
        self.transitions.get(&(from, to)).map(|transition| transition.guard.as_str())
    }

    /// Whether the table allows a transition.
    pub fn permits(&self, from: State, to: State) -> bool {
        self.transitions.contains_key(&(from, to))
    }

    /// The states the table allows leaving a state for.
    pub fn successors(&self, from: State) -> Vec<State> {
        self.transitions
            .iter()
            .filter(|((source, _), _)| *source == from)
            .map(|((_, target), _)| *target)
            .collect()
    }
}
