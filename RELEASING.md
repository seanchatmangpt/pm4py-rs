# PM4Py Rust Release Process

This document describes the complete release process for PM4Py Rust.

---

## Release Checklist

### Phase 1: Preparation (1-2 weeks before)

- [ ] Update version in `Cargo.toml`
  ```toml
  version = "X.Y.Z"
  ```

- [ ] Update version in `src/version.rs`
  ```rust
  pub const VERSION: &str = "X.Y.Z";
  pub const VERSION_MAJOR: u32 = X;
  pub const VERSION_MINOR: u32 = Y;
  pub const VERSION_PATCH: u32 = Z;
  ```

- [ ] Verify all tests pass locally
  ```bash
  cargo test --all
  cargo test --all --release
  ```

- [ ] Run clippy with strict warnings
  ```bash
  cargo clippy --all --all-targets --all-features -- -D warnings
  ```

- [ ] Check documentation builds
  ```bash
  cargo doc --no-deps --document-private-items
  ```

- [ ] Verify examples compile and run
  ```bash
  cargo build --examples
  cargo run --example simple_discovery
  cargo run --example conformance_checking
  cargo run --example visualization
  ```

### Phase 2: Code Quality (1 week before)

- [ ] Run full test suite with coverage
  ```bash
  cargo test --all --verbose
  ```

- [ ] Ensure 80%+ code coverage
  - Review coverage report
  - Add tests for uncovered branches

- [ ] Run benchmarks and compare
  ```bash
  cargo bench --no-run
  cargo bench --bench discovery
  cargo bench --bench conformance
  cargo bench --bench io
  ```

- [ ] Verify no clippy warnings
  ```bash
  cargo clippy --all --all-targets --all-features 2>&1 | grep -i warning
  # Should return empty
  ```

- [ ] Security audit
  ```bash
  cargo audit
  # Should show 0 vulnerabilities
  ```

- [ ] Check dependencies for updates
  ```bash
  cargo outdated
  # Update if security issues
  ```

- [ ] Format code
  ```bash
  cargo fmt --all
  ```

### Phase 3: Documentation (1 week before)

- [ ] Update `CHANGELOG.md`
  - Add new version section at top
  - List all new features (Added)
  - List all changes (Changed)
  - List all bug fixes (Fixed)
  - List any deprecations (Deprecated)
  - List removals (Removed)
  - List security fixes (Security)

- [ ] Create `RELEASE_X.Y.Z.md`
  - Highlight major features
  - Document new APIs with examples
  - List performance improvements
  - Note breaking changes (if any)
  - Include migration guide
  - Acknowledge contributors

- [ ] Update `README.md`
  - Update version in quick start
  - Add any new major features
  - Update feature list if changed

- [ ] Update `docs/UPGRADE.md` (if needed)
  - Document API changes
  - Provide migration examples
  - Note any deprecations

- [ ] Verify all doc examples compile
  ```bash
  cargo test --doc
  ```

### Phase 4: Git & Tags (Release day)

- [ ] Create release branch (if major version)
  ```bash
  git checkout -b release/v0.3.0
  ```

- [ ] Commit version changes
  ```bash
  git add -A
  git commit -m "release: version 0.3.0"
  ```

- [ ] Create annotated git tag
  ```bash
  git tag -a v0.3.0 -m "Release version 0.3.0"
  ```

- [ ] Push changes
  ```bash
  git push origin main
  git push origin v0.3.0
  ```

### Phase 5: Publishing to crates.io

- [ ] Test publish (dry-run)
  ```bash
  cargo publish --dry-run
  ```

- [ ] Publish to crates.io
  ```bash
  cargo publish
  ```

- [ ] Verify on crates.io
  - Wait 1-2 minutes for crate to appear
  - Check https://crates.io/crates/pm4py

### Phase 6: GitHub Release

- [ ] Go to GitHub Releases page
- [ ] Create new release from tag
- [ ] Add description from RELEASE_X.Y.Z.md
- [ ] Mark as latest release

### Phase 7: Announcements

- [ ] Post on GitHub Discussions
- [ ] Update documentation
- [ ] Notify users if applicable

### Phase 8: Post-Release

- [ ] Create next development version
- [ ] Monitor for issues
- [ ] Plan next release

---

## Hotfix Releases

For critical bug fixes:

```bash
git checkout -b hotfix/v0.3.1 v0.3.0
# Make fix and commit
git tag -a v0.3.1 -m "Hotfix: description"
cargo publish
```

---

## Version Numbering

Follow Semantic Versioning (https://semver.org):

- **MAJOR** (0.X.0): Breaking changes, major new features
- **MINOR** (X.Y.0): New features, no breaking changes
- **PATCH** (X.Y.Z): Bug fixes only

---

## Contact & Questions

For questions about the release process:
- **Author**: Sean Chatman (info@chatmangpt.com)
- **Repository**: https://github.com/seanchatmangpt/pm4py-rust
- **Issues**: https://github.com/seanchatmangpt/pm4py-rust/issues

---

**Happy releasing! 🚀**
