from os.path import join, dirname
from polyglot_piranha import execute_piranha, PiranhaArguments
import logging
import argparse

feature_flag_dir = dirname(__file__)

def clean_flag(flag_name: str, directory: str, piranha_config: str):
    logging.info("Running the stale feature flag cleanup demo for Java")

    directory_path = join(feature_flag_dir, directory)
    configuration_path = join(feature_flag_dir, piranha_config)
    print(configuration_path)

    args = PiranhaArguments(
        "java",
        substitutions={
            "arg": flag_name,
            "val": "true"
        },
        path_to_codebase=directory_path,
        path_to_configurations=configuration_path,
    )
    output_summary_java = execute_piranha(args)
    print(output_summary_java)

def log():
    FORMAT = "%(levelname)s %(name)s %(asctime)-15s %(filename)s:%(lineno)d %(message)s"
    logging.basicConfig(format=FORMAT)
    logging.getLogger().setLevel(logging.DEBUG)

def main():
    parser = argparse.ArgumentParser(description='Stale feature flag cleanup demo for Java.')
    parser.add_argument('-f', '--flag', dest='flag_name', type=str, required=True, help='Name of the feature flag to be cleaned up')
    parser.add_argument('-d', '--dir', dest='directory', type=str, required=True, help='Path to the directory containing the repos')
    parser.add_argument('-c', '--config', dest='piranha_config', type=str, required=True, help='Path to the Piranha configuration file')

    args = parser.parse_args()

    clean_flag(args.flag_name, args.directory, args.piranha_config)

if __name__ == "__main__":
    main()
