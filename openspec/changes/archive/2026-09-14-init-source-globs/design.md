# Design: init-source-globs

## Decision

The renderer, not the survey, drops `json` from `source_globs`. The
survey reports what exists; what to read is a rendering decision, and
keeping it in `render` keeps the refusal's example and the written file
in step. The exclusion is a constant list of one entry, `NO_CITATIONS`,
so a second comment-less format joins it in one line.

The rule is "formats with nowhere to write a citation", not "code
formats". A CSS comment or a YAML comment can cite a requirement and the
lint should find it.
