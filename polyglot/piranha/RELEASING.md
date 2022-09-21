Releasing
=========

 1. Change the version in `Cargo.toml` to the next version.
 2. Update the `CHANGELOG.md` for the impending release.
 3. Update the `README.md` with the new version.
 4. `git commit -am "[PolyglotPiranha] Prepare for release X.Y.Z."` (where X.Y.Z is the new version)
 5. `git tag -a vX.Y.Z-polyglot -m "PolyglotPiranha Version X.Y.Z"` (where X.Y.Z is the new version)
 6. Create a PR, request for review and merge it into `master` after it is approved.
 7. From your terminal run : `gh workflow run "Release Polyglot Piranha" --ref master`
 8. Wait till this action is completed.
 9. Manually release MacOS M1 wheel (Currently GitHub Actions do not provide M1 VM)
    1. Pull the latest `master` branch 
    2. `cd polyglot/piranha` 
    3. `maturin build --release` 
    4. `twine upload --skip-existing target/wheels/*`
 10. `git push && git push --tags`
 11. Visit [polyglot-piranha](https://pypi.org/project/polyglot-piranha/)
