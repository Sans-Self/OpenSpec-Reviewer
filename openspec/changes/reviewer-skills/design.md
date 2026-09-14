# Design: reviewer-skills

## Context

`.claude/skills/` in this repository already holds skills the OpenSpec
CLI generated, each a directory with a `SKILL.md` whose frontmatter
names the skill, its description, allowed tools and a generator version.
The reviewer's four skills follow that layout exactly, so an agent sees
one convention. `reviewer-assist` will ship prompt templates under
`openspec/reviewer/prompts/`; skills are the second consumer of that
directory.

## Goals / Non-Goals

**Goals:** an agent can run the reviewer and act on its JSON without
being taught; a project can rewrite a skill's instructions without
forking the tool; a skill can never drift silently from the requirement
it depends on.

**Non-Goals:** a plugin system, network installs, agent flavours beyond
Claude Code.

## Decisions

**Skills are files in the binary.** Each skill is one `SKILL.md`
included with `include_str!`, under `src/skills/claude/<name>/`. Install
writes `.claude/skills/opsx-reviewer-<name>/SKILL.md`. The frontmatter
carries `metadata.generatedBy: openspec-reviewer <version>` and
`metadata.checksum`, the hash of the body as written. On a second
install a file whose checksum matches its recorded one is overwritten;
a file that differs is left alone and named in the output, because
someone edited it by hand and the tool does not know better.

**Override is body replacement, not merge.** A project file at
`openspec/reviewer/skills/<name>.md` is the body the installed skill
gets; the shipped frontmatter is kept so the agent still sees the right
name and tools. Merging two prose documents is a judgment the tool
should not make; replacing one with the other is a fact it can record
in the frontmatter as `metadata.override: openspec/reviewer/skills/<name>.md`.

**Skills cite their requirements.** Each body ends with a "Depends on"
list of `spec:` citations into the reviewer's own canon, `citations`,
`glossary`, `term-drift`, `review-findings`. When the reviewer is
installed into another repository those citations do not resolve there,
and the lint would report them. The install therefore rewrites them to
plain text unless the target repository is this one, detected by the
presence of `openspec/specs/glossary/spec.md` with the requirement that
names the check. Cheaper than teaching the lint about foreign canon, and
honest: in a foreign repository the citations are documentation, not
contracts.

**Every skill writes a change.** `define` writes
`openspec/changes/<name>/specs/definitions/spec.md` under ADDED;
`crossref` and `triage` write MODIFIED entries into the change under
review's own deltas; `cite` writes to test files, which are code, not
spec, and the one place a skill edits outside `openspec/changes/`. None
writes under `openspec/specs/`. The skill text says so in its first
paragraph, and the reviewer's own review of the resulting change is the
approval step.

**Triage is a router.** Its body carries the table of finding kinds to
usual fixes: a dangling citation wants the nearest canon name, a missing
scenario wants one drafted from the body, a removed-still-cited wants a
decision between the test and the requirement. Kinds that need reading,
`sibling_uses_removed`, `sibling_uses_old_name`, `term_in_use`,
`modified_has_citers`, are listed and handed to crossref rather than
fixed. Every proposed edit is shown before it is applied, and nothing
is dismissed: dismissal is a key in the view, for a person.

**Crossref has a measurable exit.** Its final step re-runs the review
and reports drift warnings before and after. A sibling the skill drafted
a delta for, without the removed term, stops warning by the reviewer's
own rule, so the count going down is the skill's evidence and the
warnings left are the human's.

**`lint init` learns `.claude`.** When `.claude/` exists, `init` adds it
to `source_roots` and `md` to `source_globs`, so installed skills are
scanned as citers. One more entry in the candidate roots list.

## Risks / Trade-offs

- A skill's instructions can go stale against the binary's behaviour
  when a requirement changes. → The citation rule turns that into a
  lint error here, where the skills live.
- Agents vary in how faithfully they follow a "confirm before writing"
  instruction. → Every skill writes into a change, and the change is
  reviewed with the tool that motivated the skill; canon is never one
  keystroke away.

## Testing

Install and list are tested against a temp repository: fresh install,
second install with one hand-edited file, an override file present,
this repository versus a foreign one for the citation rewrite. Skill
bodies are tested for shape: frontmatter fields, the "writes a change"
paragraph, the "Depends on" list resolving against canon. The skills'
behaviour is prose an agent follows and is not executed by tests.
