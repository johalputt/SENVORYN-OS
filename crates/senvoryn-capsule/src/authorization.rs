//! The authorization a capsule needs before it may act, and the check that
//! the authorization binds the capsule it was given.
//!
//! The System Control Plane is the authority on whether a capsule may run.
//! This module does not decide that and does not second-guess it: it reads
//! the decision the SCP made and checks the thing the decision has to bind —
//! **the exact digest of the plan that was authorized**, **the version of the
//! target the decision was made against** — plus the approvals the capsule
//! asked for.
//!
//! The decision is read from the record the SCP produces
//! (`senvoryn_scp::record::DecisionRecord`). It used to arrive as a value the
//! host assembled, and a value the host assembles is a value the host can
//! assemble wrongly: nothing tied the digest in it to anything the policy
//! layer had decided.
//!
//! Keeping the SCP outside this crate is deliberate. Two authorities on the
//! same question is the failure mode the shared conformance vectors exist to
//! prevent, and a second copy of the policy here would be exactly that.

use std::collections::BTreeMap;

use senvoryn_scp::DecisionStatus;
use senvoryn_scp::record::DecisionRecord;

/// An approval given for a capsule.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Approval {
    /// The kind of approval, matching the capsule's `approval_requirements`.
    pub approval_type: String,
    /// The digest the approval was given for.
    ///
    /// An approval given for a different plan is not an approval for this
    /// one. Specification §5.3: "required user/dual approvals bind that
    /// digest".
    pub binds_digest: String,
}

/// The System Control Plane's decision about a capsule.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Authorization {
    /// The status the decision carried.
    ///
    /// Only `PERMIT` grants authority. A decision that refused, asked for
    /// evidence, deferred or quarantined the request authorises nothing, and
    /// the executor reads the status rather than trusting the host to have
    /// checked it.
    pub decision_status: DecisionStatus,
    /// The digest of the plan the decision permits.
    ///
    /// `None` is a refusal, not a wildcard. A request that named no plan
    /// produced a decision about an operation rather than about a plan, and
    /// such a decision authorises no plan at all.
    pub permitted_digest: Option<String>,
    /// The version of the target the decision was made against.
    ///
    /// `None` is a refusal, not a wildcard. A decision made before the
    /// resource was resolved names no version, and a decision that cannot say
    /// which version of the object it judged authorises no effect on that
    /// object.
    pub permitted_resource_version: Option<String>,
    /// The approvals that were given.
    pub approvals: Vec<Approval>,
}

impl Authorization {
    /// Reads the authorization out of a decision record.
    ///
    /// The record is the policy layer's own output, read through the closed
    /// reader in `senvoryn_scp::record`, so a field the record does not
    /// define cannot be carried in unnoticed.
    ///
    /// The approvals are not in the record — the policy layer is given the
    /// approval's digest, not the approval itself — so they are passed in
    /// beside it. That is the last of the authorization the host still
    /// assembles; the plan and the version now come from the decision.
    pub fn from_record(record: &DecisionRecord, approvals: Vec<Approval>) -> Self {
        Self {
            decision_status: record.status,
            permitted_digest: record.permitted_payload_digest.clone(),
            permitted_resource_version: record.target_resource_version.clone(),
            approvals,
        }
    }
}

/// Why an authorization does not permit a capsule.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthorizationError {
    /// The decision granted no authority.
    DecisionGrantsNoAuthority {
        /// The status the decision carried.
        status: &'static str,
    },
    /// The decision does not name the plan it was made about.
    PlanNotNamed,
    /// The decision was made about a different plan.
    DigestMismatch {
        /// The digest the capsule carries.
        capsule_digest: String,
        /// The digest the decision permits.
        permitted_digest: String,
    },
    /// The decision does not name the version it was made against.
    TargetVersionNotNamed,
    /// The decision was made against a different version of the target.
    TargetVersionMismatch {
        /// The version the capsule expects the target to be at.
        capsule_expected_version: String,
        /// The version the decision was made against.
        permitted_resource_version: String,
    },
    /// An approval the capsule requires is missing.
    MissingApproval {
        /// The kind of approval that is missing.
        approval_type: String,
    },
    /// Not enough approvals of a required kind were given.
    InsufficientApprovals {
        /// The kind of approval.
        approval_type: String,
        /// How many were given.
        given: usize,
        /// How many are needed.
        required: u32,
    },
    /// An approval was given for a different plan.
    ApprovalBoundElsewhere {
        /// The kind of approval.
        approval_type: String,
    },
}

impl core::fmt::Display for AuthorizationError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::DecisionGrantsNoAuthority { status } => {
                write!(f, "the decision granted no authority: its status is {status}")
            }
            Self::PlanNotNamed => {
                write!(f, "the decision does not name the plan it was made about")
            }
            Self::DigestMismatch { capsule_digest, permitted_digest } => write!(
                f,
                "the decision permits digest {permitted_digest:?}, not the capsule's {capsule_digest:?}"
            ),
            Self::TargetVersionNotNamed => write!(
                f,
                "the decision does not name the version of the target it was made against"
            ),
            Self::TargetVersionMismatch {
                capsule_expected_version,
                permitted_resource_version,
            } => {
                write!(
                    f,
                    "the decision was made against version {permitted_resource_version:?} of the target, not the version {capsule_expected_version:?} the capsule expects"
                )
            }
            Self::MissingApproval { approval_type } => {
                write!(f, "the capsule requires a {approval_type} approval and none was given")
            }
            Self::InsufficientApprovals { approval_type, given, required } => write!(
                f,
                "the capsule requires {required} {approval_type} approval(s) and {given} were given"
            ),
            Self::ApprovalBoundElsewhere { approval_type } => {
                write!(f, "a {approval_type} approval was given for a different plan")
            }
        }
    }
}

impl std::error::Error for AuthorizationError {}

/// Checks that an authorization permits a capsule.
///
/// The status comes first: a decision that granted no authority authorises
/// nothing, and no amount of matching digests makes a refusal into a grant.
/// The digest check comes next because it is the one that cannot be worked
/// around: an approval for a different plan is not an approval for this one,
/// however many there are of them. The version check comes after it: a
/// decision names the version of the target it was made against, and the
/// target may have moved since (ADR-018, threat model AS-19). The executor
/// already checks that the target is still at the version the capsule
/// expects; this is the other half — that the decision was made about that
/// version at all.
pub fn check(
    capsule: &crate::capsule::ChangeCapsule,
    authorization: &Authorization,
) -> Result<(), AuthorizationError> {
    if !authorization.decision_status.grants_authority() {
        return Err(AuthorizationError::DecisionGrantsNoAuthority {
            status: authorization.decision_status.as_str(),
        });
    }
    let Some(permitted_digest) = &authorization.permitted_digest else {
        return Err(AuthorizationError::PlanNotNamed);
    };
    if permitted_digest != &capsule.payload_digest {
        return Err(AuthorizationError::DigestMismatch {
            capsule_digest: capsule.payload_digest.clone(),
            permitted_digest: permitted_digest.clone(),
        });
    }
    let Some(permitted_version) = &authorization.permitted_resource_version else {
        return Err(AuthorizationError::TargetVersionNotNamed);
    };
    if permitted_version != &capsule.target.expected_version {
        return Err(AuthorizationError::TargetVersionMismatch {
            capsule_expected_version: capsule.target.expected_version.clone(),
            permitted_resource_version: permitted_version.clone(),
        });
    }
    let mut given: BTreeMap<&str, usize> = BTreeMap::new();
    for approval in &authorization.approvals {
        if approval.binds_digest != capsule.payload_digest {
            return Err(AuthorizationError::ApprovalBoundElsewhere {
                approval_type: approval.approval_type.clone(),
            });
        }
        *given.entry(approval.approval_type.as_str()).or_default() += 1;
    }
    for requirement in &capsule.approval_requirements {
        let count = given.get(requirement.approval_type.as_str()).copied().unwrap_or(0);
        if count == 0 {
            return Err(AuthorizationError::MissingApproval {
                approval_type: requirement.approval_type.clone(),
            });
        }
        if u32::try_from(count).unwrap_or(u32::MAX) < requirement.count {
            return Err(AuthorizationError::InsufficientApprovals {
                approval_type: requirement.approval_type.clone(),
                given: count,
                required: requirement.count,
            });
        }
    }
    Ok(())
}
