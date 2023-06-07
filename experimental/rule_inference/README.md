# PiranhaAgent

PiranhaAgent uses OpenAI's GPT-4 model to infer piranha rules from code examples. 
It generates these rules in TOML format, which can be applied to refactor other parts of the codebase.

## Install

To get started with PiranhaAgent, follow these instructions:

1. Clone this repository:
```
git clone https://github.com/uber/piranha.git
```

2. Create a Python virtual environment and activate it:

```
python3 -m venv .env
source .env/bin/activate
```

3. Navigate into the directory:
```
cd experimental/rule_inference
```
4. Install the necessary requirements:
```
pip install -r requirements.txt
```




## Usage

To make use of PiranhaAgent, please follow the steps below:

1. Execute the main.py script with the appropriate command-line arguments. The required format for the command is:

```
python piranha_agent.py -s <source_file> -t <target_file> -l <language> -k <openai_api_key>
```

Here,

- `<source_file>`: The path to the original source code file.
- `<target_file>`: The path to the refactored source code file.
- `<language>`: (Optional) This specifies the programming language of the source code files. The default language is `java`.
- `<openai_api_key>`: OpenAI Secret API Key

## Demo

To run a demo of PiranhaAgent, execute the following command:

```
python3 piranha_agent.py --source-file demo/before.java --target-file demo/after.java -k <YOUR_KEY>
```

After running this demo, the agent will print a TOML file containing a piranha rules that transforms
the `before.java` file into the `after.java` file.

## How it works

1. The script reads both the original and refactored source code files, generating the Tree-sitter representation for each.
2. A prompt is formulated using the task explanation, examples, and input template.
3. This prompt is fed to the GPT-4 model, which generates a TOML file encapsulating the refactoring rules.
4. The generated TOML file is output to the console.

## Files

- `main.py`: The main script to run PiranhaAgent.
- `base_prompt.py`: Contains the prompt template, and helper functions build specific prompts.

We also feed the model the cleanup rules as examples, which can be found in the [src/cleanup_rules](../../src/cleanup_rules) directory.
