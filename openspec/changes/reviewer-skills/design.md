# Design: reviewer-skills

## Context

`.claude/skills/` in this repository already holds skills the OpenSpec
CLI generated, each a directory with a `SKILL.md` whose frontmatter
names the skill, its description, allowed tools and a generator version.
The reviewer's skills follow that layout exactly, so an agent sees one
convention. The same layout is read by Codex, OpenCode and omp from
`.agents/skills/`, and OpenCode and omp read `.claude/skills/` as well;
the frontmatter they all require is `name` and `description`. Commands
have no such convergence: each CLI has its own directory and Codex has
deprecated the concept. `reviewer-assist` will ship prompt templates
under `openspec/reviewer/prompts/`; skills are the second consumer of
that directory.

## Goals / Non-Goals

**Goals:** an agent can run the reviewer and act on its JSON without
being taught; a user can call the four task skills by a namespaced
name; a project can rewrite a skill's instructions without forking the
tool; a skill can never drift silently from the requirement it depends
on.

**Non-Goals:** a plugin system, network installs, command syntaxes
beyond Claude Code's.

## Decisions

**Skills are the artefact, commands are pointers to them.** A skill is
loaded by the agent from its description or by the user by name, and
the body is the same file in every CLI that reads `SKILL.md`. A command
is user-only, one directory per CLI, and in Claude Code the only way to
get a colon-namespaced name. So each of the four task skills gets one
command file, `.claude/commands/opsx-reviewer/<name>.md`, whose body
tells the agent to load the skill and pass `$ARGUMENTS` along. The
instructions live once, in the skill; the command is an address. omp
reads Claude's command directory with the same `ns:name` alias, so the
address carries there for free.

**The workflow skill has no command.** It is the one skill written for
the agent, not the user: when to run which reviewer command, how to
read the output, which task skill answers which finding. Its
description is tuned for implicit invocation, "working in a repository
with `openspec/` and an `openspec-reviewer` binary", and it does not
opt out of model invocation. The four task skills each state in their
description that they change files on confirmation, so an agent that
loads one uninvited still asks first.

**Skills are files in the binary.** Each skill is one `SKILL.md`
included with `include_str!`, under `src/skills/<name>/`. Install
writes `.claude/skills/opsx-reviewer-<name>/SKILL.md` and, when
`.agents/` exists, the same file under `.agents/skills/`. The install
never creates `.agents/`: a directory another CLI owns is not the
reviewer's to introduce. The frontmatter carries
`metadata.generatedBy: openspec-reviewer <version>` and
`metadata.checksum`, the hash of the body as written. On a second
install a file whose checksum matches its recorded one is overwritten;
a file that differs is left alone and named in the output, because
someone edited it by hand and the tool does not know better. Command
files carry the same two fields and follow the same rule.

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
spec, and the one place a skill edits outside `openspec/changes/`;
`workflow` writes nothing itself and says which skill does. None writes
under `openspec/specs/`. The skill text says so in its first paragraph,
and the reviewer's own review of the resulting change is the approval
step.

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

**`lint init` learns `.claude` and `.agents`.** When either directory
exists, `init` adds it to `source_roots` and `md` to `source_globs`, so
installed skills are scanned as citers. Two more entries in the
candidate roots list.

## Risks / Trade-offs

- A skill's instructions can go stale against the binary's behaviour
  when a requirement changes. → The citation rule turns that into a
  lint error here, where the skills live.
- Agents vary in how faithfully they follow a "confirm before writing"
  instruction. → Every skill writes into a change, and the change is
  reviewed with the tool that motivated the skill; canon is never one
  keystroke away.
- The workflow skill loads implicitly, so a wrong description costs
  context in every session. → The description names two concrete
  markers, the `openspec/` directory and the binary, and nothing
  broader.

## Testing

Install and list are tested against a temp repository: fresh install
with and without `.agents/`, second install with one hand-edited file,
an override file present, this repository versus a foreign one for the
citation rewrite, command files present for four skills and absent for
the workflow. Skill bodies are tested for shape: frontmatter fields,
the "writes a change" paragraph, the "Depends on" list resolving
against canon, the workflow body naming every finding kind the review
can emit. The skills' behaviour is prose an agent follows and is not
executed by tests.
