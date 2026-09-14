# Fixtures

Pairs of canon and delta taken from [Opake](https://github.com/Opake-at/Opake)'s
OpenSpec tree.

| Fixture | Exercises |
| --- | --- |
| `sweep-gate` | A real open change against its canon: an ADDED requirement plus a MODIFIED one whose first paragraph changes a few words, two paragraphs join it, two scenarios are restated with different wording and two are new. |
| `rename-modified` | A RENAMED entry plus a MODIFIED entry under the new name; the body differs by one word. |
| `rewrap` | A requirement restated verbatim with different line breaks: it must show as unchanged. |
| `lint` | A whole repository for the citation lint: two canon specs, one open change, source under two roots with literal and call-form citations, a skipped directory, and `openspec/reviewer.toml`. Copied into a temp dir and initialized as git at test time. |
| `drift` | Two canon specs sharing a backticked identifier, a quoted string and a phrase, plus a delta that drops all three from one of them. |
