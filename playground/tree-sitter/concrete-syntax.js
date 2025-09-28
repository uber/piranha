/**
 * Concrete Syntax Integration for Tree-sitter Playground
 */

/**
 * Conditional logging based on the logging checkbox
 */
function debugLog(...args) {
    const loggingCheckbox = document.getElementById('logging-checkbox');
    if (loggingCheckbox && loggingCheckbox.checked) {
        console.log(...args);
    }
}

function debugError(...args) {
    const loggingCheckbox = document.getElementById('logging-checkbox');
    if (loggingCheckbox && loggingCheckbox.checked) {
        console.error(...args);
    }
}

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

// Helper function to get the concrete syntax editor
function getConcreteSyntaxEditor() {
    const editorElement = document.getElementById('concrete-syntax-editor');
    if (editorElement && editorElement.CodeMirror) {
        return editorElement.CodeMirror;
    }
    return null;
}

// Initialize UI functionality
function initConcreteSyntaxUI() {
    const queryCheckbox = document.getElementById('query-checkbox');
    const queryTypeSelector = document.querySelector('.query-type-selector');
    const queryTypeRadios = document.querySelectorAll('input[name="query-type"]');
    const queryContainer = document.getElementById('query-container');
    const concreteSyntaxContainer = document.getElementById('concrete-syntax-container');
    const statusText = document.getElementById('concrete-syntax-status');
    const languageSelect = document.getElementById('language-select');

    // Initialize CodeMirror for concrete syntax
    const concreteSyntaxEditor = window.CodeMirror(document.getElementById('concrete-syntax-editor'), {
        lineNumbers: true,
        lineWrapping: true,
        theme: 'default',
        placeholder: 'Enter concrete syntax pattern...',
        mode: 'text',
        tabSize: 2,
        indentWithTabs: false,
        extraKeys: {
            'Ctrl-Enter': function(cm) {
                const pattern = cm.getValue().trim();
                if (pattern) {
                    executeConcreteSyntaxPattern(pattern);
                }
            },
            'Cmd-Enter': function(cm) {
                const pattern = cm.getValue().trim();
                if (pattern) {
                    executeConcreteSyntaxPattern(pattern);
                }
            }
        }
    });

    // Show/hide query sections based on query checkbox and type
    queryCheckbox.addEventListener('change', function() {
        if (this.checked) {
            queryTypeSelector.style.display = 'flex';
            updateQueryContainerVisibility();
        } else {
            queryTypeSelector.style.display = 'none';
            queryContainer.style.visibility = 'hidden';
            queryContainer.style.position = 'absolute';
            concreteSyntaxContainer.style.visibility = 'hidden';
            concreteSyntaxContainer.style.position = 'absolute';
        }
    });

    // Initialize visibility on page load
    if (queryCheckbox.checked) {
        queryTypeSelector.style.display = 'flex';
        updateQueryContainerVisibility();
    }

    // Handle query type changes
    queryTypeRadios.forEach(radio => {
        radio.addEventListener('change', function() {
            updateQueryContainerVisibility();
        });
    });

    function updateQueryContainerVisibility() {
        const queryType = document.querySelector('input[name="query-type"]:checked').value;
        
        if (queryType === 'tree-sitter') {
            queryContainer.style.visibility = 'visible';
            queryContainer.style.position = 'relative';
            concreteSyntaxContainer.style.visibility = 'hidden';
            concreteSyntaxContainer.style.position = 'absolute';
        } else {
            queryContainer.style.visibility = 'hidden';
            queryContainer.style.position = 'absolute';
            concreteSyntaxContainer.style.visibility = 'visible';
            concreteSyntaxContainer.style.position = 'relative';
        }
    }

    // Auto-execute on input changes
    let debounceTimer;
    concreteSyntaxEditor.on('change', function(cm) {
        const pattern = cm.getValue().trim();
        clearTimeout(debounceTimer);
        debounceTimer = setTimeout(() => {
            if (pattern) {
                executeConcreteSyntaxPattern(pattern);
            } else {
                clearHighlights();
            }
        }, 300); // 300ms debounce
    });

    // Initialize UI
    if (queryCheckbox.checked) {
        queryTypeSelector.style.display = 'flex';
        updateQueryContainerVisibility();
    }
}

// Execute concrete syntax pattern with fresh parsing and editor highlighting
function executeConcreteSyntaxPattern(pattern) {
    const statusText = document.getElementById('concrete-syntax-status');
    
    statusText.textContent = 'Searching...';

    if (!window.ConcreteSyntax) {
        statusText.textContent = 'WASM not ready';
        return;
    }

    try {
        // Always get fresh source code
        let sourceCode = getFreshSourceCode();
        
        if (!sourceCode || !sourceCode.trim()) {
            statusText.textContent = 'No code';
            clearHighlights();
            return;
        }

        // Always parse fresh tree
        const languageSelect = document.getElementById('language-select');
        const currentLanguage = languageSelect ? languageSelect.value : 'python';
        
        parseFreshTree(sourceCode, currentLanguage).then(tree => {
            if (!tree || !tree.rootNode) {
                statusText.textContent = 'Parse failed';
                return;
            }

            // Execute concrete syntax matching
            const matches = executeConcreteSyntaxQuery(pattern, tree.rootNode, sourceCode);
            
            // Highlight matches in editor
            highlightMatchesInEditor(matches);
            
            statusText.textContent = matches.length > 0 ? `${matches.length} match${matches.length !== 1 ? 'es' : ''}` : 'No matches';
            
        }).catch(err => {
            console.error('Parse error:', err);
            statusText.textContent = 'Parse error';
        });
        
    } catch (error) {
        console.error('Error executing concrete syntax pattern:', error);
        statusText.textContent = 'Error';
    }
}

// Get current source code from playground
function getFreshSourceCode() {
    if (window.playground && window.playground.codeEditor) {
        return window.playground.codeEditor.getValue();
    }
    
    // Fallback to CodeMirror search
    const codeMirrorElements = document.querySelectorAll('.CodeMirror');
    for (let cm of codeMirrorElements) {
        if (cm.CodeMirror && cm.CodeMirror.getValue) {
            const code = cm.CodeMirror.getValue();
            if (code && code.trim()) {
                return code;
            }
        }
    }
    
    // Last fallback to textarea
    const codeInput = document.getElementById('code-input');
    return codeInput?.value || '';
}

// Parse fresh tree every time
async function parseFreshTree(sourceCode, language) {
    if (!window.TreeSitter || !window.TreeSitter.Language) {
        throw new Error('TreeSitter not available');
    }

    const languageWasm = `./assets/tree-sitter-${language}.wasm`;
    const lang = await window.TreeSitter.Language.load(languageWasm);
    const parser = new window.TreeSitter.Parser();
    parser.setLanguage(lang);
    return parser.parse(sourceCode);
}

// Direct CodeMirror highlighting
function highlightMatchesInEditor(matches) {
    debugLog('highlightMatchesInEditor called with:', matches);
    
    // Clear previous highlights first
    clearHighlights();
    
    if (!matches || matches.length === 0) return;

    // Find CodeMirror instance directly
    const codeMirrorElements = document.querySelectorAll('.CodeMirror');
    let codeEditor = null;
    
    for (let cmElement of codeMirrorElements) {
        if (cmElement.CodeMirror) {
            codeEditor = cmElement.CodeMirror;
            break;
        }
    }
    
    if (!codeEditor) {
        debugLog('No CodeMirror instance found');
        return;
    }

    debugLog('Found CodeMirror, highlighting matches');

    // Use tree-sitter colors
    const colors = [
        "#0550ae", "#ab5000", "#116329", "#844708", "#6639ba", 
        "#7d4e00", "#0969da", "#1a7f37", "#cf222e", "#8250df"
    ];

    matches.forEach((match, index) => {
        debugLog('Processing match', index, ':', match);
        // Handle both snake_case and camelCase field names
        const range = match.range;
        const startPoint = range.startPoint || range.start_point;
        const endPoint = range.endPoint || range.end_point;
        
        if (range && startPoint && endPoint) {
            const from = {
                line: startPoint.row,
                ch: startPoint.column
            };
            const to = {
                line: endPoint.row, 
                ch: endPoint.column
            };

            // Highlight the full match with primary color
            codeEditor.markText(from, to, {
                css: `background-color: ${colors[0]}20; border: 1px solid ${colors[0]}60;`,
                title: `Concrete Syntax Match ${index + 1}`,
                className: 'concrete-syntax-match'
            });

            // Highlight captured variables with different colors
            const captures = match.matches;
            if (captures && typeof captures === 'object') {
                let colorIndex = 1;
                debugLog('Match captures:', captures);
                
                // Handle both Map and plain object
                const captureEntries = captures instanceof Map ? 
                    Array.from(captures.entries()) : 
                    Object.entries(captures);
                
                captureEntries.forEach(([key, captureData]) => {
                    debugLog('Capture:', key, captureData);
                    if (key !== '*' && captureData) {
                        const captureColor = colors[colorIndex % colors.length];
                        
                        // Check if captureData has range information (CapturedNode structure)
                        const captureRange = captureData.range;
                        const captureStartPoint = captureRange?.startPoint || captureRange?.start_point;
                        const captureEndPoint = captureRange?.endPoint || captureRange?.end_point;
                        
                        if (captureRange && captureStartPoint && captureEndPoint) {
                            // Use exact range from capture data
                            const captureFrom = {
                                line: captureStartPoint.row,
                                ch: captureStartPoint.column
                            };
                            const captureTo = {
                                line: captureEndPoint.row,
                                ch: captureEndPoint.column
                            };
                            
                            debugLog(`Highlighting capture ${key} from (${captureFrom.line},${captureFrom.ch}) to (${captureTo.line},${captureTo.ch})`);
                            
                            codeEditor.markText(captureFrom, captureTo, {
                                css: `background-color: ${captureColor}30; border: 1px solid ${captureColor}; border-radius: 2px;`,
                                title: `${key}: ${captureData.text || captureData}`,
                                className: 'concrete-syntax-capture'
                            });
                        } else if (typeof captureData === 'string') {
                            // Fallback: find the capture text within the match
                            const matchText = match.matchedString || match.matched_string || '';
                            const captureStart = matchText.indexOf(captureData);
                            
                            if (captureStart >= 0) {
                                const captureFrom = {
                                    line: from.line,
                                    ch: from.ch + captureStart
                                };
                                const captureTo = {
                                    line: from.line, 
                                    ch: from.ch + captureStart + captureData.length
                                };
                                
                                debugLog(`Highlighting string capture ${key} from (${captureFrom.line},${captureFrom.ch}) to (${captureTo.line},${captureTo.ch})`);
                                
                                codeEditor.markText(captureFrom, captureTo, {
                                    css: `background-color: ${captureColor}30; border: 1px solid ${captureColor}; border-radius: 2px;`,
                                    title: `${key}: ${captureData}`,
                                    className: 'concrete-syntax-capture'
                                });
                            }
                        }
                        
                        colorIndex++;
                    }
                });
            }
        }
    });
}

// Clear highlights directly from CodeMirror
function clearHighlights() {
    const codeMirrorElements = document.querySelectorAll('.CodeMirror');
    for (let cmElement of codeMirrorElements) {
        if (cmElement.CodeMirror) {
            const cm = cmElement.CodeMirror;
            cm.getAllMarks().forEach(mark => {
                if (mark.className && (mark.className.includes('concrete-syntax'))) {
                    mark.clear();
                }
            });
        }
    }
}

// Execute concrete syntax matching
window.executeConcreteSyntaxQuery = function(pattern, node, sourceCode) {
    if (!window.ConcreteSyntax) {
        debugError('Concrete syntax WASM not initialized');
        return [];
    }

    try {
        const wrappedNode = new TreeSitterNodeWrapper(node);
        const sourceBytes = new TextEncoder().encode(sourceCode);
        
        const matches = window.ConcreteSyntax.getAllMatches(
            pattern, 
            wrappedNode, 
            sourceBytes, 
            true, // recursive
            null  // replace_node
        );
        
        debugLog('Concrete syntax matches:', matches);
        return matches;
    } catch (error) {
        debugError('Concrete syntax matching error:', error);
        return [];
    }
};

// Integrate with playground's highlighting system
function setupPlaygroundIntegration() {
    debugLog('Setting up playground integration');
    debugLog('window.playground:', window.playground);
    
    // Check if playground and codeEditor are available, retry if not
    function attachCodeChangeListener() {
        if (window.playground && window.playground.codeEditor) {
            debugLog('Attaching code change listener to playground editor');
            let debounceTimer;
            
            window.playground.codeEditor.on('change', function() {
                debugLog('Code editor change detected');
                
                // Only re-execute if concrete syntax is active and has a pattern
                const queryType = document.querySelector('input[name="query-type"]:checked');
                if (queryType && queryType.value === 'concrete-syntax') {
                    debugLog('Concrete syntax is active, checking for pattern');
                    const concreteSyntaxEditor = getConcreteSyntaxEditor();
                    if (concreteSyntaxEditor) {
                        const pattern = concreteSyntaxEditor.getValue().trim();
                        if (pattern) {
                            debugLog('Pattern found, executing concrete syntax matching');
                            clearTimeout(debounceTimer);
                            debounceTimer = setTimeout(() => {
                                executeConcreteSyntaxPattern(pattern);
                            }, 500); // 500ms debounce for code changes
                        } else {
                            debugLog('No pattern found');
                        }
                    } else {
                        debugLog('No concrete syntax editor found');
                    }
                } else {
                    debugLog('Concrete syntax not active');
                }
            });
            
            return true; // Successfully attached
        }
        return false; // Failed to attach
    }
    
    // Try to attach immediately
    if (!attachCodeChangeListener()) {
        // If it fails, retry periodically
        const retryInterval = setInterval(() => {
            if (attachCodeChangeListener()) {
                clearInterval(retryInterval);
                debugLog('Code change listener attached after retry');
            }
        }, 100);
        
        // Give up after 10 seconds
        setTimeout(() => {
            clearInterval(retryInterval);
            debugLog('Gave up trying to attach code change listener');
        }, 10000);
    }
    
    // Get access to playground's CodeMirror instance and highlighting functions
    window.playground.highlightConcreteSyntaxMatches = function(matches) {
        debugLog('highlightConcreteSyntaxMatches function called with:', matches);
        // Clear existing highlights first
        const codeEditor = window.playground.codeEditor;
        if (!codeEditor) return;

        // Clear previous concrete syntax highlights
        codeEditor.getAllMarks().forEach(mark => {
            if (mark.className && mark.className.startsWith('concrete-syntax')) {
                mark.clear();
            }
        });

        if (!matches || matches.length === 0) return;

        // Use playground's color scheme for consistency
        const colors = window.playground.LIGHT_COLORS || [
            "#0550ae", "#ab5000", "#116329", "#844708", "#6639ba", 
            "#7d4e00", "#0969da", "#1a7f37", "#cf222e", "#8250df"
        ];

        debugLog('Highlighting matches:', matches);
        
        matches.forEach((match, index) => {
            debugLog('Match', index, ':', match);
            // Handle both snake_case and camelCase field names
            const range = match.range;
            const startPoint = range.startPoint || range.start_point;
            const endPoint = range.endPoint || range.end_point;
            
            if (range && startPoint && endPoint) {
                const from = {
                    line: startPoint.row,
                    ch: startPoint.column
                };
                const to = {
                    line: endPoint.row, 
                    ch: endPoint.column
                };

                // Highlight the full match with primary color
                codeEditor.markText(from, to, {
                    css: `background-color: ${colors[0]}20; border: 1px solid ${colors[0]}60;`,
                    title: `Concrete Syntax Match ${index + 1}`
                });

                // Highlight captured variables with different colors
                const captures = match.matches;
                if (captures && typeof captures === 'object') {
                    let colorIndex = 1;
                    debugLog('Match captures:', captures);
                    
                    // Handle both Map and plain object
                    const captureEntries = captures instanceof Map ? 
                        Array.from(captures.entries()) : 
                        Object.entries(captures);
                    
                    captureEntries.forEach(([key, captureData]) => {
                        debugLog('Capture:', key, captureData);
                        if (key !== '*' && captureData) {
                            const captureColor = colors[colorIndex % colors.length];
                            
                            // Check if captureData has range information (CapturedNode structure)
                            const captureRange = captureData.range;
                            const captureStartPoint = captureRange?.startPoint || captureRange?.start_point;
                            const captureEndPoint = captureRange?.endPoint || captureRange?.end_point;
                            
                            if (captureRange && captureStartPoint && captureEndPoint) {
                                // Use exact range from capture data
                                const captureFrom = {
                                    line: captureStartPoint.row,
                                    ch: captureStartPoint.column
                                };
                                const captureTo = {
                                    line: captureEndPoint.row,
                                    ch: captureEndPoint.column
                                };
                                
                                debugLog(`Highlighting capture ${key} from (${captureFrom.line},${captureFrom.ch}) to (${captureTo.line},${captureTo.ch})`);
                                
                                codeEditor.markText(captureFrom, captureTo, {
                                    css: `background-color: ${captureColor}30; border: 1px solid ${captureColor}; border-radius: 2px;`,
                                    title: `${key}: ${captureData.text || captureData}`
                                });
                            } else if (typeof captureData === 'string') {
                                // Fallback: find the capture text within the match
                                const matchText = match.matchedString || match.matched_string || '';
                                const captureStart = matchText.indexOf(captureData);
                                
                                if (captureStart >= 0) {
                                    const captureFrom = {
                                        line: from.line,
                                        ch: from.ch + captureStart
                                    };
                                    const captureTo = {
                                        line: from.line, 
                                        ch: from.ch + captureStart + captureData.length
                                    };
                                    
                                    debugLog(`Highlighting string capture ${key} from (${captureFrom.line},${captureFrom.ch}) to (${captureTo.line},${captureTo.ch})`);
                                    
                                    codeEditor.markText(captureFrom, captureTo, {
                                        css: `background-color: ${captureColor}30; border: 1px solid ${captureColor}; border-radius: 2px;`,
                                        title: `${key}: ${captureData}`
                                    });
                                }
                            }
                            
                            colorIndex++;
                        }
                    });
                }
            }
        });
    };
}

// Initialize when DOM is ready
document.addEventListener('DOMContentLoaded', initConcreteSyntaxUI);

// Debug function to test the integration
window.testConcreteSyntaxIntegration = function() {
    const results = {
        playground: !!window.playground,
        codeEditor: !!(window.playground && window.playground.codeEditor),
        concreteSyntax: !!window.ConcreteSyntax,
        queryType: null,
        pattern: null,
        concreteSyntaxEditor: null
    };
    
    const queryType = document.querySelector('input[name="query-type"]:checked');
    if (queryType) {
        results.queryType = queryType.value;
    }
    
    const concreteSyntaxEditor = getConcreteSyntaxEditor();
    if (concreteSyntaxEditor) {
        results.concreteSyntaxEditor = true;
        results.pattern = concreteSyntaxEditor.getValue().trim();
    } else {
        results.concreteSyntaxEditor = false;
    }
    
    console.log('Concrete Syntax Integration Status:', results);
    
    if (results.playground && results.codeEditor && results.concreteSyntax) {
        console.log('✅ All systems operational!');
        if (results.queryType === 'concrete-syntax' && results.pattern) {
            console.log('✅ Ready for concrete syntax matching');
        } else {
            console.log('ℹ️ Switch to concrete syntax mode and enter a pattern to test');
        }
    } else {
        console.log('❌ Some components not ready:', {
            needsPlayground: !results.playground,
            needsCodeEditor: !results.codeEditor,
            needsConcreteSyntax: !results.concreteSyntax
        });
    }
    
    return results;
};

// Hook into playground initialization to access tree and source
const originalInitializePlayground = window.initializePlayground;
window.initializePlayground = function(options) {
    const result = originalInitializePlayground ? originalInitializePlayground(options) : null;
    
    // Set up multiple checks to ensure playground is ready
    let setupAttempts = 0;
    const maxAttempts = 50; // 5 seconds total
    
    const checkPlayground = setInterval(() => {
        setupAttempts++;
        
        if (window.playground && window.playground.codeEditor) {
            clearInterval(checkPlayground);
            debugLog('Playground fully accessible for concrete syntax integration');
            
            // Give it a bit more time to ensure everything is initialized
            setTimeout(() => {
                setupPlaygroundIntegration();
            }, 100);
        } else if (setupAttempts >= maxAttempts) {
            clearInterval(checkPlayground);
            debugLog('Timeout waiting for playground initialization');
            // Try setup anyway in case some parts work
            setupPlaygroundIntegration();
        }
    }, 100);
    
    return result;
};

// Also set up a fallback initialization
document.addEventListener('DOMContentLoaded', function() {
    // Wait a bit for everything to load, then try setup
    setTimeout(() => {
        if (window.playground && window.playground.codeEditor) {
            debugLog('Fallback playground integration setup');
            setupPlaygroundIntegration();
        }
    }, 2000);
});
