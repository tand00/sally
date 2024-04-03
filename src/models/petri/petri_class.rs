use super::PetriMarking;
use crate::models::time::TimeInterval;

pub struct PetriClass {
    marking: PetriMarking,
    intervals: Vec<TimeInterval>
}