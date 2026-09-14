# Tasks: glossary-term-binding

## 1. Parsing

- [ ] 1.1 The marker parser recognizes `A spec MUST use <term> to mean:`
      as the binding line, backticked or bare, and lifts it out of the
      meaning while leaving it in the body a pairing diffs.
- [ ] 1.2 `Term::from_requirement` records whether the binding line is
      present and whether it names the requirement.

## 2. Check

- [ ] 2.1 A term with no binding line, or one naming another term, is a
      warning in `lint` and on the term's pairing.

## 3. Canon and skill

- [ ] 3.1 The five terms in `openspec/specs/definitions/spec.md` gain the
      line.
- [ ] 3.2 `src/skills/define.md` drafts the line.

## 4. Regression

- [ ] 4.1 `bug__drafted_term_fails_validation`: write the shape
      `/opsx-reviewer:define` instructs into a change, run
      `openspec validate`, assert no error. Fails before task 3.2.
- [ ] 4.2 The same assertion over every shipped skill that writes an
      artefact, so an instructed shape that stops validating fails the
      suite rather than reaching a user.

## 5. Wrap-up

- [ ] 5.1 Fixture glossary gains the line, plus one term without it.
- [ ] 5.2 Every requirement of this change cited from at least one test
      title.
