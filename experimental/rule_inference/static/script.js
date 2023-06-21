/* ########## THIS CODE COMES FROM THE TREE SITTER PLAYGROUND  ##########
   ########## https://github.com/tree-sitter/tree-sitter/      ##########

 */

let tree;

(async () => {
  const codeInputBefore = document.getElementById("code-input-before");
  const codeIntputAfter = document.getElementById("code-input-after");
  const languageSelect = document.getElementById("language-select");
  const queryInput = document.getElementById("query-input");
  const explanation_input = document.getElementById("explanation-input");
  languageSelect.addEventListener("change", handleLanguageChange);
  handleLanguageChange();

  const codeBefore = CodeMirror.fromTextArea(codeInputBefore, {
    lineNumbers: true,
    showCursorWhenSelecting: true,
    mode: "javascript",
  });

  const codeAfter = CodeMirror.fromTextArea(codeIntputAfter, {
    lineNumbers: true,
    showCursorWhenSelecting: true,
    mode: "javascript",
  });

  const queryEditor = CodeMirror.fromTextArea(queryInput, {
    lineNumbers: true,
    showCursorWhenSelecting: true,
    mode: "toml",
  });

  const explanationEditor = CodeMirror.fromTextArea(explanation_input, {
    lineNumbers: true,
    lineWrapping: true,
  });

  // Function to dynamically load script
  function loadScript(url, callback) {
    const script = document.createElement("script");
    script.type = "text/javascript";
    script.src = url;
    script.onload = callback;
    document.body.appendChild(script);
  }

  // Event listener for language change
  function handleLanguageChange() {
    const selectedLanguage = languageSelect.value;

    // Use baseURL variable to generate URL
    // if language is javascript turn ito to java
    const langName =
      selectedLanguage === "java" ? "javascript" : selectedLanguage;
    const scriptUrl = `${codeMirror}/mode/${langName}/${langName}.js`;

    loadScript(scriptUrl, function () {
      codeBefore.setOption("mode", langName);
      codeAfter.setOption("mode", langName);
    });
  }

  document
    .getElementById("submit-folder-button")
    .addEventListener("click", async function () {
      const folderPath = document.getElementById("folder-input").value; // Get the value of the folder path input

      const response = await fetch("/api/process_folder", {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify({
          folder_path: folderPath, // Include the folder path in the request body
        }),
      });

      // handle response...
    });

  // First, we need to establish a socket connection.
  const socket = io.connect("http://127.0.0.1:5000");

  // We can listen for the 'infer_result' event and react when it happens.
  socket.on("infer_result", function (data) {
    // This is where you could update your interface with the data.
    const toml = data.rule;
    document.getElementById("query-container").style.display = "block";
    queryEditor.setValue(toml);

    // Change the button to show that processing is happening
    const button = document.getElementById("submit-button");
    const buttonText = document.getElementById("button-text");
    const spinner = document.getElementById("spinner");

    button.disabled = false; // Enable button
    spinner.style.display = "none"; // Hide spinner
    buttonText.textContent = "Infer"; // Reset button text
  });

  socket.on("infer_progress", function (data) {
    // This is where you could update your interface with the data.
    const toml = data.rule;
    document.getElementById("query-container").style.display = "block";
    queryEditor.setValue(toml);
  });

  // To start the inference, we can emit the 'infer_piranha' event.
  document
    .getElementById("submit-button")
    .addEventListener("click", async function () {
      const sourceCode = codeBefore.getValue();
      const targetCode = codeAfter.getValue();
      const userExplanation = explanationEditor.getValue();
      const language = languageSelect.value;

      // We don't need to worry about disabling the button or showing a spinner,
      // since we can now update the interface in real-time as the inference happens.
      socket.emit("infer_piranha", {
        source_code: sourceCode,
        target_code: targetCode,
        language: language,
        hints: userExplanation,
      });

      // Change the button to show that processing is happening
      const button = document.getElementById("submit-button");
      const buttonText = document.getElementById("button-text");
      const spinner = document.getElementById("spinner");

      button.disabled = true; // Disable button
      spinner.style.display = "inline-block"; // Show spinner
      buttonText.textContent = "Processing..."; // Change button text
    });
})();
