use std::{ops::{Add, Div, Mul, Neg, Sub}, sync::RwLock};

use super::{time::Interval, Model};

pub enum ComputationTree<'a> {
    AddBranches(Box<ComputationTree<'a>>, Box<ComputationTree<'a>>),
    SubBranches(Box<ComputationTree<'a>>, Box<ComputationTree<'a>>),
    MulBranches(Box<ComputationTree<'a>>, Box<ComputationTree<'a>>),
    DivBranches(Box<ComputationTree<'a>>, Box<ComputationTree<'a>>),
    ScalBranch(f64, Box<ComputationTree<'a>>),
    Leaf(&'a ModelParam)
}

use ComputationTree::*;

pub struct ModelParam {
    pub constraint : Interval<f64>,
}

impl<'a> ComputationTree<'a> {

    pub fn constraint(&self) -> Interval<f64> {
        todo!()
    }

}

// let a = ModelParam::new(0)
// let b = ModelParam::new(1.0)
// let c = &a + &b
// let c = c * 4
// c.grad(&a)

impl<'a> Add for ComputationTree<'a> {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        AddBranches(Box::new(self), Box::new(rhs))
    }
}

impl<'a> Sub for ComputationTree<'a> {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        SubBranches(Box::new(self), Box::new(rhs))
    }
}

impl<'a> Neg for ComputationTree<'a> {
    type Output = Self;
    fn neg(self) -> Self::Output {
        ScalBranch(-1.0, Box::new(self))
    }
}

impl<'a> Mul for ComputationTree<'a> {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        MulBranches(Box::new(self), Box::new(rhs))
    }
}

impl<'a> Div for ComputationTree<'a> {
    type Output = Self;
    fn div(self, rhs: Self) -> Self::Output {
        DivBranches(Box::new(self), Box::new(rhs))
    }
}

impl<'a> Mul<f64> for ComputationTree<'a> {
    type Output = Self;
    fn mul(self, rhs: f64) -> Self::Output {
        ScalBranch(rhs, Box::new(self))
    }
}
impl<'a> Mul<ComputationTree<'a>> for f64 {
    type Output = ComputationTree<'a>;
    fn mul(self, rhs: ComputationTree<'a>) -> Self::Output {
        rhs.mul(self)
    }
}

impl<'a> Add for &'a ModelParam {
    type Output = ComputationTree<'a>;
    fn add(self, rhs: Self) -> Self::Output {
        Leaf(self) + Leaf(rhs)
    }
}
impl<'a> Sub for &'a ModelParam {
    type Output = ComputationTree<'a>;
    fn sub(self, rhs: Self) -> Self::Output {
        Leaf(self) - Leaf(rhs)
    }
}
impl<'a> Mul for &'a ModelParam {
    type Output = ComputationTree<'a>;
    fn mul(self, rhs: Self) -> Self::Output {
        Leaf(self) * Leaf(rhs)
    }
}
impl<'a> Div for &'a ModelParam {
    type Output = ComputationTree<'a>;
    fn div(self, rhs: Self) -> Self::Output {
        Leaf(self) / Leaf(rhs)
    }
}
impl<'a> Neg for &'a ModelParam {
    type Output = ComputationTree<'a>;
    fn neg(self) -> Self::Output {
        -Leaf(self)
    }
}
impl<'a> Mul<f64> for &'a ModelParam {
    type Output = ComputationTree<'a>;
    fn mul(self, rhs : f64) -> Self::Output {
        Leaf(self) * rhs
    }
}
impl<'a> Mul<&'a ModelParam> for f64 {
    type Output = ComputationTree<'a>;
    fn mul(self, rhs: &'a ModelParam) -> Self::Output {
        rhs.mul(self)
    }
}