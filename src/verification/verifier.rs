use std::{hash::Hash, ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, Not}};
use crate::{computation::virtual_memory::EvaluationType, models::{model_clock::ModelClock, model_var::ModelVar}};

use super::query::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VerificationStatus {
    Maybe,
    Unverified,
    Verified
}

use VerificationStatus::*;

impl VerificationStatus {
    pub fn good(&self) -> bool {
        *self == Verified
    }
    pub fn unsure(&self) -> bool {
        *self == Maybe
    }
    pub fn bad(&self) -> bool {
        *self == Unverified
    }
}

impl BitOr for VerificationStatus {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        match(self, rhs) {
            (Verified, _) => Verified,
            (_, Verified) => Verified,
            (Unverified, Unverified) => Unverified,
            _ => Maybe,
        }
    }
}

impl BitOrAssign for VerificationStatus {
    fn bitor_assign(&mut self, rhs: Self) {
        *self = *self | rhs
    }
}

impl BitAnd for VerificationStatus {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        match(self, rhs) {
            (Verified, Verified) => Verified,
            (_, Unverified) => Unverified,
            (Unverified, _) => Unverified,
            _ => Maybe,
        }
    }
}

impl BitAndAssign for VerificationStatus {
    fn bitand_assign(&mut self, rhs: Self) {
        *self = *self & rhs
    }
}

impl Not for VerificationStatus {
    type Output = Self;
    fn not(self) -> Self::Output {
        match self {
            Verified => Unverified,
            Unverified => Verified,
            Maybe => Maybe,
        }
    }
}

impl Default for VerificationStatus {
    fn default() -> Self {
        Maybe
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum VerificationBound {
    #[serde(rename = "time_bound")]
    TimeRunBound(u32),
    #[serde(rename = "steps_bound")]
    StepsRunBound(usize),
    #[serde(rename = "var_bound")]
    VarRunBound(ModelVar, i32),
    #[serde(rename = "no_bound")]
    NoRunBound,
}

impl Default for VerificationBound {
    fn default() -> Self {
        Self::NoRunBound
    }
}

pub trait Verifiable : Hash {
    fn evaluate_var(&self, var : &ModelVar) -> EvaluationType;
    fn evaluate_clock(&self, _ : &ModelClock) -> f64 {
        f64::NAN
    }
    fn is_deadlocked(&self) -> bool;
    fn as_verifiable(&self) -> &impl Verifiable
        where Self : Sized 
    {
        self
    }
}

pub type EvaluationState = u64; // Hashs of (Query, Verifiable)

pub struct Verification {
    pub query : Query,
    pub status : VerificationStatus,
    pub bound : VerificationBound,
}

impl Verification {

    pub fn new(query : Query, bound : VerificationBound) -> Self {
        Verification {
            query, bound,
            status : Maybe,
        }
    }

    pub fn verify(&mut self, _ : &mut Query, _ : &impl Verifiable) {
        
    }

}
