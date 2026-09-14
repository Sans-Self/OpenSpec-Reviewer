//! The review model, serialized as-is.

use crate::review::Review;

pub fn render(review: &Review) -> String {
    serde_json::to_string_pretty(review).expect("review serializes")
}
