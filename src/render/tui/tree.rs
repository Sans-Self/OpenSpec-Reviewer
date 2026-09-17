//! Tree guides for the list. A row's prefix depends on whether it and
//! each of its ancestors have a later sibling, and both change when a
//! requirement folds, so the guides are derived from the visible rows
//! on every draw rather than stored on the row.

use super::app::Row;

const BRANCH: &str = "├─ ";
const LAST: &str = "└─ ";
const CONTINUE: &str = "│  ";
const BLANK: &str = "   ";

/// How deep a row sits under its change. With a single change there is
/// no change row and the artefacts hang from an implicit root.
pub fn depth(row: &Row) -> usize {
    match row {
        Row::Change { .. } => 0,
        Row::Artefact { .. } | Row::Capability { .. } => 1,
        Row::Requirement { .. } => 2,
        Row::Scenario { .. } => 3,
    }
}

/// One guide prefix per row, in row order.
pub fn guides(rows: &[Row]) -> Vec<String> {
    let depths: Vec<usize> = rows.iter().map(depth).collect();
    let last: Vec<bool> = (0..rows.len()).map(|i| is_last(&depths, i)).collect();
    (0..rows.len()).map(|i| prefix(&depths, &last, i)).collect()
}

/// A row is the last sibling when no later row at its depth comes before
/// a row at a shallower depth.
fn is_last(depths: &[usize], i: usize) -> bool {
    let d = depths[i];
    !depths[i + 1..]
        .iter()
        .take_while(|&&x| x >= d)
        .any(|&x| x == d)
}

fn prefix(depths: &[usize], last: &[bool], i: usize) -> String {
    let d = depths[i];
    if d == 0 {
        return String::new();
    }
    let continuation = (1..d).map(|k| {
        let ancestor = (0..i).rev().find(|&j| depths[j] == k);
        match ancestor {
            Some(j) if !last[j] => CONTINUE,
            _ => BLANK,
        }
    });
    let connector = if last[i] { LAST } else { BRANCH };
    continuation.chain(std::iter::once(connector)).collect()
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;

    fn artefact(change: usize, index: usize) -> Row {
        Row::Artefact { change, index }
    }

    fn capability(change: usize, capability: usize) -> Row {
        Row::Capability { change, capability }
    }

    fn requirement(change: usize, capability: usize, index: usize) -> Row {
        Row::Requirement {
            change,
            capability,
            index,
        }
    }

    fn scenario(change: usize, capability: usize, index: usize, scenario: usize) -> Row {
        Row::Scenario {
            change,
            capability,
            index,
            scenario,
        }
    }

    #[test]
    fn the_view_is_a_list_and_a_detail_pane__the_last_requirement_closes_the_branch() {
        let rows = [
            artefact(0, 0),
            capability(0, 0),
            requirement(0, 0, 0),
            requirement(0, 0, 1),
        ];
        assert_eq!(guides(&rows), ["├─ ", "└─ ", "   ├─ ", "   └─ "]);
    }

    #[test]
    fn the_view_is_a_list_and_a_detail_pane__a_scenario_under_a_requirement_with_a_later_sibling() {
        let rows = [
            capability(0, 0),
            requirement(0, 0, 0),
            scenario(0, 0, 0, 0),
            requirement(0, 0, 1),
            capability(0, 1),
            requirement(0, 1, 0),
            scenario(0, 1, 0, 0),
        ];
        assert_eq!(
            guides(&rows),
            [
                "├─ ",
                "│  ├─ ",
                "│  │  └─ ",
                "│  └─ ",
                "└─ ",
                "   └─ ",
                "      └─ "
            ]
        );
    }

    #[test]
    fn the_view_is_a_list_and_a_detail_pane__two_changes_are_the_roots() {
        let rows = [
            Row::Change { change: 0 },
            artefact(0, 0),
            capability(0, 0),
            requirement(0, 0, 0),
            Row::Change { change: 1 },
            capability(1, 0),
            requirement(1, 0, 0),
        ];
        assert_eq!(
            guides(&rows),
            ["", "├─ ", "└─ ", "   └─ ", "", "└─ ", "   └─ "]
        );
    }
}
