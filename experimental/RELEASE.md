# Releasing

1. Update the version in `setup.py` to the next version.
2. Update the `CHANGELOG.md` for the impending release.
3. `git commit -am "Prepare for release X.Y.Z"` (where X.Y.Z is the new version)
4. `git tag -a vX.Y.Z -m "Version X.Y.Z"` (where X.Y.Z is the new version)
5. Create a PR, request for review and merge it into `master` (or `main`, depending on your default branch) after it is approved.
6. `git push && git push --tags`
7. Build the package:
   1. `python setup.py sdist bdist_wheel`
8. Upload the package to PyPI:
   1. `twine upload dist/*`
9. Visit [polyglot-piranha-playground](https://pypi.org/project/polyglot-piranha-playground/)
