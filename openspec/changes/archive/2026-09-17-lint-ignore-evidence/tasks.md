# Tasks: lint-ignore-evidence

## 1. Configuration

- [x] 1.1 `[[lint.ignore_evidence]]` parses into entries carrying one of
      `citation`, `path`, `test` or `commit`, and `reason`.
- [x] 1.2 An entry with no kind, with two kinds, or without a reason is a
      configuration error naming the entry, at exit `2`.

## 2. The lint

- [x] 2.1 The cited-path, cited-test, cited-commit and dangling-citation
      findings drop the ones an entry names, matched as exact strings.
- [x] 2.2 An entry that silenced nothing is a warning naming the entry and
      `openspec/reviewer.toml`.

## 3. This repository

- [x] 3.1 `openspec/reviewer.toml` carries an entry per example the specs
      quote, each with its reason.
- [x] 3.2 `openspec-reviewer lint` reports no errors here.
