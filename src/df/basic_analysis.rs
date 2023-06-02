use clap::builder::Str;
use crate::df::df::{DataflowAnalysis, Direction, Sigma};

pub struct ReachingSigma {
    variables: Vec<String>,
}

pub struct ForwardReachingDirection;

impl Sigma for ReachingSigma {
    type Node = ();
    type LatticeValue = Vec<String>;

    fn merge(&self, other: &Self) -> Self {
        todo!()
    }

    fn is_equal(&self, other: &Self) -> bool {
        todo!()
    }

    fn lookup(&self, var: &Self::Node) -> Option<&Self::LatticeValue> {
        todo!()
    }

    fn set(&mut self, var: Self::Node, value: Self::LatticeValue) {
        todo!()
    }
}

impl Direction for ForwardReachingDirection {
    type Node = ();
    type Sigma = ReachingSigma;

    fn successors(&self, node: &Self::Node) -> Vec<Self::Node> {
        todo!()
    }

    fn initial_value(&self) -> Self::Sigma {
        todo!()
    }

    fn transfer(&self, node: &Self::Node, input: &Self::Sigma) -> Self::Sigma {
        todo!()
    }
}

#[cfg(test)]
#[path = "unit_tests/basic_analysis_tests.rs"]
mod basic_analysis_tests;

