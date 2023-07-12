(async () => {
  const elements = {
    codeInputBefore: document.getElementById("code-input-before"),
    codeIntputAfter: document.getElementById("code-input-after"),
    languageSelect: document.getElementById("language-select"),
    queryInput: document.getElementById("query-input"),
    gptExplanation: document.getElementById("gpt-rule-explanation"),
    explanationInput: document.getElementById("explanation-input"),
    submitFolderButton: document.getElementById("submit-button-folder"),
    submitButtonInfer: document.getElementById("submit-button-infer"),
    submitButtonImprovement: document.getElementById(
      "submit-button-improvement",
    ),
    testButton: document.getElementById("test-button"),
  };

  elements.languageSelect.addEventListener("change", handleLanguageChange);
  handleLanguageChange();

  const editors = {
    codeBefore: CodeMirror.fromTextArea(
      elements.codeInputBefore,
      editorOptions("javascript"),
    ),
    codeAfter: CodeMirror.fromTextArea(
      elements.codeIntputAfter,
      editorOptions("javascript"),
    ),
    queryEditor: CodeMirror.fromTextArea(
      elements.queryInput,
      editorOptions("toml"),
    ),
    requirementsEditor: CodeMirror.fromTextArea(elements.explanationInput, {
      lineWrapping: true,
    }),
  };

  function editorOptions(mode) {
    return {
      lineNumbers: true,
      showCursorWhenSelecting: true,
      mode: mode,
    };
  }

  function loadScript(url, callback) {
    const script = document.createElement("script");
    script.type = "text/javascript";
    script.src = url;
    script.onload = callback;
    document.body.appendChild(script);
  }

  function handleLanguageChange() {
    const selectedLanguage = elements.languageSelect.value;
    const langName =
      selectedLanguage === "java" ? "javascript" : selectedLanguage;
    const scriptUrl = `${codeMirror}/mode/${langName}/${langName}.js`;

    loadScript(scriptUrl, function () {
      editors.codeBefore.setOption("mode", langName);
      editors.codeAfter.setOption("mode", langName);
    });
  }

  function displayButton(disabled, textContent, name) {
    // Change the button ending in -name
    const button = document.getElementById("submit-button-" + name);
    const buttonText = document.getElementById("button-text-" + name);
    const spinner = document.getElementById("spinner-" + name);

    button.disabled = disabled;
    spinner.style.display = disabled ? "inline-block" : "none";
    buttonText.textContent = textContent;
    return button;
  }

  elements.submitFolderButton.addEventListener("click", emitRefactorEvent);
  elements.submitButtonInfer.addEventListener("click", emitInferEvent);
  elements.submitButtonImprovement.addEventListener("click", emitImproveEvent);
  elements.testButton.addEventListener("click", emitTestEvent);

  async function emitInferEvent() {
    const sourceCode = editors.codeBefore.getValue();
    const targetCode = editors.codeAfter.getValue();
    const language = elements.languageSelect.value;

    displayButton(true, "Inferring...", "infer");

    const response = await fetch("http://127.0.0.1:5000/infer_rule_graph", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        source_code: sourceCode,
        target_code: targetCode,
        language: language,
      }),
    });

    const data = await response.json();
    let button = elements.submitButtonInfer;

    if (response.status === 200) {
      updateInterface(data.rules);

      displayButton(false, "Successfully inferred rules", "infer");
      button.classList.add("btn-success");

      // Set a timeout to fade the button back to the original state
      setTimeout(() => {
        button.classList.remove("btn-success", "btn-danger");
        displayButton(false, "Infer rules from templates", "infer");
      }, 3000);
    } else {
      displayButton(false, "Unable to infer rules", "infer");
      button.classList.add("btn-danger");

      // Set a timeout to fade the button back to the original state
      setTimeout(() => {
        button.classList.remove("btn-success", "btn-danger");
        displayButton(false, "Infer rules from templates", "infer");
      }, 3000);
    }

    await emitImproveEventAux("general");
  }

  async function emitTestEvent() {
    const sourceCode = editors.codeBefore.getValue();
    const rules = editors.queryEditor.getValue();
    const language = elements.languageSelect.value;

    const response = await fetch("http://127.0.0.1:5000/test_rule", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        source_code: sourceCode,
        rules: rules,
        language: language,
      }),
    });

    let button = document.getElementById("test-button");
    button.disabled = true;

    const data = await response.json();
    // Add a new socket listener for the test result
    if (response.status === 200) {
      return updateTestButton(
        data,
        "btn-success",
        "Successfully applied rule",
        data.refactored_code,
      );
    } else {
      return updateTestButton(data, "btn-danger", "Error", data.error);
    }
  }

  async function emitRefactorEvent() {
    const folderPath = document.getElementById("folder-input").value;
    const rules = editors.queryEditor.getValue();
    const language = elements.languageSelect.value;

    const response = await fetch("http://127.0.0.1:5000/refactor_codebase", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        rules: rules,
        folder_path: folderPath,
        language: language,
      }),
    });

    displayButton(true, "Processing...", "folder");
    const data = await response.json();

    let button = document.getElementById("submit-button-folder");
    if (response.status === 200) {
      button.classList.add("btn-success");
      displayButton(false, "Successfully refactored codebase", "folder");
    } else {
      button.classList.add("btn-danger");
      displayButton(false, "Unable to refactor codebase", "folder");
    }

    setTimeout(() => {
      button.classList.remove("btn-success", "btn-danger");
      displayButton(false, "Apply rules", "folder");
    }, 3000);
  }

  async function emitImproveEvent() {
    await emitImproveEventAux("user");
  }

  async function emitImproveEventAux(option) {
    const sourceCode = editors.codeBefore.getValue();
    const targetCode = editors.codeAfter.getValue();
    const language = elements.languageSelect.value;
    const rules = editors.queryEditor.getValue();
    const requirements = editors.requirementsEditor.getValue();
    displayButton(true, "Improving...", "improvement");

    const response = await fetch("http://127.0.0.1:5000/improve_rule_graph", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        source_code: sourceCode,
        target_code: targetCode,
        language: language,
        rules: rules,
        requirements: requirements,
        option: option,
      }),
    });

    const data = await response.json();
    console.log(data);

    let button = document.getElementById("submit-button-improvement");
    if (response.status === 200) {
      var converter = new showdown.Converter();
      var markdown = converter.makeHtml(data.gpt_output);
      displayButton(false, "Successfully improved rule", "improvement");
      button.classList.add("btn-success");
      updateInterface(data.rule);
    } else {
      displayButton(false, "Unable to improve / generate rule", "improvement");
      button.classList.add("btn-danger");
    }

    // Set a timeout to fade the button back to the original state
    setTimeout(() => {
      button.classList.remove("btn-success", "btn-danger");
      displayButton(false, "Improve rule", "improvement");
    }, 3000);
  }

  function updateTestButton(data, status, buttonText, editorValue) {
    console.log(data);
    let button = document.getElementById("test-button");
    button.disabled = false;
    button.classList.remove("btn-success", "btn-danger");
    button.classList.add(status);
    button.textContent = buttonText;
    editors.codeAfter.setValue(editorValue);

    // Set a timeout to fade the button back to the original state
    setTimeout(() => {
      button.classList.remove("btn-success", "btn-danger");
      button.classList.add("btn-primary");
      button.textContent = "Apply rule to code before";
    }, 3000);

    return button;
  }

  function updateInterface(rule) {
    document.getElementById("query-container").style.display = "block";
    document.getElementById("gpt-rule-explanation-container").style.display =
      "block";
    document.getElementById("explanation-container").style.display = "block";
    document.getElementById("path-container").style.display = "block";
    document.getElementById("submit-button-improvement").style.display =
      "block";
    editors.queryEditor.setValue(rule);
    if (editors.requirementsEditor.getValue() === "") {
      editors.requirementsEditor.setValue(" ");
    }
  }
})();
