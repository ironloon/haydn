# Release Checklist — v0.0.1

## Pre-release

- [ ] All tests pass locally: `cargo test --workspace`
- [ ] `cargo publish --dry-run -p haydn-vm` passes
- [ ] `cargo publish --dry-run --no-verify -p haydn-tuning` passes
- [ ] `cargo publish --dry-run --no-verify -p haydn-audio` passes
- [ ] `cargo publish --dry-run --no-verify -p haydn-performer` passes
- [ ] `CARGO_REGISTRY_TOKEN` configured as GitHub repo secret
- [ ] Repository URLs consistent across all Cargo.toml files (`jwgeller/haydn`)

## Release

- [ ] Create and push git tag: `git tag v0.0.1 && git push origin v0.0.1`
- [ ] Wait for CI to pass on the tag (GitHub Actions)
- [ ] Review draft GitHub Release (auto-created by CI)
- [ ] Click "Publish release" to trigger crate publishing
- [ ] Verify all 5 crates appear on crates.io
- [ ] Test: `cargo install haydn` works from a clean environment

## Post-release

- [ ] Submit esolang wiki page (copy from `docs/esolang-wiki.md`)
- [ ] Verify `cargo install haydn` link in wiki page works
