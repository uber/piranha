import argparse
from update_imports import update_imports

def _parse_args():
    parser = argparse.ArgumentParser(description="Migrates scala tests!!!")
    parser.add_argument(
        "--path_to_codebase",
        required=True,
        help="Path to the codebase directory.",
    )
    
    args = parser.parse_args()
    return args

def main():
    args = _parse_args()
    update_imports(args.path_to_codebase, dry_run=True)

if __name__ == "__main__":
    main()
