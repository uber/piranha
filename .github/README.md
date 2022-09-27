# Piranha GitHub Workflows

## Pull Requests

For pull requests, workflows and jobs will be triggered based on the changes made in a directory.

We define a [GitHub reusable workflow](https://docs.github.com/en/actions/using-workflows/reusing-workflows) in [detect_changes.yml](workflows/detect_changes.yml) that uses the [paths-filter](https://github.com/dorny/paths-filter) action to detect changes made in each directory declared in [filters.yaml](filters.yaml).

Regular CI workflows conditionally execute their jobs based on the output of the reusable workflow's job.
If no change is detected, the regular workflow will be skipped and its `skip_` counterpart will run. The `skip_` workflows have the will always pass.
They have the same `name` key and required job names as the regular workflow.
This is a [workaround suggested by GitHub](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/defining-the-mergeability-of-pull-requests/troubleshooting-required-status-checks#example) to handle required checks that are skipped.

## Push to `master` and accepted PRs

All workflows and jobs will be executed when there is a push to `master`. This scenario includes accepted PRs.
