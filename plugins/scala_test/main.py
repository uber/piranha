import argparse
from update_imports import update_imports

def _parse_args():
    parser = argparse.ArgumentParser(description="Updates the codebase to use a new version of `scalatest_2.12`")
    parser.add_argument(
        "--path_to_codebase",
        required=True,
        help="Path to the codebase directory.",
    )
    parser.add_argument(
        "--new_version",
        required=True,
        default="3.2.2",
        help="Version of `scalatest` to update to.",
    )
    args = parser.parse_args()
    return args

def main():
    args = _parse_args()
    update_imports(args.path_to_codebase, args.new_version, dry_run=True)

if __name__ == "__main__":
    main()
