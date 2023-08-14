# `scalatest` Migration Plugin 

## Usage: 

Clone the repository - `git clone https://github.com/uber/piranha.git`

Install the dependencies - `pip3 install -r plugins/scala_test/requirements.txt`

Run the tool - `python3 plugins/scala_test/main.py -h`

CLI: 
```
usage: main.py [-h] --path_to_codebase PATH_TO_CODEBASE

Migrates scala tests!!!

options:
  -h, --help            show this help message and exit
  --path_to_codebase PATH_TO_CODEBASE
                        Path to the codebase directory.
```

## Test
```
pytest plugins/scala_test
```
