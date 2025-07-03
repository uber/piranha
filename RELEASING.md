Releasing
=========

 1. `git checkout master && git pull` to get latest master.
 2. Change the version in `Cargo.toml` to the next version.
 3. `git commit -am "Release X.Y.Z."` (where X.Y.Z is the new version)
 4. `git tag -a vX.Y.Z -m "Piranha X.Y.Z"` (where X.Y.Z is the new version)
 5. `git push && git push --tags`
 6. The release workflow will automatically be triggered to build wheels and source distributions, automatically upload to PyPI, and then create a GitHub release.
 7. Visit [polyglot-piranha](https://pypi.org/project/polyglot-piranha/) in PyPI and [GitHub Release](https://github.com/uber/piranha/releases) to see if releases are there.
