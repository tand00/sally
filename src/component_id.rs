#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ComponentID {
    #[default]
    Unmapped,
    Mapped(usize),
}

impl ComponentID {

    pub fn next(&self) -> ComponentID {
        match self {
            Self::Unmapped => Self::Mapped(0),
            Self::Mapped(i) => Self::Mapped(i+1),
        }
    }

    pub fn increment(&mut self) -> ComponentID {
        match self {
            Self::Unmapped => *self = Self::Mapped(0),
            Self::Mapped(i) => *i = *i+1,
        }
        *self
    }

    pub fn is_mapped(&self) -> bool {
        matches!(self, Self::Mapped(_))
    }

}

impl From<usize> for ComponentID {
    fn from(value: usize) -> Self {
        Self::Mapped(value)
    }
}

impl<T> From<&Vec<T>> for ComponentID {
    fn from(value: &Vec<T>) -> Self {
        Self::Mapped(value.len())
    }
}