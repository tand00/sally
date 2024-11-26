use std::{ops::{Add, Div, Mul, Neg, Sub}, sync::{Mutex, RwLock}};

use crate::computation::intervals::Convex;

use super::{time::Interval, Model};

pub trait DiffFunc : Fn(f64) -> f64 {

    fn grad_fn(&self, x : f64) -> f64;

    fn interval_image(&self, i : Interval<f64>) -> Interval<f64>;

}

pub enum ComputationTree<'a> {
    AddBranches(Box<ComputationTree<'a>>, Box<ComputationTree<'a>>),
    SubBranches(Box<ComputationTree<'a>>, Box<ComputationTree<'a>>),
    MulBranches(Box<ComputationTree<'a>>, Box<ComputationTree<'a>>),
    DivBranches(Box<ComputationTree<'a>>, Box<ComputationTree<'a>>),
    ScalBranch(f64, Box<ComputationTree<'a>>),
    ApplyFunc(Box<dyn DiffFunc>, Box<ComputationTree<'a>>),
    Leaf(&'a ModelParam),
    Constant(f64)
}

use ComputationTree::*;

pub struct ModelParam {
    pub value : Option<f64>,
    pub constraint : Interval<f64>,
    pub grad : Mutex<f64>
}

impl<'a> ComputationTree<'a> {

    pub fn constraint(&self) -> Interval<f64> {
        match self {
            AddBranches(c1, c2) => 
                c1.constraint() + c2.constraint(),
            SubBranches(c1, c2) => 
                c1.constraint() - c2.constraint(),
            MulBranches(c1, c2) => 
                c1.constraint() + todo!(),
            DivBranches(c1, c2) => todo!(),
            ScalBranch(s, c1) => c1.constraint() * (*s),
            Leaf(p) => p.constraint,
            Constant(_) => Interval::full(),
            ApplyFunc(diff_func, c) => 
                diff_func.interval_image(c.constraint()),
            
        }
    }

    pub fn value(&self) -> Option<f64> {
        match self {
            AddBranches(c1, c2) => 
                match (c1.value(), c2.value()) {
                    (Some(a), Some(b)) => Some(a + b),
                    _ => None
                },
            SubBranches(c1, c2) => 
                match (c1.value(), c2.value()) {
                    (Some(a), Some(b)) => Some(a - b),
                    _ => None
                },
            MulBranches(c1, c2) => 
                match (c1.value(), c2.value()) {
                    (Some(a), Some(b)) => Some(a * b),
                    _ => None
                },
            DivBranches(c1, c2) => 
                match (c1.value(), c2.value()) {
                    (Some(a), Some(b)) => Some(a / b),
                    _ => None
                },
            ScalBranch(s, c1) => 
                c1.value().map(|x| (*s) * x),
            Leaf(p) => p.value,
            Constant(c) => Some(*c),
            ApplyFunc(diff_func, c) => 
                c.value().map(diff_func),
        }
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