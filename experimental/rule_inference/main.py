import os
import openai
import argparse
import logging

from piranha_agent import PiranhaAgent
from logger_formatter import CustomFormatter
from rule_application import CodebaseRefactorer


logger = logging.getLogger("InferenceCli")
logger.setLevel(logging.DEBUG)

ch = logging.StreamHandler()
ch.setLevel(logging.DEBUG)
ch.setFormatter(CustomFormatter())
logger.addHandler(ch)


def main():
    logger.info("Starting Piranha Agent")
    arg_parser = argparse.ArgumentParser()
    arg_parser.add_argument(
        "-s",
        "--source-file",
        type=str,
        required=True,
        help="Path to the original source file containing the code before the transformation",
    )
    arg_parser.add_argument(
        "-t",
        "--target-file",
        type=str,
        required=True,
        help="Path to the target source file containing the code after the transformation",
    )
    arg_parser.add_argument(
        "-l",
        "--language",
        type=str,
        default="java",
        help="Language of the source and target code",
    )
    arg_parser.add_argument(
        "-k", "--openai-api-key", type=str, required=True, help="OpenAI API key"
    )

    arg_parser.add_argument(
        "-p",
        "--path-to-codebase",
        type=str,
        default="",
        help="Code base where the rule should be applied after a successful inference",
    )

    arg_parser.add_argument(
        "-c",
        "--path-to-piranha-config",
        type=str,
        default="./piranha-configs/",
        help="The directory where rule should be persisted",
    )

    args = arg_parser.parse_args()
    source_code = open(args.source_file, "r").read()
    target_code = open(args.target_file, "r").read()

    openai.api_key = args.openai_api_key
    agent = PiranhaAgent(source_code, target_code, language=args.language)

    rule_name, rule = agent.infer_rules()
    logger.info(f"Generated rule:\n{rule}")

    file_path = os.path.join(args.path_to_piranha_config, rule_name)
    logger.info(f"Writing rule to {file_path}")
    os.makedirs(args.path_to_piranha_config, exist_ok=True)
    with open(file_path, "w") as f:
        f.write(rule)

    if args.path_to_codebase:
        refactor = CodebaseRefactorer(
            args.language, args.path_to_codebase, args.path_to_piranha_config
        )
        refactor.refactor_codebase()


main()
