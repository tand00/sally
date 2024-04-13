use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, Not};
use super::query::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum VerificationStatus {
    Maybe,
    Unverified,
    Verified
}

use VerificationStatus::*;

use crate::models::Label;

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

#[derive(Clone, PartialEq, Eq)]
pub enum VerificationBound {
    Time(u32),
    Steps(u32),
    Var(Label, u32),
    NoBound,
}

pub trait Verifiable {

    fn evaluate_object(&self, id : usize) -> i32;

    fn is_deadlocked(&self) -> bool;

}

pub struct Verification {
    pub query : Query,
    pub status : VerificationStatus,
    pub bound : VerificationBound
}

impl Verification {

    pub fn verify(&mut self, state : &impl Verifiable) -> VerificationStatus {
        Verified
    }

}

