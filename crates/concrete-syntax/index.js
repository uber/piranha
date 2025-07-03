const Parser = require('tree-sitter');
const Python = require('tree-sitter-python');
const { readFileSync } = require('fs');
const { getAllMatches, parsePattern } = require('./pkg/concrete_syntax.js');

/**
 * Wrapper class that implements the WasmSyntaxNode interface expected by Rust
 */
class TreeSitterNodeWrapper {
    constructor(node) {
        this.node = node;
    }

    get startByte() {
        return this.node.startIndex;
    }

    get endByte() {
        return this.node.endIndex;
    }

    get startRow() {
        return this.node.startPosition.row;
    }

    get startColumn() {
        return this.node.startPosition.column;
    }

    get endRow() {
        return this.node.endPosition.row;
    }

    get endColumn() {
        return this.node.endPosition.column;
    }

    get kind() {
        return this.node.type;
    }

    get childCount() {
        return this.node.childCount;
    }

    child(index) {
        const child = this.node.child(index);
        return child ? new TreeSitterNodeWrapper(child) : null;
    }

    children() {
        const children = [];
        for (let i = 0; i < this.node.childCount; i++) {
            const child = this.node.child(i);
            if (child) {
                children.push(new TreeSitterNodeWrapper(child));
            }
        }
        return children;
    }

    utf8Text(sourceCode) {
        return this.node.text;
    }

    clone() {
        return new TreeSitterNodeWrapper(this.node);
    }

    walk() {
        return new TreeSitterCursorWrapper(this.node);
    }
}

/**
 * Wrapper class that implements the WasmSyntaxCursor interface expected by Rust
 */
class TreeSitterCursorWrapper {
    constructor(node) {
        this.currentNode = node;
        this.nodeStack = [];
        this.indexStack = []; // Track current index at each level
        this.currentIndex = 0; // Current index in parent's children
    }

    node() {
        return new TreeSitterNodeWrapper(this.currentNode);
    }

    gotoFirstChild() {
        if (this.currentNode.childCount > 0) {
            this.nodeStack.push(this.currentNode);
            this.indexStack.push(this.currentIndex);
            this.currentNode = this.currentNode.child(0);
            this.currentIndex = 0;
            return true;
        }
        return false;
    }

    gotoNextSibling() {
        if (this.nodeStack.length === 0) return false;
        
        const parent = this.nodeStack[this.nodeStack.length - 1];
        
        if (this.currentIndex + 1 < parent.childCount) {
            this.currentIndex += 1;
            this.currentNode = parent.child(this.currentIndex);
            return true;
        }
        return false;
    }

    gotoParent() {
        if (this.nodeStack.length > 0) {
            this.currentNode = this.nodeStack.pop();
            this.currentIndex = this.indexStack.pop();
            return true;
        }
        return false;
    }

    clone() {
        const cloned = new TreeSitterCursorWrapper(this.currentNode);
        cloned.nodeStack = [...this.nodeStack];
        cloned.indexStack = [...this.indexStack];
        cloned.currentIndex = this.currentIndex;
        return cloned;
    }
}

/**
 * Simple demo of concrete syntax pattern matching on Python code
 */
async function main() {
    console.log('🐍 Python Concrete Syntax Pattern Matching Demo\n');

    // Initialize tree-sitter parser for Python
    const parser = new Parser();
    parser.setLanguage(Python);

    // Sample Python code to analyze
    const pythonCode = `
def greet(name):
    message = f"Hello, {name}!"
    print(message)
    return message

def calculate(x, y):
    result = x + y
    return result

# Some variables
age = 25
name = "Alice"
score = calculate(10, 15)
foo(c, d)
foo(1,4)
`;

    console.log('📄 Python Code:');
    console.log(pythonCode);
    console.log('─'.repeat(50));

    // Parse the Python code
    const tree = parser.parse(pythonCode);
    const wrappedRootNode = new TreeSitterNodeWrapper(tree.rootNode);
    const sourceBytes = new TextEncoder().encode(pythonCode);

    const pattern = 'foo(:[arg1], :[arg2]) |> :[arg1] in ["1"], :[arg2] in ["4"]';
    console.log(`   Pattern: "${pattern}"`);
    
    try {
        const matches = getAllMatches(pattern, wrappedRootNode, sourceBytes, true, null);
        
        if (matches.length > 0) {
            console.log(`   ✅ Found ${matches.length} match(es):`);
            matches.forEach((match, index) => {
                const line = match.range.start_point.row + 1;
                const matchText = match.matched_string;
                console.log(`      ${index + 1}. Line ${line}: "${matchText}"`);
                // Show captures if any
                const captureKeys = Object.keys(match.matches).filter(key => key !== '*');
                if (captureKeys.length > 0) {
                    captureKeys.forEach(key => {
                        const captureText = match.matches[key];
                        console.log(`         ${key}: "${captureText}"`);
                    });
                }
            });
        } else {
            console.log(`   ❌ No matches found`);
        }
    } catch (error) {
        console.log(`   ⚠️  Error: ${error.message}`);
    }
    

    console.log('\n🎉 Pattern matching complete!');
}

// Run the demo
main().catch(console.error); 