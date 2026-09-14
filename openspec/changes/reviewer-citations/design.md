# Design: reviewer-citations

## Shape

```
src/
  citations/
    mod.rs        Citation, Citer, CitationIndex
    grammar.rs    literal and call-form regexes
    scan.rs       walk specs, deltas, source roots
    evidence.rs   path, test-name, hash checks
    radius.rs     REMOVED / MODIFIED blast radius per change
    coverage.rs   the ledger
    config.rs     openspec/reviewer.toml
  drift/
    terms.rs      removed-term extraction in three tiers
    siblings.rs   search over canon and the change's own deltas
    filter.rs     stop list and commonness threshold
  review/
    findings.rs   gains the citation and drift finding kinds
```

`citations::scan` produces a `CitationIndex`; everything after it is a
pure function over the index, canon and the open changes. The `lint`
subcommand and the review join both consume the same index, so they
cannot disagree.

## Index

```rust
struct Citation { capability: String, requirement: String /* normalized */ }
struct Citer    { file: PathBuf, citing_capability: Option<String> }

struct CitationIndex {
    citers:    BTreeMap<Citation, Vec<Citer>>,
    in_flight: BTreeSet<Citation>,          // ADDED by open changes
    canon:     BTreeMap<String, BTreeSet<String>>,
}
```

Resolution is `in_flight.contains(c) || canon[c.capability].contains(c.requirement)`.
The in-flight set is the reason a test can cite a requirement before the
change syncs; an abandoned change makes those cites dangle on the next
run, which is the intended pressure.

## Grammar

Two regexes, ported as-is from the TypeScript lint because their edge
cases were earned:

- Literal: `spec:([a-z0-9-]+)\s*§\s*([^"`\n]+)`. The name runs to a
  double quote, backtick or newline. A single quote does not terminate,
  so `the page's route` survives; the cost is that single-quoted
  citations are unsupported, which the rel-monorepo convention already
  states.
- Call: `<helper>\(\s*['"]([a-z0-9-]+)['"]\s*,\s*(?:"([^"]+)"|'([^']+)')\s*,?\s*\)`.
  Either quote style, tolerant of a trailing comma and line breaks,
  because prettier picks the quote that avoids escapes.

`<helper>` is interpolated from config after `regex::escape`.

## Evidence

Path and test-name detection use configured prefixes, extensions and
pattern. The path regex uses a negative lookbehind for `[\w./-]` so a
prefix starting with `.` still matches at a word boundary; `regex` has no
lookbehind, so the port checks the preceding character by hand after a
match. Test names are looked up with `git grep -q --fixed-strings` over
the configured globs, the same call the script makes, because it is
faster than walking and reading every file for each name. Hashes go to
`git cat-file -e <hash>^{commit}`.

## Blast radius

Per open change, per capability delta:

- REMOVED `R`: for each citer of `(cap, R)`, skip when
  `citer.citing_capability == cap` or the change has a delta for
  `citer.citing_capability`; otherwise an error.
- MODIFIED `R`: collect citers with `citing_capability != cap`; if any,
  one note listing them.

The same function returns `Vec<Finding>` keyed by `(change, cap, R)`. The
`lint` subcommand prints them; the review attaches those whose change is
under review to the matching pairing.

## Configuration

`openspec/reviewer.toml`, sitting next to the OpenSpec CLI's own
`config.yaml`:

```toml
[lint]
source_roots     = ["apps", "packages"]
source_globs     = ["**/*.ts", "**/*.tsx", "**/*.mts"]
skip_dirs        = ["node_modules", ".next", "dist", "test-results", "playwright-report"]
path_prefixes    = ["apps", "packages", "docs", "bin", ".github", ".claude"]
path_extensions  = ["ts", "tsx", "mts", "mjs", "md", "json", "css", "yml", "yaml"]
test_pattern     = "bug__\\w+"
cite_helper      = "cite"
change_scopes    = ["ui", "energiehuis", "auth", "infra", "spec"]
grandfathered    = ["page-editor", "sitemap-dashboard"]
```

Parsed with `serde` + `toml` and `#[serde(deny_unknown_fields)]`, so a
typo in a key is an error naming the key rather than a silently skipped
check. The file itself is required: a repository that has not said where
its source lives gets a refusal with a template, not a lint that quietly
checks half of what it could. Inside the file every field is optional.
Absent `source_roots` means no source scan and no test-name check;
absent `path_prefixes` means no path check; absent `change_scopes` means
no name check. The summary line names what a present file left out.

`change_scopes` is a literal list rather than a pointer into
`.github/git-conventions.json` because the tool should not know about
one repository's CI conventions. rel-monorepo can generate the list into
the toml from its JSON in one line of its own tooling.

## Source walk

The `ignore` crate walks roots honouring `.gitignore`, which subsumes
most of `skip_dirs`; the configured list is applied on top for
directories that are committed but never contain citations. Globs are
matched with `ignore`'s own `overrides`.

## Review join

The foundation's `Finding` enum gains:

```
CitationDangling     error    a citation in this delta does not resolve
RemovedStillCited    error    REMOVED requirement has outside citers
ModifiedHasCiters    note     MODIFIED requirement has outside citers
SiblingUsesRemoved   warning  a canon requirement elsewhere still uses a removed term
SiblingUsesOldName   warning  a canon requirement elsewhere still uses a renamed name
```

Each carries `Vec<PathBuf>` of citing files, rendered as an indented list
under the finding in the detail pane and in plain text, and as an array
in JSON.

## Term drift

Input is the foundation's `Pairing`: before and after text, both
normalized. Extraction runs on the concatenated body and scenario text of
each side.

```rust
enum Tier { Backticked, Quoted, Phrase }
struct RemovedTerm { text: String, tier: Tier }
```

- Backticked: every `` `…` `` span on the before side whose exact text
  does not occur on the after side.
- Quoted: every `"…"` span, same rule.
- Phrase: word n-grams for n in 2..=4 over the before side, minus any
  n-gram present on the after side, minus n-grams that are all stop words
  or spec keywords, minus n-grams shorter than 8 characters. Longer
  n-grams that contain a surviving shorter one are kept; shorter n-grams
  contained in a surviving longer one are dropped, so "status badge" is
  reported once, not as "status", "badge" and "the status badge".

The search is a case-insensitive substring match over each canon
requirement's normalized text outside the pairing's capability, then
over the change's own deltas for those capabilities to decide whether the
hit is already handled. Canon is at most a few hundred requirements, so
this is a linear scan per term with no index.

Commonness: a phrase found in more than `term_drift.max_common` canon
requirements is noise, not drift, and is dropped before search. Backticked
and quoted tiers skip the threshold because an identifier used in ten
places that one change removes is exactly the case worth ten findings.

Old-name search for RENAMED pairings strips `spec:…§…` citations from the
sibling text before matching so the citation lint and drift never report
the same hit twice.

Drift runs only in the review of a change, not in `lint`, because it
needs a pairing to know what was removed.

## Dependencies

| Crate | Why |
| --- | --- |
| `toml` | The config file. |
| `regex` | The two citation grammars and the configured test pattern. |
| `ignore` | Gitignore-aware directory walk with glob overrides. |

The drift kinds carry the term and a `Vec<Sibling { capability, requirement, path }>`.

## Testing

A fixture repository under `tests/fixtures/lint/` with two canon specs,
one open change and a handful of source files, initialized as a git
repository in a temp dir at test time so hash and `git grep` checks run
for real. One test per requirement, the rel-monorepo edge cases as
regression tests: apostrophe in a cited name, dotfile prefix, spec-first
in-flight cite, grandfathered change removed. Drift is tested on the
sitemap-labels fixture the foundation already carries: `mountType:
'feature'` and "status badge" removed, a sibling that still has both.
