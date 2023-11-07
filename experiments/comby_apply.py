import subprocess
import re
import time
import argparse
from typing import List, Any
import comby
import tempfile


def extract_filenames_from_diff(diff: str) -> List[Any]:
    # Remove ANSI escape sequences
    ansi_escape = re.compile(r'\x1B(?:[@-Z\\-_]|\[[0-?]*[ -/]*[@-~])')
    cleaned_diff = ansi_escape.sub('', diff)

    # Extract filenames
    pattern = r"[+-]{6}\s(.*?)\n"
    matches = re.findall(pattern, cleaned_diff)
    return list(set(matches))

def run_comby(config_file: str, extension: str, target_directory: str, in_place=False) -> subprocess.CompletedProcess:
    cmd = 'comby'
    args = [
        cmd,
        '-config', config_file,
        '-f', f'.{extension}',
        '-directory', target_directory
    ]
    if in_place:
        args.append('-in-place')

    # Run the command
    print(f"Running {args}")
    p = subprocess.run(args, capture_output=True, text=True)
    return p


def instantiate_config(config_path: str, match: str, replace: str) -> str:
    with open(config_path, 'r') as file:
        config_data = file.read()

    # Replace match and replace placeholders with the provided arguments
    config_data = config_data.replace("{{match}}", match)
    config_data = config_data.replace("{{replace}}", replace)

    # Write the modified configuration to a temporary file
    fd, temp_path = tempfile.mkstemp()
    with open(fd, 'w') as temp_file:
        temp_file.write(config_data)

    return temp_path

if __name__ == '__main__':
    parser = argparse.ArgumentParser(description='Run comby on a specific directory.')
    parser.add_argument('-c', '--config', dest='config_file', default="./comby_rules.toml", help='The comby configuration file (TOML).')
    parser.add_argument('-e', '--extension', required=True, help='File extension to target with comby.')
    parser.add_argument('-d', '--directory', dest='target_directory', required=True, help='Directory in which to run comby.')
    parser.add_argument('-i', '--in-place', action='store_true', help='Apply changes in-place.')

    # The arguments of the flag
    parser.add_argument('-m', '--match', required=True, help='The match pattern for comby.')
    parser.add_argument('-r', '--replace', required=True, help='The replace pattern for comby.')

    args = parser.parse_args()
    temp_config_path = instantiate_config(args.config_file, args.match, args.replace)

    # Preview changes
    process = run_comby(temp_config_path, args.extension, args.target_directory)
    affected_files = extract_filenames_from_diff(process.stdout)
    comby_runs = 0

    while affected_files:
        print("The following files will be modified:")
        for file in affected_files:
            print(file)

        if args.in_place:
            start = time.time()
            run_comby(temp_config_path, args.extension, args.target_directory, in_place=True)
            comby_runs += time.time() - start
            print(f'Time taken: {time.time() - start} seconds')

        process = run_comby(temp_config_path, args.extension, args.target_directory)
        affected_files = extract_filenames_from_diff(process.stdout)

    print(f"Comby total runs: {comby_runs}")
    os.remove(temp_config_path)
