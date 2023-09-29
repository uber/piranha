import subprocess
import re
import time
from typing import List, Any
import comby


def extract_filenames_from_diff(diff: str) -> list[Any]:
    # Remove ANSI escape sequences
    ansi_escape = re.compile(r'\x1B(?:[@-Z\\-_]|\[[0-?]*[ -/]*[@-~])')
    cleaned_diff = ansi_escape.sub('', diff)

    # Extract filenames
    pattern = r"[+-]{6}\s(.*?)\n"
    matches = re.findall(pattern, cleaned_diff)
    return list(set(matches))

def run_comby(config_file: str, extension: str, in_place=False) -> subprocess.CompletedProcess:
    cmd = 'comby'
    args = [
        cmd,
        '-config', config_file,
        '-f', f'.{extension}'
    ]
    if in_place:
        args.append('-in-place')

    # Run the command
    p = subprocess.run(args, capture_output=True, text=True)
    return p


if __name__ == '__main__':
    # Preview changes
    process = run_comby('comby_rules.toml', "java")
    affected_files = extract_filenames_from_diff(process.stdout)

    while affected_files:
        print("The following files will be modified:")
        for file in affected_files:
            print(file)
        # Apply the changes with -in-place
        start = time.time()
        run_comby('comby_rules.toml', "java", in_place=True)
        print(time.time() - start)
        process = run_comby('comby_rules.toml', "java")
        affected_files = extract_filenames_from_diff(process.stdout)



