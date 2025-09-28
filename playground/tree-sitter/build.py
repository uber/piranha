#!/usr/bin/env python3

from __future__ import annotations

import argparse
import json
import shutil
import subprocess
import tempfile
import urllib.parse
from pathlib import Path
from typing import Dict, List

_HERE = Path(__file__).parent

_LANGUAGES = ["swift", "kotlin", "java", "go", "python"]


def run_command(cmd: List[str], cwd: Path = None) -> subprocess.CompletedProcess[str]:
    """Run a shell command and return the result."""
    try:
        result = subprocess.run(
            cmd, cwd=cwd, capture_output=True, text=True, check=True
        )
        return result
    except subprocess.CalledProcessError as e:
        print(f"Command failed: {' '.join(cmd)}")
        print(f"Error: {e.stderr}")
        raise


def build_concrete_syntax_wasm(repo_root: Path) -> Path:
    """Build WASM files for concrete-syntax crate."""
    concrete_syntax_dir = repo_root / "crates" / "concrete-syntax"
    pkg_web_dir = concrete_syntax_dir / "pkg-web"
    
    print("Building concrete syntax WASM package...")
    
    # Check if WASM files need to be built (simple existence check)
    wasm_files = [
        pkg_web_dir / "concrete_syntax.js",
        pkg_web_dir / "concrete_syntax_bg.wasm"
    ]
    
    if all(f.exists() for f in wasm_files):
        print("✓ Concrete syntax WASM files already exist, skipping build...")
    else:
        print("Building WASM package for web target...")
        
        # Build the WASM target first
        run_command([
            "cargo", "build", 
            "--target", "wasm32-unknown-unknown",
            "--no-default-features", 
            "--features", "wasm"
        ], cwd=concrete_syntax_dir)
        
        # Generate WASM bindings
        run_command([
            "wasm-pack", "build",
            "--target", "web",
            "--out-dir", "pkg-web", 
            "--no-default-features",
            "--features", "wasm"
        ], cwd=concrete_syntax_dir)
        
        print("✅ WASM package built successfully")
    
    # Verify files exist
    for wasm_file in wasm_files:
        if not wasm_file.exists():
            raise FileNotFoundError(f"Required WASM file not found: {wasm_file}")
    
    return pkg_web_dir


def extract_tree_sitter_deps(repo_root: Path) -> list[tuple[str, str, dict[str, str]]]:
    """Extract tree-sitter dependencies from cargo metadata."""
    cmd = ["cargo", "metadata", "--format-version", "1"]
    result = run_command(cmd, cwd=repo_root)
    metadata = json.loads(result.stdout)

    deps = []

    for package in metadata["packages"]:
        if not package["name"].startswith("tree-sitter-"):
            continue

        lang_name = package["name"].replace("tree-sitter-", "")
        if lang_name in _LANGUAGES:
            deps.append((lang_name, package["name"], package))

    return deps


def parse_git_source(dep_info: Dict) -> tuple[str, str]:
    """Parse git source information from dependency."""
    source: str = dep_info["source"]
    # An example git string: "git+https://github.com/danieltrt/tree-sitter-go.git?rev=ea5ceb716012db8813a2c05fab23c3a020988724#ea5ceb716012db8813a2c05fab23c3a020988724"
    # So we first remove the "git+" prefix and remove the "#" part if it exists.
    source = source.removeprefix("git+").split("#")[0].strip()

    if "?" not in source:
        raise ValueError(f"Expecting ? in git source string: {source}")

    git_url, query_string = source.split("?", 1)
    params = urllib.parse.parse_qs(query_string)
    rev = (
        params.get("rev", [None])[0]
        or params.get("branch", [None])[0]
        or params.get("tag", [None])[0]
    )
    if not rev:
        raise ValueError(f"Missing rev/branch/tag information in git source: {source}")

    return git_url, rev


def clone_grammar(name: str, dep_info: Dict, temp_dir: Path) -> Path:
    """Clone a grammar repository to temporary directory."""
    source: str = dep_info["source"]
    # If it is a git source, parse it. Otherwise, it is a registry source, and we can assume it is
    # from tree-sitter official repo.
    if source.startswith("git+"):
        git_url, version = parse_git_source(dep_info)
    elif source.startswith("registry+"):
        repo_name = name.replace("tree-sitter-", "")
        git_url = f"https://github.com/tree-sitter/tree-sitter-{repo_name}"
        version = "v" + dep_info["version"]
    else:
        raise ValueError(f"Unsupported source type for {name}: {source}")

    clone_dir = temp_dir / name

    print(f"Cloning {name} from {git_url} and checking out {version}")
    run_command(["git", "clone", git_url, str(clone_dir)])
    run_command(["git", "checkout", version], cwd=clone_dir)

    return clone_dir


def build_wasm(grammar_dir: Path, name: str) -> Path:
    """Build WASM file for a grammar."""
    print(f"Building WASM for {name}")

    # Note that we have to use tree-sitter CLI 0.24 since the main tree-sitter and grammars
    # we use in Piranha are old and not compatible with the latest tree-sitter CLI.
    # TODO: remove this restriction once we upstream all our changes to tree-sitter grammars
    #  and upgrade to latest tree-sitter in Piranha.
    try:
        proc = run_command(["tree-sitter", "--version"])
        version = proc.stdout.strip().split()[1]
        if not version.startswith("0.24"):
            raise RuntimeError(f"tree-sitter CLI version {version} not supported")
    except (subprocess.CalledProcessError, FileNotFoundError, RuntimeError):
        raise RuntimeError(
            "tree-sitter CLI version 0.24.x is required. Install with: cargo install tree-sitter-cli --version 0.24.4"
        )

    print(f"Using tree-sitter CLI version: {proc.stdout.strip()}")

    run_command(["tree-sitter", "build", "--wasm"], cwd=grammar_dir)

    wasm_file = grammar_dir / f"{name}.wasm"
    if not wasm_file.exists():
        raise FileNotFoundError(f"WASM file not found for {name}")

    return wasm_file


def copy_wasm_to_assets(wasm_file: Path, lang_name: str, assets_dir: Path) -> Path:
    """Copy WASM file to assets directory."""
    assets_dir.mkdir(exist_ok=True)

    dest_file = assets_dir / f"tree-sitter-{lang_name}.wasm"

    print(f"Copying {wasm_file} to {dest_file}")
    shutil.copy2(wasm_file, dest_file)

    return dest_file


def instantiate_index_html(template_path: Path, output_path: Path):
    with template_path.open("r") as inp, output_path.open("w") as out:
        content = inp.read()
        languages = []
        for lang in _LANGUAGES:
            # Set python as the default language
            if lang == "python":
                languages.append(f'<option value="{lang}" selected>{lang.title()}</option>')
            else:
                languages.append(f'<option value="{lang}">{lang.title()}</option>')
        content = content.replace("{{ LANGUAGE_OPTIONS }}", "\n".join(languages))
        out.write(content)


def main():
    """Build WASM files for all supported tree-sitter dependencies."""

    """Main entry point with argument parsing."""
    parser = argparse.ArgumentParser(
        description="Build tree-sitter playground with WASM files"
    )
    parser.add_argument(
        "--dist-dir",
        "-d",
        type=Path,
        help="Directory to copy playground files and build WASM files to",
        default=Path().cwd() / "dist",
    )

    args = parser.parse_args()
    dist_dir = Path(args.dist_dir)

    if dist_dir.exists():
        print(f"Dist directory {dist_dir} already exists, clearing it...")
        shutil.rmtree(dist_dir)

    proc = run_command(["git", "rev-parse", "--show-toplevel"])
    repo_root = Path(proc.stdout.strip())
    print(f"Using repo root: {repo_root}")
    print()

    print("Instantiating index.html.template to dist directory...")
    dist_dir.mkdir(parents=True, exist_ok=True)
    instantiate_index_html(_HERE / "index.html.template", dist_dir / "index.html")
    
    # Copy concrete syntax integration files
    print("Copying concrete syntax integration files...")
    concrete_syntax_files = ["concrete-syntax.css", "concrete-syntax.js"]
    for file_name in concrete_syntax_files:
        src_file = _HERE / file_name
        if src_file.exists():
            dest_file = dist_dir / file_name
            print(f"Copying {src_file} to {dest_file}")
            shutil.copy2(src_file, dest_file)
        else:
            raise FileNotFoundError(f"Required concrete syntax file not found: {src_file}")
    
    # Build and copy concrete syntax WASM files
    print("\nBuilding concrete syntax WASM files...")
    concrete_syntax_pkg_dir = build_concrete_syntax_wasm(repo_root)
    
    assets_dir = dist_dir / "assets"
    assets_dir.mkdir(exist_ok=True)
    for wasm_file in ["concrete_syntax.js", "concrete_syntax_bg.wasm"]:
        src_file = concrete_syntax_pkg_dir / wasm_file
        dest_file = assets_dir / wasm_file
        print(f"Copying {src_file} to {dest_file}")
        shutil.copy2(src_file, dest_file)

    print("\nBuilding WASM files for all supported tree-sitter dependencies...")

    print("Extracting tree-sitter dependencies to build WASM grammars...")
    deps = extract_tree_sitter_deps(repo_root)

    if not deps:
        raise RuntimeError("No supported tree-sitter dependencies found")

    print(f"Found {len(deps)} supported tree-sitter dependencies:")
    for lang_name, pkg_name, _ in deps:
        print(f"  - {pkg_name} ({lang_name})")

    with tempfile.TemporaryDirectory() as temp_dir_str:
        temp_dir = Path(temp_dir_str)

        for lang_name, pkg_name, dep_info in deps:
            print(f"\n--- Processing {pkg_name} ---")

            grammar_dir = clone_grammar(pkg_name, dep_info, temp_dir)
            wasm_file = build_wasm(grammar_dir, pkg_name)
            copy_wasm_to_assets(wasm_file, lang_name, assets_dir)

            print(f"✓ Successfully built {pkg_name}")

        print("\n=== Build Complete ===")
        print(f"Successfully built {len(deps)} grammars: {_LANGUAGES}")
        print(f"Concrete syntax WASM integration: ✓")
        print(f"Output directory: {dist_dir}")


if __name__ == "__main__":
    main()
