# Tasks: lint-init

## 1. Template

- [x] 1.1 `Survey` type and `render(&Survey) -> String` in
      `citations::config`; the refusal message renders the default
      survey; a test parses the rendered text with `parse_config`.

## 2. Survey

- [x] 2.1 `survey(root) -> Survey` in `source::lint`: candidate roots that
      exist, extensions found under them, `docs` presence.

## 3. Command

- [x] 3.1 `lint init` subcommand: refuses without `openspec/`, refuses
      when the file exists, otherwise writes the rendered survey with
      `create_new` and prints the path.
- [x] 3.2 The no-config refusal names `lint init`.

## 4. Wrap-up

- [x] 4.1 Tests for the four `init` scenarios and the changed refusal,
      titled by requirement.
- [x] 4.2 README Citations section opens with `lint init`.
