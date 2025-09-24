mod verifier;
mod verification_iterator;

pub mod query;
pub mod smc;
pub mod text_query_parser;

pub use verifier::*;
pub use query::Query;

pub enum VerificationType {
    Analytical,
    Discrete,
    Statistical
}

pub struct VerificationConfig {

}
