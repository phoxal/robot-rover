# Phoxal robot-rover

Open exploratory sandbox rover. Example of simple rover using phoxal framework.

The committed root `Cargo.lock` selects the exact framework train. The root
package is intentionally empty: it keeps the `phoxal` dependency in the real
Cargo graph even while this project has no user-authored Rust services.

## Editor schemas

`robot.yaml` opens with a `# $schema:` comment pointing at a generated schema
that is not committed. Populate it after cloning, and again after upgrading the
CLI, so the association resolves:

```sh
phoxal schema generate
```

JetBrains IDEs and current `yaml-language-server` clients read that comment and
give completion and unknown-property inspection from it. The schemas are a
structural aid only - `phoxal validate` remains authoritative.

## License

MIT

## Phoxal

- <https://phoxal.com>
- <https://github.com/phoxal>
