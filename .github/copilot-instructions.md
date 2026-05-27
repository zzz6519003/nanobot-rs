# Copilot Repository Instructions

For any code change that is ready to commit, run these checks first:

```bash
just fmt-check
taplo lint
cargo clippy --all-targets --all-features -- -D warnings
```

Before `git push`, also run:

```bash
cargo test --all-targets --all-features
```

Do not bypass these checks unless explicitly requested by the user.

