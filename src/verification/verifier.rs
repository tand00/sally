use std::{hash::Hash, ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, Not}};
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
        return *self == Verified
    }
    pub fn unsure(&self) -> bool {
        return *self == Maybe
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

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VerificationBound {
    TimeRunBound(u32),
    StepsRunBound(usize),
    VarRunBound(usize, i32),
    NoRunBound,
}

pub trait Verifiable : Hash {
    fn evaluate_object(&self, id : usize) -> i32;
    fn evaluate_clock(&self, _ : usize) -> f64 {
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

    pub fn verify(&mut self, query : &mut Query, state : &impl Verifiable) {
        
    }

}
