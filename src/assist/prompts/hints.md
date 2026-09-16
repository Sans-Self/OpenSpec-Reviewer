## Reply

Reply with a JSON array and nothing else. No prose before it, no prose
after it, no fenced block around it. An empty array is a complete answer
and the expected one when there is nothing to report.

Each element is an object with these fields:

- `kind`, one of `compound_condition`, `uncovered_must`,
  `plain_language`, `term_misuse`, `sibling_invalidated`,
  `rejected_approach`. No other value is read.
- `message`, one line saying what is wrong. Name the thing: the word,
  the clause, the term. Under 120 characters.
- `quote`, optional: the sentence or keyword line the hint is about,
  copied from the text above.
- `scenario`, optional: the name of the scenario the hint is about.

```json
[
  {
    "kind": "compound_condition",
    "message": "the WHEN joins mounting a page and rendering the row",
    "quote": "- **WHEN** a route mounts a page and the index renders",
    "scenario": "Route-mounted page appears as a row"
  }
]
```
