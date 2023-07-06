from setuptools import setup, find_packages

setup(
    name="MyPackageName",
    version="0.1",
    packages=find_packages(),
    install_requires=[
        "tree-sitter",
        "tree-sitter-languages",
        "attrs",
        "openai",
        "polyglot-piranha",
        "toml",
        "pytest",
        "flask",
        "flask-socketio",
        "comby",
    ],
)
