# Re:CraftLaunch: Release Procedure (RELEASE)

This document establishes the release pipeline, version tagging, optimized binary build, and packaging process for `Re:CraftLaunch`.

---

## 1. Pre-Release Checklist

Verify all items below before cutting a release build or tagging:

- [ ] All unit, integration, and doc tests pass (`cargo test --all-targets` & `cargo test --doc`)
- [ ] Static analysis yields zero warnings (`cargo clippy --all-targets -- -D warnings`)
- [ ] Code formatting conforms to style guidelines (`cargo fmt --check`)
- [ ] All `.rs` files conform to the 1,000-line ceiling (`fd -e rs --exec-batch wc -l`)
- [ ] SemVer version updated in `Cargo.toml`
- [ ] Version badges and status text synchronized in `README.md` and `README_JA.md`
- [ ] Release notes synchronized across `CHANGELOG.md`, `docs/ja/CHANGELOG.md`, and `docs/en/CHANGELOG.md`
- [ ] Manual test guide items verified on a Windows desktop (`docs/en/MANUAL_TEST_GUIDE.md`)

---

## 2. Optimized Release Build

Build the standalone single-binary executable for Windows:

```powershell
# 1. Build optimized release binary
cargo build --release

# 2. Inspect generated binary
ls target/release/Re-CraftLaunch.exe
```

---

## 3. Version Tagging & Git Push Pipeline

> [!IMPORTANT]
> Git commits and pushes must strictly occur upon explicit direction from the boss (per `RULES.md`).

```powershell
# 1. Create release commit
git add .
git commit -m "chore: release vX.Y.Z"

# 2. Create annotated git tag
git tag -a vX.Y.Z -m "vX.Y.Z: Release summary"

# 3. Push to remote repository
git push origin main
git push origin --tags
```

---

## 4. Distribution Packaging (Scoop / ZIP)

```powershell
# Create standalone release ZIP archive
Compress-Archive -Path target/release/Re-CraftLaunch.exe, LICENSE, README.md -DestinationPath target/release/Re-CraftLaunch-vX.Y.Z-windows-x64.zip
```
