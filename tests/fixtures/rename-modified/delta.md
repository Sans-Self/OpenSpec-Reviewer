## RENAMED Requirements

- FROM: `### Requirement: Clients act on the outcome, never on URI matching`
- TO: `### Requirement: Clients dispatch on the outcome, never on URI matching`

## MODIFIED Requirements

### Requirement: Clients dispatch on the outcome, never on URI matching

A client consuming `at.opake.keyring:delete` SHALL dispatch on the payload's outcome: `unchanged` and `rolled_back` leave tracked workspace state untouched (the rollback's follow-up upsert carries the rebuild), and `torn_down` removes the entry keyed by the payload's `workspace_id` — the genesis URI, per `spec:workspace-identity § Genesis URI is the workspace identity`. Clients SHALL NOT compare the deleted `uri` against tracked keys to decide whether a workspace ended; the dispatch-side rule is `spec:workspace-identity § SSE keyring dispatch keys on derived genesis`.

A missing or unrecognized `outcome` SHALL deserialize as `unchanged` — under version skew a client may under-react (stale until the next event or bootstrap) but never wrongly drop a living workspace.

#### Scenario: genesis delete does not drop the sidebar entry

- **GIVEN** a member's connected client tracking a workspace whose chain has superseded past genesis
- **WHEN** the genesis record's delete event arrives (`outcome: unchanged`, `uri` equal to the tracked `workspace_id`)
- **THEN** the workspace remains in the keeper
- Regression: `bug__genesis_delete_tombstone_drops_living_workspace` (crates/opake-core/src/indexer/workspace_keeper/tests.rs)

#### Scenario: teardown drops the entry by workspace_id

- **GIVEN** a member's connected client tracking a sole-record workspace
- **WHEN** the delete event arrives with `outcome: torn_down`
- **THEN** the keeper entry keyed by the payload's `workspace_id` is removed, matching what the next bootstrap would show
