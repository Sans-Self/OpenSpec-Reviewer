# input-sources (delta)

## MODIFIED Requirements

### Requirement: Canon files in a snapshot are shown as plain diffs

When a snapshot holds a file under `openspec/specs/`, the tool MUST list
it under a canon heading as a plain line diff. It MUST NOT match those
lines to a change.

#### Scenario: Direct canon edit

- **WHEN** the snapshot modifies `openspec/specs/alpha/spec.md`
- **THEN** the review shows that file's line diff under canon
- **AND** no requirement pairing is made from it
