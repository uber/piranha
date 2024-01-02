Releasing
=========

 1. Create a new branch - `git checkout -b "release-X.Y.Z"` (where X.Y.Z is the new version)
 2. Change the version in `Cargo.toml` to the next version.
 3. Update the `CHANGELOG.md` for the impending release.
 4. Update the `README.md` with the new version.
 5. `git commit -am "[PolyglotPiranha] Prepare for release X.Y.Z."` (where X.Y.Z is the new version)
 6. `git tag -a vX.Y.Z-polyglot -m "PolyglotPiranha Version X.Y.Z"` (where X.Y.Z is the new version)
 7. `git push --set-upstream release-X.Y.Z && git push --tags`
 8. Create a PR, request for review and merge it into `master` after it is approved.
 9. From your terminal run : `gh workflow run "Release Polyglot Piranha" --ref master` or do this from the Github UI
 10. Wait till this action is completed.
 11. Manually release MacOS M1 wheel (Currently GitHub Actions do not provide M1 VM)
    1. Clone the piranha repo and pull the latest `master` branch 
    2. `cargo build --no-default-features`
    3. `maturin build --release`
    4. `twine upload --skip-existing target/wheels/*`
 13. Visit [polyglot-piranha](https://pypi.org/project/polyglot-piranha/)
