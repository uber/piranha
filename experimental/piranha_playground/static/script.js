(async () => {
  /**
   * UI elements in the script
   */
  const elements = {
    codeInputBefore: document.getElementById("code-input-before"),
    codeIntputAfter: document.getElementById("code-input-after"),
    languageSelect: document.getElementById("language-select"),
    queryInput: document.getElementById("query-input"),
    explanationInput: document.getElementById("explanation-input"),
    submitFolderButton: document.getElementById("submit-button-folder"),
    submitButtonInfer: document.getElementById("submit-button-infer"),
    submitButtonImprovement: document.getElementById(
      "submit-button-improvement",
    ),
    testButton: document.getElementById("submit-button-test"),
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

  /**
   * Function to load a script dynamically
   * @param {string} url - The URL of the script to load
   * @param {Function} callback - Function to be called once the script has loaded
   */
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

  // Define a list of button names
  let buttonNames = ["test", "infer", "improvement", "folder"];

  // Initial text of the buttons
  let buttonElements = {};
  buttonNames.forEach((name) => {
    let element = document.getElementById("submit-button-" + name);
    buttonElements[name] = element.innerText;
  });

  function displayButton(disabled, textContent, name) {
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

  /* Function to make an async request to the provided URL
   * @param {string} url - The URL to send the request to
   * @param {Object} requestData - The request data to be sent
   * @param {string} buttonName - The name of the button that triggered the request
   * @param {Function} onSuccess - Function to be called if the request was successful
   */
  async function makeRequest(url, requestData, buttonName, onSuccess) {
    const button = displayButton(true, "Processing...", buttonName);
    const response = await fetch(url, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(requestData),
    });
    const data = await response.json();

    if (response.status === 200) {
      onSuccess(data);
      button.classList.add("btn-success");
      displayButton(false, `Success`, buttonName);
    } else {
      button.classList.add("btn-danger");
      displayButton(false, `Error`, buttonName);
      showAlert(data.error || "An error occurred", "danger");
    }

    setTimeout(() => {
      button.classList.remove("btn-success", "btn-danger");
      displayButton(false, buttonElements[buttonName], buttonName);
    }, 3000);
  }

  async function emitInferEvent() {
    const sourceCode = editors.codeBefore.getValue();
    const targetCode = editors.codeAfter.getValue();
    const language = elements.languageSelect.value;

    await makeRequest(
      "http://127.0.0.1:5000/infer_rule_graph",
      {
        source_code: sourceCode,
        target_code: targetCode,
        language: language,
      },
      "infer",
      (data) => {
        updateInterface(data.rules);
        // emitImproveEventAux("general");
      },
    );
  }

  async function emitTestEvent() {
    const sourceCode = editors.codeBefore.getValue();
    const rules = editors.queryEditor.getValue();
    const language = elements.languageSelect.value;

    await makeRequest(
      "http://127.0.0.1:5000/test_rule",
      {
        source_code: sourceCode,
        rules: rules,
        language: language,
      },
      "test",
      (data) => {
        editors.codeAfter.setValue(data.refactored_code);
      },
    );
  }

  async function emitRefactorEvent() {
    const folderPath = document.getElementById("folder-input").value;
    const rules = editors.queryEditor.getValue();
    const language = elements.languageSelect.value;

    await makeRequest(
      "http://127.0.0.1:5000/refactor_codebase",
      {
        rules: rules,
        folder_path: folderPath,
        language: language,
      },
      "folder",
      (data) => {},
    );
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

    await makeRequest(
      "http://127.0.0.1:5000/improve_rule_graph",
      {
        source_code: sourceCode,
        target_code: targetCode,
        language: language,
        rules: rules,
        requirements: requirements,
        option: option,
      },
      "improvement",
      (data) => {
        updateInterface(data.rule);
      },
    );
  }

  /**
   * Function to update the interface with the provided rule, and display the improvement button
   * @param rule: The rule to be displayed
   */

  function updateInterface(rule) {
    document.getElementById("explanation-container").style.display = "block";
    document.getElementById("submit-button-improvement").style.display =
      "block";
    editors.queryEditor.setValue(rule);
    if (editors.requirementsEditor.getValue() === "") {
      editors.requirementsEditor.setValue(" ");
    }
  }

  /**
   * Display an alert message
   * @param {string} message - The message to be displayed
   * @param {string} alertType - The type of the alert to be displayed
   */
  function showAlert(message, alertType) {
    const alertPlaceholder = document.getElementById("alert-placeholder");
    const alertBox = document.createElement("div");

    alertBox.className = `alert alert-${alertType} fade show`;
    alertBox.role = "alert";
    alertBox.innerHTML = message;

    alertPlaceholder.appendChild(alertBox);

    setTimeout(() => {
      alertPlaceholder.removeChild(alertBox);
    }, 3000);
  }
})();
