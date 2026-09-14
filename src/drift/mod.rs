//! Term drift: words a pairing removes that a canon requirement elsewhere
//! still uses. Pure over the pairing, canon and the change's own deltas.

pub mod filter;
pub mod siblings;
pub mod terms;

pub use siblings::drift_findings;
pub use terms::{removed_terms, RemovedTerm, Tier};
