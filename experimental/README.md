# PiranhaAgent 

This code uses OpenAI's GPT-4 model to generate piranha rules in TOML format.

## Requirements

- Python 3.7+
- OpenAI Python library
- Tree-sitter Python library
- Tree-sitter-languages Python library

## Usage

1. Set the `OPENAI_API_KEY` environment variable with your OpenAI API key.
2. Run the script with the following command:

```
python main.py -s <source_file> -t <target_file> -l <language>
```

- `<source_file>`: Path to the original source code file.
- `<target_file>`: Path to the refactored source code file.
- `<language>`: (Optional) Programming language of the source code files. Default is "java".

## How it works

1. The script reads the original and refactored source code files and generates their Tree-sitter representations.
2. It then formats a prompt using the task explanation, examples, and input template.
3. The prompt is sent to the GPT-4 model, which generates a TOML file containing the refactoring rules.
4. The generated TOML file is printed to the console.

## Files

- `main.py`: Main script to run the PiranhaAgent.
- `explanation.txt`: Task explanation for the GPT-4 model.
- `input_template.txt`: Input template for formatting the prompt.

We also feed the model the cleanup rules as examples under `src/cleanup`