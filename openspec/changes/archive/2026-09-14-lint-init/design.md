# Design: lint-init

## Context

`citations::config` holds a static `TEMPLATE` string and the refusal that
prints it. `source::lint::Workspace` already walks configured roots with
the `ignore` crate. `init` sits between the two: it has to look at the
repository before there is a configuration telling it where to look.

## Goals / Non-Goals

**Goals:** one command from "no config" to "a config that runs and is
mostly right". Every guess is a measurement of the working tree, so the
written file needs no explanation beyond its own comments.

**Non-Goals:** merging into an existing file, interactive prompts,
guessing change scopes.

## Decisions

**The template is rendered from a `Survey`, not edited as text.**
`citations::config::render(&Survey) -> String` takes
`Survey { roots: Vec<String>, extensions: BTreeSet<String>, has_docs: bool }`
and returns the TOML. The refusal message calls it with a default survey
for this repository's shape, so the minimal example the spec requires and
the file `init` writes come from one function and cannot drift. The
rendered text is parsed back through `parse_config` in a test so the
template can never fall out of step with the schema.

**Surveying is a walk over the candidate roots only.** `source::lint`
gains `survey(root) -> Survey`: for each candidate directory that exists,
walk it with the same `ignore` builder the lint uses, `skip_dirs` fixed
to the default list, and collect the extensions of regular files that
appear in the candidate extension set. Candidates are a fixed list in
the spec; anything outside it is not a citation target the lint knows
what to do with. The walk stops early per extension once seen, so a
large tree costs one pass.

**Extensions drive both `source_globs` and `path_extensions`.** A glob
per extension, `**/*.<ext>`, is verbose and obvious, which beats a
brace pattern the reader has to decode. `path_extensions` gets the same
set plus `md`, because specs cite documentation whether or not source
roots contain it.

**Refusals are values.** `ConfigError` gains `Exists { path }` and
`NoOpenSpec { dir }`; `init` returns them and `main` prints them like
every other error with exit `2`. `init` never touches an existing file:
it opens with `create_new`, so the check and the write are one syscall.

## Risks / Trade-offs

- A monorepo whose source lives outside the candidate list gets an empty
  `source_roots`. The written file says so in a comment above the field,
  and the lint's summary line names the field as unconfigured. →
  Acceptable: the reader edits one line instead of writing the file.
- `tests` as a source root doubles as a fixture directory in some repos
  and will be scanned for citations. → That is the intended behaviour;
  fixtures that cite are citers.
