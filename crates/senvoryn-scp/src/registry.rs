//! The trusted Operation Registry, and the risk tier it assigns.
//!
//! The registry is the authority on risk. The caller's claim is never
//! authoritative: a claim below the registry's tier is a mismatch, and a
//! claim above it is ignored in favour of the registry's tier. There is no
//! default low-risk classification, and an operation the registry does not
//! contain has no tier at all.

use std::collections::BTreeSet;

use senvoryn_canonical::{Value, parse};

/// The sample Operation Registry, embedded at compile time.
///
/// This is the prototype registry: it is unsigned, and its `signature_ref`
/// says so. A signed registry is ADR-002's work and is not done.
const SAMPLE_REGISTRY_JSON: &str = include_str!("../../../examples/operation-registry.sample.json");

/// A risk tier, in ascending order of consequence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RiskTier {
    /// No consequence beyond reading.
    R0,
    /// Bounded and reversible.
    R1,
    /// Compensatable.
    R2,
    /// Not reversible.
    R3,
}

impl RiskTier {
    /// The tier as it appears in a request or a registry entry.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::R0 => "R0",
            Self::R1 => "R1",
            Self::R2 => "R2",
            Self::R3 => "R3",
        }
    }

    /// Reads a tier from its serialized form.
    pub fn parse(text: &str) -> Option<Self> {
        match text {
            "R0" => Some(Self::R0),
            "R1" => Some(Self::R1),
            "R2" => Some(Self::R2),
            "R3" => Some(Self::R3),
            _ => None,
        }
    }
}

/// The audit class an operation's record must reach.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AuditClass {
    /// Ordinary audit recording.
    Standard,
    /// Sensitive audit recording.
    Sensitive,
    /// Security-critical audit recording.
    SecurityCritical,
}

impl AuditClass {
    /// The class as it appears in a registry entry.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Standard => "STANDARD",
            Self::Sensitive => "SENSITIVE",
            Self::SecurityCritical => "SECURITY_CRITICAL",
        }
    }

    /// Reads a class from its serialized form.
    pub fn parse(text: &str) -> Option<Self> {
        match text {
            "STANDARD" => Some(Self::Standard),
            "SENSITIVE" => Some(Self::Sensitive),
            "SECURITY_CRITICAL" => Some(Self::SecurityCritical),
            _ => None,
        }
    }
}

/// How an operation's effects can be undone.
///
/// The ordering is by how reversible the operation is, least reversible last,
/// so that merging two rules for the same operation keeps the stricter one.
/// `Unknown` sorts last because an operation whose reversibility the registry
/// does not state is treated as the worst case, not as the best one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Reversibility {
    /// The effect can be undone exactly.
    Reversible,
    /// The effect can be compensated for, but not undone.
    Compensatable,
    /// The effect cannot be undone or compensated for.
    NonReversible,
    /// The registry does not say. Treated as the worst case.
    Unknown,
}

impl Reversibility {
    /// The value as it appears in a registry entry.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Reversible => "REVERSIBLE",
            Self::Compensatable => "COMPENSATABLE",
            Self::NonReversible => "NON_REVERSIBLE",
            Self::Unknown => "UNKNOWN",
        }
    }

    /// Reads a value from its serialized form.
    pub fn parse(text: &str) -> Option<Self> {
        match text {
            "REVERSIBLE" => Some(Self::Reversible),
            "COMPENSATABLE" => Some(Self::Compensatable),
            "NON_REVERSIBLE" => Some(Self::NonReversible),
            "UNKNOWN" => Some(Self::Unknown),
            _ => None,
        }
    }

    /// Whether the operation can be brought back by compensation.
    ///
    /// `UNKNOWN` is not compensation. An operation whose reversibility the
    /// registry does not state cannot be rolled back on the executor's
    /// initiative, so it needs human recovery instead.
    pub fn is_compensatable(&self) -> bool {
        matches!(self, Self::Reversible | Self::Compensatable)
    }
}

/// One entry of the Operation Registry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OperationEntry {
    /// The operation this entry describes.
    pub operation_id: String,
    /// The risk tier the operation carries.
    pub risk_tier: RiskTier,
    /// The approvals the operation requires. `NONE` means none.
    pub approval_requirements: BTreeSet<String>,
    /// The system modes the operation may be requested in.
    pub allowed_modes: BTreeSet<String>,
    /// The resource types the operation is defined for.
    pub resource_types: BTreeSet<String>,
    /// The longest the operation may run, in milliseconds.
    pub max_duration_ms: Option<u64>,
    /// The most times the operation may be performed.
    pub max_operation_count: Option<u64>,
    /// The preconditions the operation requires.
    pub required_preconditions: BTreeSet<String>,
    /// The postconditions the operation must leave satisfied.
    ///
    /// An operation with no postconditions cannot be verified, and
    /// specification §5.2 says an outcome that cannot be proven is `PARTIAL`
    /// or `NEEDS_HUMAN_RECOVERY`, never `COMMITTED`.
    pub required_postconditions: BTreeSet<String>,
    /// How the operation's effects can be undone.
    pub reversibility: Reversibility,
    /// The out-of-process effects the operation has.
    ///
    /// An operation with an external effect is not undone by restoring local
    /// state; specification §5.4 says "undo" must not imply the external world
    /// has been rewound.
    pub external_effects: BTreeSet<String>,
    /// Whether repeating the operation has the same effect as running it once.
    pub idempotent: bool,
    /// The audit class the operation's record must reach.
    pub audit_class: AuditClass,
}

/// The trusted Operation Registry.
///
/// The registry is a list rather than a map because it may hold more than one
/// rule for the same operation, and the strictest rule is the one that
/// applies.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct OperationRegistry {
    entries: Vec<OperationEntry>,
}

/// Why a registry document could not be read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegistryError {
    /// A required field was missing, of the wrong type, or out of range.
    Malformed {
        /// The field at fault.
        field: String,
        /// What was wrong with it.
        detail: String,
    },
    /// The document was not valid canonical JSON.
    NotCanonical(String),
}

impl core::fmt::Display for RegistryError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Malformed { field, detail } => {
                write!(f, "registry field {field:?} is malformed: {detail}")
            }
            Self::NotCanonical(detail) => write!(f, "the registry is not valid JSON: {detail}"),
        }
    }
}

impl std::error::Error for RegistryError {}

impl OperationRegistry {
    /// A registry holding the given entries.
    pub fn from_entries(entries: Vec<OperationEntry>) -> Self {
        Self { entries }
    }

    /// The entries in the registry, in the order they were read.
    pub fn entries(&self) -> &[OperationEntry] {
        &self.entries
    }

    /// The strictest rule that applies to an operation, if the registry has
    /// any rule for it.
    ///
    /// "When multiple rules apply, the most restrictive risk tier wins" —
    /// specification §3.2. The same principle is applied to the other
    /// dimensions: the narrowest mode set, the union of approval
    /// requirements, the smallest budgets, and the highest audit class. A
    /// caller therefore cannot escape a strict rule by also matching a lax
    /// one.
    pub fn effective(&self, operation_id: &str) -> Option<OperationEntry> {
        let matching: Vec<&OperationEntry> =
            self.entries.iter().filter(|entry| entry.operation_id == operation_id).collect();
        let first = matching.first()?;
        let mut merged = (*first).clone();
        for entry in &matching[1..] {
            merged.risk_tier = merged.risk_tier.max(entry.risk_tier);
            merged.audit_class = merged.audit_class.max(entry.audit_class);
            merged.allowed_modes =
                merged.allowed_modes.intersection(&entry.allowed_modes).cloned().collect();
            merged.resource_types =
                merged.resource_types.union(&entry.resource_types).cloned().collect();
            merged.approval_requirements =
                merged.approval_requirements.union(&entry.approval_requirements).cloned().collect();
            merged.required_preconditions = merged
                .required_preconditions
                .union(&entry.required_preconditions)
                .cloned()
                .collect();
            merged.required_postconditions = merged
                .required_postconditions
                .union(&entry.required_postconditions)
                .cloned()
                .collect();
            merged.external_effects =
                merged.external_effects.union(&entry.external_effects).cloned().collect();
            // The least reversible rule wins, because a caller must not be able
            // to escape a strict rule by also matching a lax one.
            merged.reversibility = merged.reversibility.min(entry.reversibility);
            // Only an operation that is idempotent under every matching rule
            // may be treated as idempotent.
            merged.idempotent = merged.idempotent && entry.idempotent;
            merged.max_duration_ms = min_option(merged.max_duration_ms, entry.max_duration_ms);
            merged.max_operation_count =
                min_option(merged.max_operation_count, entry.max_operation_count);
        }
        Some(merged)
    }

    /// Reads a registry from a canonical document.
    pub fn from_value(document: &Value) -> Result<Self, RegistryError> {
        let schema = document.get("schema").and_then(Value::as_str).ok_or_else(|| {
            RegistryError::Malformed {
                field: "schema".to_owned(),
                detail: "absent or not a string".to_owned(),
            }
        })?;
        if schema != "senvoryn.operation-registry.v1" {
            return Err(RegistryError::Malformed {
                field: "schema".to_owned(),
                detail: format!("{schema:?} is not an operation registry"),
            });
        }
        let operations = document.get("operations").and_then(Value::as_array).ok_or_else(|| {
            RegistryError::Malformed {
                field: "operations".to_owned(),
                detail: "absent or not an array".to_owned(),
            }
        })?;
        let mut entries = Vec::with_capacity(operations.len());
        for operation in operations {
            entries.push(read_entry(operation)?);
        }
        Ok(Self { entries })
    }

    /// Reads a registry from JSON text.
    pub fn from_json(text: &str) -> Result<Self, RegistryError> {
        let document =
            parse(text).map_err(|error| RegistryError::NotCanonical(error.to_string()))?;
        Self::from_value(&document)
    }

    /// The embedded sample registry.
    ///
    /// Unsigned, and its `signature_ref` says so. It is the prototype's
    /// policy data, not a trust anchor.
    pub fn sample() -> Result<Self, RegistryError> {
        Self::from_json(SAMPLE_REGISTRY_JSON)
    }
}

fn min_option(left: Option<u64>, right: Option<u64>) -> Option<u64> {
    match (left, right) {
        (None, other) | (other, None) => other,
        (Some(a), Some(b)) => Some(a.min(b)),
    }
}

fn read_entry(entry: &Value) -> Result<OperationEntry, RegistryError> {
    let operation_id = string_field(entry, "operation_id")?;
    let risk_tier_text = string_field(entry, "risk_tier")?;
    let risk_tier = RiskTier::parse(&risk_tier_text).ok_or_else(|| RegistryError::Malformed {
        field: "risk_tier".to_owned(),
        detail: format!("{risk_tier_text:?} is not a risk tier"),
    })?;
    let audit_class_text = entry.get("audit_class").and_then(Value::as_str).unwrap_or("STANDARD");
    let audit_class =
        AuditClass::parse(audit_class_text).ok_or_else(|| RegistryError::Malformed {
            field: "audit_class".to_owned(),
            detail: format!("{audit_class_text:?} is not an audit class"),
        })?;
    let reversibility_text =
        entry.get("reversibility").and_then(Value::as_str).unwrap_or("UNKNOWN");
    let reversibility =
        Reversibility::parse(reversibility_text).ok_or_else(|| RegistryError::Malformed {
            field: "reversibility".to_owned(),
            detail: format!("{reversibility_text:?} is not a reversibility"),
        })?;
    Ok(OperationEntry {
        operation_id,
        risk_tier,
        approval_requirements: string_set_field(entry, "approval_requirements"),
        allowed_modes: string_set_field(entry, "allowed_modes"),
        resource_types: string_set_field(entry, "resource_types"),
        max_duration_ms: integer_field(entry, "max_duration_ms")?,
        max_operation_count: integer_field(entry, "max_operation_count")?,
        required_preconditions: string_set_field(entry, "required_preconditions"),
        required_postconditions: string_set_field(entry, "required_postconditions"),
        reversibility,
        external_effects: string_set_field(entry, "external_effects"),
        idempotent: entry.get("idempotent").and_then(Value::as_bool).unwrap_or(false),
        audit_class,
    })
}

fn string_field(entry: &Value, field: &str) -> Result<String, RegistryError> {
    entry.get(field).and_then(Value::as_str).map(str::to_owned).ok_or_else(|| {
        RegistryError::Malformed {
            field: field.to_owned(),
            detail: "absent or not a string".to_owned(),
        }
    })
}

fn string_set_field(entry: &Value, field: &str) -> BTreeSet<String> {
    entry
        .get(field)
        .and_then(Value::as_array)
        .map(|items| items.iter().filter_map(Value::as_str).map(str::to_owned).collect())
        .unwrap_or_default()
}

fn integer_field(entry: &Value, field: &str) -> Result<Option<u64>, RegistryError> {
    match entry.get(field) {
        None | Some(Value::Null) => Ok(None),
        Some(value) => {
            let number = value.as_int().ok_or_else(|| RegistryError::Malformed {
                field: field.to_owned(),
                detail: format!("expected an integer, found {}", value.type_name()),
            })?;
            u64::try_from(number).map(Some).map_err(|_| RegistryError::Malformed {
                field: field.to_owned(),
                detail: format!("{number} is negative"),
            })
        }
    }
}
