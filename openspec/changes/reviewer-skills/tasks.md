# Tasks: reviewer-skills

## 1. Store and install

- [ ] 1.1 `src/skills/`: the four `SKILL.md` bodies embedded, frontmatter
      with name, description, allowed tools, version and body checksum.
- [ ] 1.2 `skills install`: writes under `.claude/skills/`, overwrites
      only checksum-matching files, names kept files, refuses without
      `.claude/` with exit `2`.
- [ ] 1.3 `skills list`: name, description, installed / up to date /
      edited.
- [ ] 1.4 Override from `openspec/reviewer/skills/<name>.md`, recorded in
      frontmatter.
- [ ] 1.5 Citation rewrite when the target is not this repository.

## 2. Skill bodies

- [ ] 2.1 `define`: lint JSON, candidate sorting, term drafting into a
      new change, closing review.
- [ ] 2.2 `cite`: coverage JSON, test search, citation placement,
      before-and-after coverage line.
- [ ] 2.3 `crossref`: finding intake, sibling reading, three verdicts,
      archive search and quote, MODIFIED drafts on confirmation,
      before-and-after drift count.
- [ ] 2.4 `triage`: severity order, fix table, show-then-apply, routing
      to crossref, no dismissal, before-and-after summary.
- [ ] 2.5 Every body opens with where it writes and ends with a "Depends
      on" citation list.

## 3. Wrap-up

- [ ] 3.1 `lint init` adds `.claude` to the source roots when present.
- [ ] 3.2 Tests titled by requirement: install, list, override, citation
      rewrite, and the shape checks over every body.
- [ ] 3.3 Skills installed into this repository and passing `lint`.
- [ ] 3.4 README section on skills, and `openspec/changes/reviewer-assist/design.md`
      names `openspec/reviewer/` as the shared store.
