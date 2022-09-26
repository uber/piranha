# Piranha GitHub Workflows

## Pull Requests

For pull requests, workflows and jobs will be triggered based on the changes made in a directory.

We define a [GitHub reusable workflow](https://docs.github.com/en/actions/using-workflows/reusing-workflows) in [detect_changes.yml](workflows/detect_changes.yml) that uses the [paths-filter](https://github.com/dorny/paths-filter) action to detect changes made in each directory declared in [filters.yaml](filters.yaml).

Regular CI workflows conditionally execute their jobs based on the output of the reusable workflow's job.

## Push to `master` and accepted PRs

All workflows and jobs will be executed when there is a push to `master`. This scenario includes accepted PRs.
