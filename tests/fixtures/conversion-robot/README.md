# Conversion qualification robot

This robot executes ordinary Rust conversions inside the canonical brain runtime.
Its process tests exercise source capture provenance, freshness, real conversion failure, and reset behavior with World and Navigation.
Prepare and build its bundle before starting Cargo tests so the test process never recursively acquires its own Cargo build lock.

```sh
cargo phoxal prepare
cargo phoxal build --output /tmp/phoxal-conversion-bundle
PHOXAL_CONVERSION_BUNDLE=/tmp/phoxal-conversion-bundle cargo test --test composition -- --ignored --nocapture
```

Linux and macOS are supported.
Windows is unsupported.
