(async () => {
  const elements = {
    codeInputBefore: document.getElementById("code-input-before"),
    codeIntputAfter: document.getElementById("code-input-after"),
    languageSelect: document.getElementById("language-select"),
    queryInput: document.getElementById("query-input"),
    gptExplanation: document.getElementById("gpt-rule-explanation"),
    explanationInput: document.getElementById("explanation-input"),
    submitFolderButton: document.getElementById("submit-button-folder"),
    submitButton: document.getElementById("submit-button"),
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

  const socket = io.connect("http://127.0.0.1:5000");

  elements.submitFolderButton.addEventListener("click", async function () {
    emitRefactorEvent();
  });

  elements.submitButton.addEventListener("click", async function () {
    emitInferEvent();
  });

  elements.submitButtonImprovement.addEventListener("click", async function () {
    emitImproveEvent();
  });

  elements.testButton.addEventListener("click", async function () {
    emitTestEvent();
  });

  // Add a function to emit test event
  function emitTestEvent() {
    const sourceCode = editors.codeBefore.getValue();
    const rules = editors.queryEditor.getValue();
    const language = elements.languageSelect.value;
    // Here you may want to adjust the data to fit your backend needs
    socket.emit("test_rule", {
      source_code: sourceCode,
      rules: rules,
      language: language,
    });

    let button = document.getElementById("test-button");
    button.disabled = true;
  }
  function emitRefactorEvent() {
    const folderPath = document.getElementById("folder-input").value;
    const rules = editors.queryEditor.getValue();
    const language = elements.languageSelect.value;
    socket.emit("refactor_codebase", {
      rules: rules,
      folder_path: folderPath,
      language: language,
    });
    displayButton(true, "Processing...", "folder");
  }

  function emitInferEvent() {
    const sourceCode = editors.codeBefore.getValue();
    const targetCode = editors.codeAfter.getValue();
    const language = elements.languageSelect.value;
    socket.emit("infer_piranha", {
      source_code: sourceCode,
      target_code: targetCode,
      language: language,
    });
    // elements.submitButton.style.display = "none";
    const button = displayButton(
      true,
      "GPT is improving the rule ...",
      "improvement",
    );
    button.style.display = "block";
  }

  function emitImproveEvent() {
    const requirements = editors.requirementsEditor.getValue();
    const rules = editors.queryEditor.getValue();
    const language = elements.languageSelect.value;
    displayButton(true, "Improving...", "improvement");
    socket.emit("improve_piranha", {
      rules: rules,
      requirements: requirements,
      language: language,
    });
  }

  socket.on("infer_result", function (data) {
    var converter = new showdown.Converter();
    var markdown = converter.makeHtml(data.gpt_output);
    console.log(data);

    // update the explanation div
    document.getElementById("explanation").innerHTML = markdown;

    let button = document.getElementById("submit-button-improvement");
    if (data.test_result === "Success") {
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
  });

  socket.on("infer_progress", function (data) {
    var converter = new showdown.Converter();
    var markdown = converter.makeHtml(data.gpt_output);
    console.log(data);

    // update the explanation div
    document.getElementById("explanation").innerHTML = markdown;

    updateInterface(data.rule);
  });

  socket.on("refactor_progress", function (data) {
    // change button to green to indicate success
    // otherwise red to show error
    // and then revert after timeout

    let button = document.getElementById("submit-button-folder");
    if (data.result === "Success") {
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
  });

  // Add a new socket listener for the test result
  socket.on("test_result", function (data) {
    console.log(data);
    // Here you may want to handle the test result
    let button = document.getElementById("test-button");
    button.disabled = false;
    // Add appropriate CSS class based on the test result
    button.classList.remove("btn-success", "btn-danger");
    if (data.test_result === "Success") {
      button.classList.add("btn-success");
    } else {
      button.classList.add("btn-danger");
    }
    button.textContent = data.test_result;
    editors.codeAfter.setValue(data.refactored_code);

    // Set a timeout to fade the button back to the original state
    setTimeout(() => {
      button.classList.remove("btn-success", "btn-danger");
      button.classList.add("btn-primary");
      button.textContent = "Apply rule to code before";
    }, 3000);

    return button;
  });

  function updateInterface(rule) {
    document.getElementById("query-container").style.display = "block";
    document.getElementById("gpt-rule-explanation-container").style.display =
      "block";
    document.getElementById("explanation-container").style.display = "block";
    document.getElementById("path-container").style.display = "block";
    editors.queryEditor.setValue(rule);
    if (editors.requirementsEditor.getValue() === "") {
      editors.requirementsEditor.setValue(" ");
    }
  }
})();
