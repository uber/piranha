# PiranhaAgent

PiranhaAgent uses a static inference algorithm and OpenAI's GPT-4 model to generate human-like piranha rules from code examples.
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

## Usage (Playground UI)

To run the playground

1. Execute the local.py script. You have to set an environment variable `OPENAI_API_KEY` with your OpenAI API key.

```
export OPENAI_API_KEY=<YOUR_KEY>
python -m local
```

To define your transformation rules, you will need to provide pairs of code snippets: 'before' and 'after' transformation. Each piece of code that needs transformation should be marked by unique identifiers surrounded by comments (like `// 1`, `// 2`, etc.). These identifiers serve two purposes:

1. They denote which part of the code should be transformed.
2. They define the transformation sequence, or cascading, i.e., which transformations should follow which.

### Example

Consider the following code snippet:

#### Code before refactoring:

```java
class SomeClass {
  // 1 -> 2

  // 1
  void someMethod(String arg) {

  }
  // end

  void otherMethod() {
    String x;
    // 2
   	someMethod(x);
    // end
  }
}
```

#### Code after refactoring:

```java
class SomeClass {
  // 1 -> 2

  // 1
  public String someMethod() {

  }
  // end

  void otherMethod() {
    String x;
    // 2
    x = someMethod();
    // end
  }
}
```

In this example, there are two transformation points, marked by the identifiers `// 1` and `// 2`.

- `// 1` shows the transformation from `void someMethod(String arg)` to `public String someMethod()`.
- `// 2` shows the transformation from `someMethod(x)` to `x = someMethod()`.

The arrow notation `// 1 -> 2` indicates the transformation cascade, i.e., transformation `// 2` should be applied after transformation `// 1`.

Each transformation snippet begins with the identifier (like `// 1`) and ends with a `// end` comment.

This way of representing transformations helps to create clear, concise, and human-friendly refactoring rules, making it easier to manage and understand your transformations.

Make sure to follow these conventions when inputting your code for refactoring. Happy coding!

**Note: The code before and after must be syntactically correct, and it should parse. Moreover, after applying the rules to code before, the refactored code should match the code after. (spaces are ignored).**
