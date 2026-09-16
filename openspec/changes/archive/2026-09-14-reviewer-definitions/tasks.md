# Tasks: reviewer-definitions

## 1. Glossary

- [x] 1.1 `Term` and `Glossary` types; loader over canon `definitions`
      plus the change's own `definitions` delta.
- [x] 1.2 `- **Deprecated:**` parsing; meaning text with the line removed for
      the panel, kept for diffing.
- [x] 1.3 Whole-word case-insensitive matcher with plural tolerance and
      citation stripping.
- [x] 1.4 `[definitions]` config section with `capability` and
      `min_recurrence`; absence is a skip, not a refusal.

## 2. Checks

- [x] 2.1 Deprecated synonym in deltas and canon, reported per the
      placement table in the design.
- [x] 2.2 Defined but unused.
- [x] 2.3 Recurring undefined term, sharing the tier extractor with
      term-drift.
- [x] 2.4 New term without definition, computed over the whole change
      and reported on the first introducer.
- [x] 2.5 Term in use for MODIFIED and RENAMED terms.

## 3. Surfaces

- [x] 3.1 `D` definitions panel in the detail pane, per-pairing toggle.
- [x] 3.2 Findings in plain text, `--findings-only` and JSON; the
      `definitions` array in JSON.
- [x] 3.3 Lint runs the lint-scoped checks and names the glossary in its
      summary line.

## 4. Wrap-up

- [x] 4.1 Fixture glossary and canon lifted from Opake's group key and
      manager vocabulary.
- [x] 4.2 Every requirement cited from at least one test title.
- [x] 4.3 A starter `openspec/specs/definitions/spec.md` for this
      repository itself: pairing, snapshot, source, finding, canon.
