const fs = require('fs');
const path = require('path');
const fg = require('fast-glob');
const recast = require("recast");
const ArgumentParser = require('argparse').ArgumentParser;
const jsRefactor = require("./src/refactor");
const {parseOptions} = require("./config/utils");

let jsFiles = fg.sync([path.join(process.cwd(), 'app/**/*.js')]);
console.log("Total JS files in frontend/app: ", jsFiles.length);

const parser = new ArgumentParser();
const requiredArgs = parser.addArgumentGroup();
requiredArgs.addArgument(['-f', '--flag'], {
  help: 'Name of the stale flag',
  required: true,
});

requiredArgs.addArgument(['--modify-file'], {
  help: 'Name of the output file',
  action: 'storeTrue',
  defaultValue: false,
});

const args = parser.parseArgs();

let flagname = args.flag;
if (!flagname) {
  throw "Flag name is required!";
}

const filesHavingFlagKeyword = [], allModifiedFiles = [];
//cleanup js files
for (let filename of jsFiles) {
  console.log('Parsing file', filename);
  const ast = recast.parse(fs.readFileSync(filename, 'utf-8'), parseOptions);

  const properties = {
    "methodProperties": [
      {
        "methodName": "hasTempFeature",
        "flagType": "treated",
        "argumentIndex": 0
      }
    ]
  }
  const engine = new jsRefactor.RefactorEngine(
    ast,
    properties,
    true,
    flagname,
    15,
    true,
    true,
    filename,
  );
  const { changed, isFlagKeywordFoundInFile} = engine.refactorPipeline();

  if(isFlagKeywordFoundInFile) filesHavingFlagKeyword.push(filename);
  if(changed) {
    allModifiedFiles.push(filename);
  } else {
    continue;
  }

  const output = recast.print(ast).code;
  writeOutput(filename, output);
}

function writeOutput(filename, output) {
  const {base, dir} = path.parse(filename);
  const outputDir = args.modify_file ? dir : path.join(process.cwd(), !args.modify_file ? 'output' : '', dir);
  if (!fs.existsSync(outputDir)) {
    fs.mkdirSync(outputDir, {recursive: true});
  }
  const outputpath = path.join(outputDir, base);
  fs.writeFileSync(outputpath, output, function (err) {
    if (err) {
      return console.log(err);
    }
    console.log(`Output written to ${outputpath}`);
  });
}

const diff1 = filesHavingFlagKeyword.filter(filename => !allModifiedFiles.includes(filename));
const diff2 = allModifiedFiles.filter(filename => !filesHavingFlagKeyword.includes(filename));
if(diff1.length) {
  console.log("Attention: These files might need modification, please check them manually", diff1);
} else if(diff2.length) {
  console.log("Attention: These files might not need modification, please check them manually", diff2);
} else {
  console.log(`Total ${allModifiedFiles.length} files are modified`);
}