# Tasks: reviewer-ignores

## 1. Register

- [ ] 1.1 The register as a type in `src/model/`: canon plus every open
      change, each entry `<capability> § <requirement name>`, ordered by
      capability then requirement name, built once per run.
- [ ] 1.2 `build_review` and the lint build it from the canon and open
      changes they already load, and pass it to every consumer.
- [ ] 1.3 The coverage ledger enumerates the register instead of
      assembling its own set.

## 2. Configuration

- [ ] 2.1 `[[definitions.ignore]]` and `[[lint.ignore_uncited]]` parsed
      into typed entries; a missing `reason` is a configuration error
      naming the entry.
- [ ] 2.2 `in` parses as an array of scopes, each a capability or a
      `<capability> § <requirement name>`; a bare string is an error.
- [ ] 2.3 `render` writes both sections as commented examples, so the
      refusal's example and `lint init` show the shape.

## 3. Checks

- [ ] 3.1 `recurring_undefined` and `new_undefined` take the ignore list
      and the scope of the text under check, skipping an exact term
      inside its scopes and never a prefix or a pattern.
- [ ] 3.2 The ledger keeps an ignored requirement out of the marked lines
      and in `<total>`, with `coverage: <cited>/<total> (<n> ignored)`.
- [ ] 3.3 The stale-entry routine in `src/citations/structure.rs`
      generalizes over any configured list and reports a dangling entry
      as a warning naming the entry, the scope at fault and
      `openspec/reviewer.toml`, judging scopes one at a time.
- [ ] 3.4 `lint` and `change` reach the same dangling verdict, covered by
      a test that runs both over one fixture.

## 4. Wrap-up

- [ ] 4.1 Fixture config with one entry of each kind, one scoped to two
      capabilities, one dangling.
- [ ] 4.2 Every requirement of this change cited from at least one test
      title.
