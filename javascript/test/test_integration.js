const assert = require("assert");
const refactor = require("../src/refactor");
const fs = require("fs"); // Read and write files
const recast = require("recast"); // Parser
const max_cleanup_steps = 15;
const config_checker = require("../src/config_checker");
const properties = config_checker.parseProperties("config/properties.json");
const estraverse = require("estraverse");

var checkForPiranhaLiterals = (ast) => {
  var result = false;

  estraverse.traverse(ast, {
    leave: function (node) {
      if (node.createdByPiranha !== undefined) {
        result = true;
      }
    },
  });

  return result;
};

describe("piranha", () => {
  describe("#sample.js", () => {
    it("featureFlag treated", () => {
      const code = fs.readFileSync("./test/input/sample.js", "utf-8");
      const ast = recast.parse(code).program;
      const flagname = "featureFlag";
      const behaviour = true;
      const expected_code = fs.readFileSync(
        "./test/treated-expected/sample-featureFlag.js",
        "utf-8"
      );

      const engine = new refactor.RefactorEngine(
        ast,
        properties,
        behaviour,
        flagname,
        max_cleanup_steps,
        false,
        true
      );
      engine.refactorPipeline();
      const refactored_code = recast.print(ast).code;
      console.log(expected_code);
      assert(!checkForPiranhaLiterals(ast));
      assert(
        expected_code === refactored_code,
        `\nEXPECTED : ${JSON.stringify(
          expected_code
        )}\nREFACTORED : ${JSON.stringify(refactored_code)}`
      );
    });

    it("featureFlag control", () => {
      const code = fs.readFileSync("./test/input/sample.js", "utf-8");
      const ast = recast.parse(code).program;
      const flagname = "featureFlag";
      const behaviour = false;
      const expected_code = fs.readFileSync(
        "./test/control-expected/sample-featureFlag.js",
        "utf-8"
      );

      const engine = new refactor.RefactorEngine(
        ast,
        properties,
        behaviour,
        flagname,
        max_cleanup_steps,
        false,
        true
      );
      engine.refactorPipeline();
      const refactored_code = recast.print(ast).code;

      assert(!checkForPiranhaLiterals(ast));
      assert(
        expected_code === refactored_code,
        `\nEXPECTED : ${JSON.stringify(
          expected_code
        )}\nREFACTORED : ${JSON.stringify(refactored_code)}`
      );
    });
  });

  describe("#sample1.js", () => {
    it("featureFlag treated", () => {
      const code = fs.readFileSync("./test/input/sample1.js", "utf-8");
      const ast = recast.parse(code).program;
      const flagname = "featureFlag";
      const behaviour = true;
      const expected_code = fs.readFileSync(
        "./test/treated-expected/sample1-featureFlag.js",
        "utf-8"
      );

      const engine = new refactor.RefactorEngine(
        ast,
        properties,
        behaviour,
        flagname,
        max_cleanup_steps,
        false,
        true
      );
      engine.refactorPipeline();
      const refactored_code = recast.print(ast).code;

      assert(!checkForPiranhaLiterals(ast));
      assert(
        expected_code === refactored_code,
        `\nEXPECTED : ${JSON.stringify(
          expected_code
        )}\nREFACTORED : ${JSON.stringify(refactored_code)}`
      );
    });

    it("featureFlag control", () => {
      const code = fs.readFileSync("./test/input/sample1.js", "utf-8");
      const ast = recast.parse(code).program;
      const flagname = "featureFlag";
      const behaviour = false;
      const expected_code = fs.readFileSync(
        "./test/control-expected/sample1-featureFlag.js",
        "utf-8"
      );

      const engine = new refactor.RefactorEngine(
        ast,
        properties,
        behaviour,
        flagname,
        max_cleanup_steps,
        false,
        true
      );
      engine.refactorPipeline();
      const refactored_code = recast.print(ast).code;

      assert(!checkForPiranhaLiterals(ast));
      assert(
        expected_code === refactored_code,
        `\nEXPECTED : ${JSON.stringify(
          expected_code
        )}\nREFACTORED : ${JSON.stringify(refactored_code)}`
      );
    });
  });

  describe("#sample2.js", () => {
    it("featureOne treated", () => {
      const code = fs.readFileSync("./test/input/sample2.js", "utf-8");
      const ast = recast.parse(code).program;
      const flagname = "featureOne";
      const behaviour = true;
      const expected_code = fs.readFileSync(
        "./test/treated-expected/sample2-featureOne.js",
        "utf-8"
      );

      const engine = new refactor.RefactorEngine(
        ast,
        properties,
        behaviour,
        flagname,
        max_cleanup_steps,
        false,
        true
      );
      engine.refactorPipeline();
      const refactored_code = recast.print(ast).code;

      assert(!checkForPiranhaLiterals(ast));
      assert(
        expected_code === refactored_code,
        `\nEXPECTED : ${JSON.stringify(
          expected_code
        )}\nREFACTORED : ${JSON.stringify(refactored_code)}`
      );
    });

    it("featureOne control", () => {
      const code = fs.readFileSync("./test/input/sample2.js", "utf-8");
      const ast = recast.parse(code).program;
      const flagname = "featureOne";
      const behaviour = false;
      const expected_code = fs.readFileSync(
        "./test/control-expected/sample2-featureOne.js",
        "utf-8"
      );

      const engine = new refactor.RefactorEngine(
        ast,
        properties,
        behaviour,
        flagname,
        max_cleanup_steps,
        false,
        true
      );
      engine.refactorPipeline();
      const refactored_code = recast.print(ast).code;

      assert(!checkForPiranhaLiterals(ast));
      assert(
        expected_code === refactored_code,
        `\nEXPECTED : ${JSON.stringify(
          expected_code
        )}\nREFACTORED : ${JSON.stringify(refactored_code)}`
      );
    });

    it("featureTwo treated", () => {
      const code = fs.readFileSync("./test/input/sample2.js", "utf-8");
      const ast = recast.parse(code).program;
      const flagname = "featureTwo";
      const behaviour = true;
      const expected_code = fs.readFileSync(
        "./test/treated-expected/sample2-featureTwo.js",
        "utf-8"
      );

      const engine = new refactor.RefactorEngine(
        ast,
        properties,
        behaviour,
        flagname,
        max_cleanup_steps,
        false,
        true
      );
      engine.refactorPipeline();
      const refactored_code = recast.print(ast).code;

      assert(!checkForPiranhaLiterals(ast));
      assert(
        expected_code === refactored_code,
        `\nEXPECTED : ${JSON.stringify(
          expected_code
        )}\nREFACTORED : ${JSON.stringify(refactored_code)}`
      );
    });

    it("featureTwo control", () => {
      const code = fs.readFileSync("./test/input/sample2.js", "utf-8");
      const ast = recast.parse(code).program;
      const flagname = "featureTwo";
      const behaviour = false;
      const expected_code = fs.readFileSync(
        "./test/control-expected/sample2-featureTwo.js",
        "utf-8"
      );

      const engine = new refactor.RefactorEngine(
        ast,
        properties,
        behaviour,
        flagname,
        max_cleanup_steps,
        false,
        true
      );
      engine.refactorPipeline();
      const refactored_code = recast.print(ast).code;

      assert(!checkForPiranhaLiterals(ast));
      assert(
        expected_code === refactored_code,
        `\nEXPECTED : ${JSON.stringify(
          expected_code
        )}\nREFACTORED : ${JSON.stringify(refactored_code)}`
      );
    });
  });
});
