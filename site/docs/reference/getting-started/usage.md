---
id: usage
title: Usage
sidebar_label: Usage
---

Polyglot Piranha can be used as a Python library or as a command-line tool.

### Python API

Here's an example of how to use the Python API:

```python
from polyglot_piranha import execute_piranha, PiranhaArguments

piranha_arguments = PiranhaArguments(
    path_to_codebase = "...",
    path_to_configurations = "...",
    language= "java",
    substitutions = {},
    dry_run = False, 
    cleanup_comments = True
)
piranha_summary = execute_piranha(piranha_arguments)
```

### Command-line Interface

Here's an example of how to use the command-line interface:

```bash
polyglot_piranha [OPTIONS] --path-to-codebase <PATH_TO_CODEBASE> --path-to-configurations <PATH_TO_CONFIGURATIONS> -l <LANGUAGE>
```

For more detailed usage instructions, please refer to the [official documentation](https://github.com/uber/piranha/blob/master/README.md).
