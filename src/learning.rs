pub mod genetic;
pub mod logic_2D;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OptimizationObjective {
    Maximize, Minimize
}

impl OptimizationObjective {

    pub fn factor(&self) -> f64 {
        match self {
            Self::Maximize => 1.0,
            Self::Minimize => -1.0,
        }
    }

}

impl Default for OptimizationObjective {
    fn default() -> Self {
        Self::Maximize
    }
}