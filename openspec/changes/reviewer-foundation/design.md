# Design: reviewer-foundation

## Shape

One crate, one binary. The domain is pure and the edges are thin:

```
src/
  main.rs            clap entry: global flags + one subcommand per source
  source/
    mod.rs           Source trait, Snapshot, SourceError
    change.rs        working tree
    diff.rs          unified diff → Snapshot (also used by gh)
    git.rs           two refs via git show
    gh.rs            gh pr diff → diff.rs
  model/             canon + delta parsing: Requirement, Scenario, DeltaKind
  review/
    pair.rs          pairing, rename joining
    diff.rs          normalisation, paragraph + word diff, scenario matching
    findings.rs      the closed Finding enum and every check
    history.rs       archive walk with rename chasing
  state/             approvals + notes, XDG store, stale detection
  render/
    tui/             ratatui app: list, detail, history, help
    text.rs          plain text
    json.rs          serde output
    markdown.rs      notes export
```

`model`, `review` and `state` never touch a terminal. `model` and
`review` never touch the filesystem either: they take strings in and
return values out, so every rule in the specs is a unit test over
fixture strings.

## Sources

```rust
pub struct FileChange { pub path: PathBuf, pub before: Option<String>, pub after: Option<String> }
pub struct Snapshot   { pub files: Vec<FileChange>, pub origin: String /* for the status line */ }

pub trait Source {
    fn fetch(&self) -> Result<Snapshot, SourceError>;
}
```

The trait is deliberately one method. A source is anything that can say
"these openspec files, before and after". The four shipped sources:

| Source | before | after |
| --- | --- | --- |
| `change` | none | working-tree file |
| `diff` | working-tree file | working-tree file with hunks applied (`diffy`) |
| `git` | `git show base:path` | `git show ref:path` |
| `gh` | delegates to `diff` with `gh pr diff` output | |

`--base` defaults to `main`, then `master`, then an error asking for it.
No merge-base guessing: the reviewer names the target or the tool stops.

`git` reads whole files from both refs rather than applying a patch, so it
never hits a pre-image mismatch. `diff` has to apply hunks because a
patch on stdin is all it has; a mismatch is a hard error.

A tangled source would fetch the PR's diff over HTTP and hand it to
`diff`, or read both sides from its API and build the snapshot directly.
Either way it is one file under `source/` plus one clap variant.

Canon is always the working directory's `openspec/specs/`. The reviewer
runs the tool from the checkout they are reviewing into, and that
checkout's canon is what the change will be synced onto.

## Why blocking I/O

Each run does one fetch, then pure computation, then rendering. There is
nothing to overlap. Tokio would add a runtime and colour every function
signature for zero concurrency. The one wait a user feels is `gh` in the
TUI, and that is a spinner over a fetch on a thread, not an async
problem. A future HTTP source uses blocking `reqwest`. If a source ever
needs to fetch many things concurrently, that source can own a runtime
internally without the rest of the crate knowing.

## Data model

```rust
struct Requirement { name: String, body: String, scenarios: Vec<Scenario> }
struct Scenario    { name: String, body: String }

enum DeltaKind { Added, Modified, Removed, Renamed { from: String } }

struct DeltaEntry { kind: DeltaKind, requirement: Requirement }
struct DeltaSpec  { capability: String, entries: Vec<DeltaEntry> }
struct Canon      { specs: BTreeMap<String, Vec<Requirement>> }

struct Pairing {
    capability: String,
    kind: DeltaKind,
    before: Option<Requirement>,
    after:  Option<Requirement>,
    findings: Vec<Finding>,
    history: Vec<HistoryEntry>,
    state: ItemState,
}
```

A `RENAMED` block pairs with the `MODIFIED` entry that carries the new
name, if there is one. The pairing then has `before` = canon under the old
name, `after` = the modified body. A rename with no matching MODIFIED
entry pairs the canon body with itself and shows only the name change.

## Parsing

Spec files are line-oriented. The grammar the tool cares about:

```
## ADDED Requirements | ## MODIFIED Requirements | ## REMOVED Requirements | ## RENAMED Requirements
### Requirement: <name>
#### Scenario: <name>
- FROM: `### Requirement: <name>`      (RENAMED only)
- TO:   `### Requirement: <name>`      (RENAMED only)
```

Everything else is body text attached to the nearest requirement or
scenario above it. Canon files use the same headings under
`## Requirements`. One hand-written line scanner covers both; no
Markdown library.

## Diffing

`similar` does the work. Two passes:

1. **Normalize.** Join hard-wrapped lines inside a paragraph into one
   line, collapse runs of spaces, keep list items and blank lines as
   paragraph boundaries. This is what makes a pure re-wrap show as
   unchanged. Both sides are normalized before comparison and the
   rendered text is the normalized text.
2. **Diff.** Paragraph-level Myers diff first; for paragraphs that pair
   as changed, a word-level inline diff. Scenarios are matched by name
   before any diffing, so a scenario present on one side only is a
   finding rather than a diff.

## Findings

Each finding has a severity, a location and a one-line message. The set
is closed and lives in one enum so the JSON output has a stable
vocabulary:

```
ModifiedWithoutCanon       error    MODIFIED names a requirement canon does not have
AddedAlreadyExists         error    ADDED names a requirement canon already has
RemovedWithoutCanon        error    REMOVED names a requirement canon does not have
RenameSourceMissing        error    RENAMED FROM names a requirement canon does not have
RenameTargetTaken          error    RENAMED TO collides with an existing canon name
ScenarioDropped            warning  after side lacks a scenario canon has
RequirementWithoutScenario warning  a requirement has zero scenarios
CrossChangeCollision       warning  another open change touches the same requirement
UnchangedModified          note     MODIFIED body equals canon after normalization
```

## State

```rust
struct ItemState { approved: Option<Approval>, note: Option<String> }
struct Approval  { text_hash: u64, at: String /* RFC 3339 */ }
```

Stored as one JSON file per repository and change under
`$XDG_STATE_HOME/openspec-reviewer/<repo>/<change>.json`. `<repo>` is the
origin URL with every non-path-safe character replaced by `_`, falling
back to the absolute top-level path treated the same way. Items are
keyed by `capability/requirement`, artefacts by their filename. The hash
is over the normalized after text; a stale approval is one whose hash
no longer matches. The store is written after every toggle and every
note edit; there is no save step to forget.

Notes are edited in `$EDITOR`. The TUI leaves the alternate screen,
runs the editor on a temp file seeded with the current note, reads it
back, and re-enters. A line-editor inside ratatui is not worth building
for multi-line prose.

## History

For a pairing `(capability, name)`, walk `openspec/changes/archive/*`
sorted by directory name. For each archive with
`specs/<capability>/spec.md`, parse the delta and take entries named
`name`. When an entry is `RENAMED { from }` to `name`, push it and
continue the walk with `from` as the name for archives earlier than that
one. The result is oldest-first; the version under review is appended as
`current`. Archives that fail to parse are skipped with a note finding
on the pairing, not an error: history is context, not gate.

## Rendering

The TUI is a list on the left and a detail pane on the right. Rows are
artefacts, then capability headings with their requirement pairings. A
row reads `[√] ~ Flat index of all routes and pages ?✎`: approval mark,
kind glyph, name, finding marker, note marker. The detail pane has three
modes cycled with `m`: inline word diff, side-by-side, raw. Under the
diff: findings, note, one-line history. `H` swaps the whole view for the
history view (entries left, version or diff-to-previous right); `Esc`
returns. Colour is additive: every change marker also has a glyph so
`NO_COLOR` loses nothing.

Plain text mirrors the inline mode. JSON is the `Vec<Pairing>` plus the
summary, serialized as-is. Markdown is notes only, for pasting into a
review.

## Dependencies

| Crate | Why |
| --- | --- |
| `clap` | Subcommands and flags. |
| `ratatui` + `crossterm` | The TUI. |
| `similar` | Word-level and paragraph-level diffs with inline change grouping. |
| `diffy` | Parsing and applying unified diffs; hunk offsets and context matching are the fiddly part. |
| `serde` + `serde_json` | JSON output and the state file. |
| `thiserror` | Error enums. |
| `directories` | XDG state dir resolution across platforms. |

No Markdown parser, no async runtime, no HTTP client.

## Testing

Fixtures are pairs of canon and delta strings under `tests/fixtures/`,
with the expected pairing and findings next to them. The first fixture
set is lifted from real changes in `rel-monorepo`: the sitemap-labels
MODIFIED that motivated the tool, a RENAMED + MODIFIED pair, and a pure
re-wrap that must show as unchanged. Sources are tested against a
throwaway git repository built in a temp dir. Every requirement in the
specs is cited from at least one test title.
