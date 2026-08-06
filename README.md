# Phoxal robot-rover

Open exploratory sandbox rover. Example of simple rover using phoxal framework.

The committed root `Cargo.lock` selects the exact framework train. The root
package is this robot's brain: `src/main.rs` declares the one mandatory
composition root with `#[phoxal::brain]`, and the CLI always builds, validates,
and stages it as `bin/brain`. The rover has no mission policy yet, so the brain
is a no-op; robot-specific mission policy, intent selection, and recovery become
ordinary Rust code compiled into that binary.

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
