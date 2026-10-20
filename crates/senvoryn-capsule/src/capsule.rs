//! The Change Capsule: an immutable plan, its evidence, and the scoped
//! authority it requests.
//!
//! A capsule is **not** a script. It names operations from the trusted
//! Operation Registry, states the preconditions that must hold, the
//! postconditions that must be true afterwards, the budget it may spend, and
//! how its effects are undone. Nothing in this module executes anything.
//!
//! # The time model
//!
//! The wire format carries RFC 3339 date-times, and the executor needs a
//! number it can compare. The conversion happens once, here, at the boundary
//! where the document is read, and it accepts **only** the UTC form
//! `YYYY-MM-DDTHH:MM:SS[.fff]Z`. A timestamp carrying a numeric offset is
//! rejected rather than converted, because converting one needs a time-zone
//! database and this workspace has no dependency to provide one — and because
//! a capsule whose clock is stated in UTC is not ambiguous. The original
//! string is kept alongside the number so the evidence shows exactly what was
//! on the wire.

use std::collections::BTreeSet;

use senvoryn_canonical::Value;
use senvoryn_scp::{Reversibility, RiskTier};

/// Why a capsule document could not be read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CapsuleError {
    /// A required field was missing, of the wrong type, or out of range.
    Malformed {
        /// The field at fault.
        field: String,
        /// What was wrong with it.
        detail: String,
    },
    /// A timestamp was not in the strict UTC form the profile accepts.
    BadTimestamp {
        /// The field at fault.
        field: String,
        /// What was wrong with it.
        detail: String,
    },
    /// The document was not valid canonical JSON.
    NotCanonical(String),
}

impl core::fmt::Display for CapsuleError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Malformed { field, detail } => {
                write!(f, "capsule field {field:?} is malformed: {detail}")
            }
            Self::BadTimestamp { field, detail } => {
                write!(f, "capsule field {field:?} is not a UTC timestamp: {detail}")
            }
            Self::NotCanonical(detail) => write!(f, "the capsule is not valid JSON: {detail}"),
        }
    }
}

impl std::error::Error for CapsuleError {}

/// The system a capsule is aimed at.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Target {
    /// The identifier of the target.
    pub target_id: String,
    /// The kind of target.
    pub target_type: String,
    /// The version the capsule expects to find. A different version makes the
    /// capsule stale, and a stale capsule is rejected rather than adapted.
    pub expected_version: String,
}

/// A condition that must hold before the capsule may run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Precondition {
    /// The predicate's name.
    pub predicate_id: String,
    /// The version observed when the predicate was last checked, if any.
    pub observed_version: Option<String>,
}

/// A condition that must hold after the capsule has run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Postcondition {
    /// The predicate's name.
    pub predicate_id: String,
    /// Whether the capsule may be committed without it.
    ///
    /// A postcondition that is not mandatory still has to be reported, but its
    /// failure does not by itself stop a commit.
    pub mandatory: bool,
}

/// One step of a capsule.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapsuleStep {
    /// The step's identifier.
    pub step_id: String,
    /// The registry operation the step performs.
    pub operation: String,
    /// The key that makes a repeated attempt safe.
    pub idempotency_key: String,
    /// The digest of the step's parameters, as the originator computed it.
    pub parameters_digest: String,
    /// The risk the originator claims for this step.
    pub risk_tier: RiskTier,
    /// The longest the step may run, in milliseconds.
    pub timeout_ms: u64,
    /// The predicates that must hold for the step to have succeeded.
    pub success_predicates: BTreeSet<String>,
    /// The recovery action to take if this step's effect has to be undone.
    pub recovery_action_id: Option<String>,
}

/// What a capsule may spend.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapsuleBudget {
    /// The longest the whole capsule may run, in milliseconds.
    pub duration_ms: u64,
    /// The most operations the whole capsule may perform.
    pub operation_count: u64,
}

/// How a capsule's effects are undone.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecoveryPlan {
    /// The plan's identifier.
    pub plan_id: String,
    /// The strategy the plan uses.
    pub strategy: String,
    /// The artifact the recovery needs, if it needs one.
    pub recovery_artifact_ref: Option<String>,
    /// The effects that cannot be undone.
    pub irreversible_effects: BTreeSet<String>,
    /// The instructions a human needs, if a human is needed.
    pub human_instructions_ref: Option<String>,
}

impl RecoveryPlan {
    /// Whether the plan can be carried out automatically.
    ///
    /// A plan whose strategy is human recovery, or which names no strategy at
    /// all, cannot be carried out by the executor. Neither can one whose
    /// recovery artifact is missing.
    pub fn is_automatically_recoverable(&self) -> bool {
        matches!(self.strategy.as_str(), "ROLLBACK" | "COMPENSATE" | "RESTORE_SNAPSHOT")
            && self.recovery_artifact_ref.is_some()
    }
}

/// An approval a capsule requires before it may run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApprovalRequirement {
    /// The kind of approval.
    pub approval_type: String,
    /// How many of this kind are needed.
    pub count: u32,
}

/// A Change Capsule: an immutable plan plus evidence and scoped authority.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChangeCapsule {
    /// The capsule's identifier.
    pub capsule_id: String,
    /// Who asked for the capsule.
    pub originator_id: String,
    /// What the capsule acts on.
    pub target: Target,
    /// The intent the capsule serves.
    pub intent_id: String,
    /// The policy epoch the capsule was made under.
    pub policy_epoch: u64,
    /// The risk the originator claims for the capsule as a whole.
    pub risk_tier: RiskTier,
    /// How the capsule's effects are undone.
    pub reversibility: Reversibility,
    /// The conditions that must hold before the capsule may run.
    pub preconditions: Vec<Precondition>,
    /// The steps, in order.
    pub steps: Vec<CapsuleStep>,
    /// What the capsule may spend.
    pub budget: CapsuleBudget,
    /// The conditions that must hold afterwards.
    pub postconditions: Vec<Postcondition>,
    /// How the capsule's effects are undone.
    pub recovery_plan: RecoveryPlan,
    /// The wire form of the issue time.
    pub issued_at: String,
    /// The wire form of the expiry time.
    pub expires_at: String,
    /// The issue time, in milliseconds since the Unix epoch.
    pub issued_at_ms: i64,
    /// The expiry time, in milliseconds since the Unix epoch.
    pub expires_at_ms: i64,
    /// The approvals the capsule requires.
    pub approval_requirements: Vec<ApprovalRequirement>,
    /// The digest of the capsule's payload, as the originator computed it.
    pub payload_digest: String,
    /// The signatures over the capsule.
    pub signature_refs: Vec<String>,
}

impl ChangeCapsule {
    /// Reads a capsule from a canonical document.
    pub fn from_value(document: &Value) -> Result<Self, CapsuleError> {
        let schema = string_field(document, "schema")?;
        if schema != "senvoryn.change-capsule.v1" {
            return Err(CapsuleError::Malformed {
                field: "schema".to_owned(),
                detail: format!("{schema:?} is not a change capsule"),
            });
        }
        let target = document.get("target").ok_or_else(|| CapsuleError::Malformed {
            field: "target".to_owned(),
            detail: "absent".to_owned(),
        })?;
        let issued_at = string_field(document, "issued_at")?;
        let expires_at = string_field(document, "expires_at")?;
        let issued_at_ms = parse_utc_millis(&issued_at).map_err(|detail| {
            CapsuleError::BadTimestamp { field: "issued_at".to_owned(), detail }
        })?;
        let expires_at_ms = parse_utc_millis(&expires_at).map_err(|detail| {
            CapsuleError::BadTimestamp { field: "expires_at".to_owned(), detail }
        })?;
        Ok(Self {
            capsule_id: string_field(document, "capsule_id")?,
            originator_id: string_field(document, "originator_id")?,
            target: Target {
                target_id: string_field(target, "target_id")?,
                target_type: string_field(target, "target_type")?,
                expected_version: string_field(target, "expected_version")?,
            },
            intent_id: string_field(document, "intent_id")?,
            policy_epoch: integer_field(document, "policy_epoch")?,
            risk_tier: RiskTier::parse(&string_field(document, "risk_tier")?).ok_or_else(|| {
                CapsuleError::Malformed {
                    field: "risk_tier".to_owned(),
                    detail: "is not a risk tier".to_owned(),
                }
            })?,
            reversibility: Reversibility::parse(&string_field(document, "reversibility")?)
                .ok_or_else(|| CapsuleError::Malformed {
                    field: "reversibility".to_owned(),
                    detail: "is not a reversibility".to_owned(),
                })?,
            preconditions: read_preconditions(document)?,
            steps: read_steps(document)?,
            budget: read_budget(document)?,
            postconditions: read_postconditions(document)?,
            recovery_plan: read_recovery_plan(document)?,
            issued_at,
            expires_at,
            issued_at_ms,
            expires_at_ms,
            approval_requirements: read_approval_requirements(document)?,
            payload_digest: string_field(document, "payload_digest")?,
            signature_refs: string_set_field(document, "signature_refs").into_iter().collect(),
        })
    }

    /// The highest risk tier any of the capsule's steps claims.
    ///
    /// The registry is the authority on risk, so this is what the capsule
    /// *claims*; the executor compares it with what the registry says.
    pub fn claimed_risk_tier(&self) -> RiskTier {
        self.steps.iter().map(|step| step.risk_tier).max().unwrap_or(self.risk_tier)
    }

    /// The mandatory postconditions, by name.
    pub fn mandatory_postconditions(&self) -> BTreeSet<String> {
        self.postconditions
            .iter()
            .filter(|postcondition| postcondition.mandatory)
            .map(|postcondition| postcondition.predicate_id.clone())
            .collect()
    }
}

fn read_preconditions(document: &Value) -> Result<Vec<Precondition>, CapsuleError> {
    let raw = document.get("preconditions").and_then(Value::as_array).ok_or_else(|| {
        CapsuleError::Malformed {
            field: "preconditions".to_owned(),
            detail: "absent or not an array".to_owned(),
        }
    })?;
    let mut preconditions = Vec::with_capacity(raw.len());
    for entry in raw {
        // The schema requires `expected`: a precondition that does not say
        // what it expects is not a precondition. The executor's environment
        // models preconditions by predicate name, so the value itself is
        // carried for the evidence rather than evaluated here, but its
        // absence is a malformed capsule.
        if entry.get("expected").is_none() {
            return Err(CapsuleError::Malformed {
                field: "preconditions".to_owned(),
                detail: "a precondition does not say what it expects".to_owned(),
            });
        }
        preconditions.push(Precondition {
            predicate_id: string_field(entry, "predicate_id")?,
            observed_version: entry
                .get("observed_version")
                .and_then(Value::as_str)
                .map(str::to_owned),
        });
    }
    Ok(preconditions)
}

fn read_postconditions(document: &Value) -> Result<Vec<Postcondition>, CapsuleError> {
    let raw = document.get("postconditions").and_then(Value::as_array).ok_or_else(|| {
        CapsuleError::Malformed {
            field: "postconditions".to_owned(),
            detail: "absent or not an array".to_owned(),
        }
    })?;
    let mut postconditions = Vec::with_capacity(raw.len());
    for entry in raw {
        let mandatory = entry.get("mandatory").and_then(Value::as_bool).ok_or_else(|| {
            CapsuleError::Malformed {
                field: "postconditions[].mandatory".to_owned(),
                detail: "absent or not a boolean".to_owned(),
            }
        })?;
        postconditions
            .push(Postcondition { predicate_id: string_field(entry, "predicate_id")?, mandatory });
    }
    Ok(postconditions)
}

fn read_steps(document: &Value) -> Result<Vec<CapsuleStep>, CapsuleError> {
    let raw =
        document.get("steps").and_then(Value::as_array).ok_or_else(|| CapsuleError::Malformed {
            field: "steps".to_owned(),
            detail: "absent or not an array".to_owned(),
        })?;
    let mut steps = Vec::with_capacity(raw.len());
    for entry in raw {
        let risk_tier = RiskTier::parse(&string_field(entry, "risk_tier")?).ok_or_else(|| {
            CapsuleError::Malformed {
                field: "steps[].risk_tier".to_owned(),
                detail: "is not a risk tier".to_owned(),
            }
        })?;
        steps.push(CapsuleStep {
            step_id: string_field(entry, "step_id")?,
            operation: string_field(entry, "operation")?,
            idempotency_key: string_field(entry, "idempotency_key")?,
            parameters_digest: string_field(entry, "parameters_digest")?,
            risk_tier,
            timeout_ms: integer_field(entry, "timeout_ms")?,
            success_predicates: string_set_field(entry, "success_predicates"),
            recovery_action_id: entry
                .get("recovery_action_id")
                .and_then(Value::as_str)
                .map(str::to_owned),
        });
    }
    Ok(steps)
}

fn read_budget(document: &Value) -> Result<CapsuleBudget, CapsuleError> {
    let budget = document.get("budget").ok_or_else(|| CapsuleError::Malformed {
        field: "budget".to_owned(),
        detail: "absent".to_owned(),
    })?;
    Ok(CapsuleBudget {
        duration_ms: integer_field(budget, "duration_ms")?,
        operation_count: integer_field(budget, "operation_count")?,
    })
}

fn read_recovery_plan(document: &Value) -> Result<RecoveryPlan, CapsuleError> {
    let plan = document.get("recovery_plan").ok_or_else(|| CapsuleError::Malformed {
        field: "recovery_plan".to_owned(),
        detail: "absent".to_owned(),
    })?;
    Ok(RecoveryPlan {
        plan_id: string_field(plan, "plan_id")?,
        strategy: string_field(plan, "strategy")?,
        recovery_artifact_ref: plan
            .get("recovery_artifact_ref")
            .and_then(Value::as_str)
            .map(str::to_owned),
        irreversible_effects: string_set_field(plan, "irreversible_effects"),
        human_instructions_ref: plan
            .get("human_instructions_ref")
            .and_then(Value::as_str)
            .map(str::to_owned),
    })
}

fn read_approval_requirements(document: &Value) -> Result<Vec<ApprovalRequirement>, CapsuleError> {
    let raw = document.get("approval_requirements").and_then(Value::as_array).ok_or_else(|| {
        CapsuleError::Malformed {
            field: "approval_requirements".to_owned(),
            detail: "absent or not an array".to_owned(),
        }
    })?;
    let mut requirements = Vec::with_capacity(raw.len());
    for entry in raw {
        let count = integer_field(entry, "count")?;
        requirements.push(ApprovalRequirement {
            approval_type: string_field(entry, "approval_type")?,
            count: u32::try_from(count).map_err(|_| CapsuleError::Malformed {
                field: "approval_requirements[].count".to_owned(),
                detail: "is larger than a u32".to_owned(),
            })?,
        });
    }
    Ok(requirements)
}

fn string_field(value: &Value, field: &str) -> Result<String, CapsuleError> {
    value.get(field).and_then(Value::as_str).map(str::to_owned).ok_or_else(|| {
        CapsuleError::Malformed {
            field: field.to_owned(),
            detail: "absent or not a string".to_owned(),
        }
    })
}

fn string_set_field(value: &Value, field: &str) -> BTreeSet<String> {
    value
        .get(field)
        .and_then(Value::as_array)
        .map(|items| items.iter().filter_map(Value::as_str).map(str::to_owned).collect())
        .unwrap_or_default()
}

fn integer_field(value: &Value, field: &str) -> Result<u64, CapsuleError> {
    let number =
        value.get(field).and_then(Value::as_int).ok_or_else(|| CapsuleError::Malformed {
            field: field.to_owned(),
            detail: "absent or not an integer".to_owned(),
        })?;
    u64::try_from(number).map_err(|_| CapsuleError::Malformed {
        field: field.to_owned(),
        detail: format!("{number} is negative"),
    })
}

/// Parses a strict UTC RFC 3339 timestamp into milliseconds since the epoch.
///
/// Only `YYYY-MM-DDTHH:MM:SSZ` and `YYYY-MM-DDTHH:MM:SS.sssZ` are accepted.
/// A numeric offset is refused rather than converted: converting one needs a
/// time-zone database, and a capsule that states its clock in UTC is not
/// ambiguous. The parser is total — there is no input for which it panics and
/// no input for which it returns a wrong number without saying so.
pub fn parse_utc_millis(text: &str) -> Result<i64, String> {
    let bytes = text.as_bytes();
    if bytes.len() < 20 || bytes.len() > 24 {
        return Err(format!("{text:?} is not a UTC timestamp"));
    }
    if bytes[4] != b'-'
        || bytes[7] != b'-'
        || bytes[10] != b'T'
        || bytes[13] != b':'
        || bytes[16] != b':'
    {
        return Err(format!("{text:?} is not a UTC timestamp"));
    }
    // The `Z` is the last byte. A fractional part, if there is one, sits
    // between the seconds and the `Z`, so the `Z` is not at a fixed offset.
    // Reading it at a fixed offset rejects every timestamp that carries
    // fractional seconds, and the fraction-handling code below would never
    // be reached.
    if bytes[bytes.len() - 1] != b'Z' {
        return Err(format!("{text:?} is not in UTC"));
    }
    let year = number(&bytes[0..4]).ok_or_else(|| format!("{text:?} has no year"))?;
    if year < 1 {
        return Err(format!("{text:?} is not a date"));
    }
    let month = number(&bytes[5..7]).ok_or_else(|| format!("{text:?} has no month"))?;
    let day = number(&bytes[8..10]).ok_or_else(|| format!("{text:?} has no day"))?;
    let hour = number(&bytes[11..13]).ok_or_else(|| format!("{text:?} has no hour"))?;
    let minute = number(&bytes[14..16]).ok_or_else(|| format!("{text:?} has no minute"))?;
    let second = number(&bytes[17..19]).ok_or_else(|| format!("{text:?} has no second"))?;
    if !(1..=12).contains(&month) || day < 1 || day > days_in_month(year, month) {
        return Err(format!("{text:?} is not a date"));
    }
    if hour > 23 || minute > 59 || second > 59 {
        return Err(format!("{text:?} is not a time"));
    }
    let mut millis = 0i64;
    if bytes.len() > 20 {
        if bytes[19] != b'.' {
            return Err(format!("{text:?} is not a UTC timestamp"));
        }
        let fraction = &text[20..text.len() - 1];
        if fraction.is_empty() || !fraction.bytes().all(|byte| byte.is_ascii_digit()) {
            return Err(format!("{text:?} has no fractional seconds"));
        }
        let mut scaled = String::with_capacity(3);
        scaled.push_str(&fraction[..fraction.len().min(3)]);
        while scaled.len() < 3 {
            scaled.push('0');
        }
        millis = number(scaled.as_bytes()).ok_or_else(|| format!("{text:?} has no fraction"))?;
    }
    let days = days_from_civil(year, month, day);
    let seconds = days * 86_400 + hour * 3_600 + minute * 60 + second;
    Ok(seconds * 1_000 + millis)
}

/// How many days a month has, in the proleptic Gregorian calendar.
///
/// A date that does not exist — the thirtieth of February, or the
/// twenty-ninth of February in a year that is not a leap year — is refused
/// rather than rolled over into the next month. A capsule that names a time
/// which never happened is malformed, and silently moving it would mean the
/// executor compared the capsule against an instant its issuer never stated.
fn days_in_month(year: i64, month: i64) -> i64 {
    const LENGTHS: [i64; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    if month == 2 && is_leap_year(year) {
        return 29;
    }
    LENGTHS[(month - 1) as usize]
}

/// Whether a year has a twenty-ninth of February.
fn is_leap_year(year: i64) -> bool {
    year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
}

/// The number of days since 1970-01-01 for a civil date.
///
/// Howard Hinnant's `days_from_civil`, which is exact for the whole proleptic
/// Gregorian calendar and needs no lookup table.
fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
    let year = if month <= 2 { year - 1 } else { year };
    let era = if year >= 0 { year } else { year - 399 } / 400;
    let year_of_era = year - era * 400;
    let month_position = if month > 2 { month - 3 } else { month + 9 };
    let day_of_year = (153 * month_position + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * 146_097 + day_of_era - 719_468
}

fn number(bytes: &[u8]) -> Option<i64> {
    if bytes.is_empty() || !bytes.iter().all(u8::is_ascii_digit) {
        return None;
    }
    let mut value = 0i64;
    for byte in bytes {
        value = value * 10 + i64::from(byte - b'0');
    }
    Some(value)
}
