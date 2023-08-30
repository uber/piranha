import argparse

from update_calendar_interval import update_CalendarInterval


def _parse_args():
    parser = argparse.ArgumentParser(
        description="Updates the codebase to use a new version of `spark3`"
    )
    parser.add_argument(
        "--path_to_codebase",
        required=True,
        help="Path to the codebase directory.",
    )
    parser.add_argument(
        "--new_version",
        default="3.3",
        help="Version of `Spark` to update to.",
    )
    args = parser.parse_args()
    return args


def main():
    args = _parse_args()
    if args.new_version == "3.3":
        upgrade_to_spark_3_3(args.path_to_codebase)


def upgrade_to_spark_3_3(path_to_codebase):
    update_CalendarInterval([path_to_codebase])


if __name__ == "__main__":
    main()
