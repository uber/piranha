/* ########## THIS CODE COMES FROM THE TREE SITTER PLAYGROUND  ##########
   ########## https://github.com/tree-sitter/tree-sitter/      ##########

 */

let tree;

(async () => {
  const codeInputBefore = document.getElementById("code-input-before");
  const codeIntputAfter = document.getElementById("code-input-after");
  const languageSelect = document.getElementById("language-select");
  const queryInput = document.getElementById("query-input");

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
    .getElementById("submit-button")
    .addEventListener("click", async function () {
      const sourceCode = codeBefore.getValue();
      const targetCode = codeAfter.getValue();
      const language = languageSelect.value;

      const response = await fetch("/api/infer_piranha", {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify({
          source_code: sourceCode,
          target_code: targetCode,
          language: language,
        }),
      });

      if (response.ok) {
        const data = await response.json();
        const toml = data[1]; // Get the second item from the response array
        queryEditor.setValue(toml); // Set the TOML in the queryEditor
      } else {
        console.error("Error:", response.status, response.statusText);
      }
    });
})();
