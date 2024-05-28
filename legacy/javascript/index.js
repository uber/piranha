const fs = require('fs');
const path = require('path');
const fg = require('fast-glob');
const recast = require("recast");
const ArgumentParser = require('argparse').ArgumentParser;
const jsRefactor = require("./src/refactor");
const { parseOptions } = require("./config/utils");
const templateRefactor = require("./src/template-refactor");
const templateRecast = require('ember-template-recast');

const parser = new ArgumentParser();
const requiredArgs = parser.addArgumentGroup();

requiredArgs.addArgument(['--flag'], {
  help: 'Name of the stale flag',
  required: true,
});

requiredArgs.addArgument(['--path'], {
  help: 'Absolute directory path  for flag cleanup',
  require: true
});

requiredArgs.addArgument(['--output'], {
  help: 'Absolute destination path of the refactored output files. File is modified in-place by default.'
});

requiredArgs.addArgument(['--properties'], {
  help: 'Path of configuration file for Piranha',
  required: true,
});

requiredArgs.addArgument(['--enable-log'], {
  help: 'Print cleanup logs',
  action: 'storeTrue',
  defaultValue: false,
});

const args = parser.parseArgs();
let flagname = args.flag;

const propertiesInJson = fs.readFileSync(args.properties);
const properties = JSON.parse(propertiesInJson);
console.log(JSON.stringify(properties));

let jsFiles = [], templateFiles = [];
if(args.path.endsWith(".js")) {
  jsFiles = [args.path];
} else if(args.path.endsWith(".hbs")) {
  templateFiles = [args.path];
} else {
  const excludeNodeModules = `!${path.join(args.path, 'node_modules/**')}`;
  const excludeDist = `!${path.join(args.path, 'dist/**')}`;
  jsFiles = fg.sync([
    path.join(args.path, '**/*.js'), excludeNodeModules, excludeDist]
  );
  templateFiles = fg.sync([path.join(args.path, '**/*.hbs'), excludeNodeModules, excludeDist]);
}
console.log("Total JS files in frontend/app: ", jsFiles.length);
console.log("Total Templates files in frontend/app: ", templateFiles.length, "\n");

if(args.output) {
  fs.rmdirSync(args.output, { recursive: true })
}

const filesHavingFlagKeyword = [], allModifiedFiles = [], templateToCleanupInfoMap = {};

//cleanup js files
for (let filename of jsFiles) {
  const content = fs.readFileSync(filename, 'utf-8');
  if(!content.includes(flagname)) continue;
  const ast = recast.parse(content, parseOptions);

  const engine = new jsRefactor.RefactorEngine(
    ast,
    properties,
    true,
    flagname,
    15,
    args.enable_log,
    false,
    filename,
  );
  const { changed, hasFlagKeywordInFile, templateCleanupInfo } = engine.refactorPipeline();
  if (path.parse(filename).base === 'component.js') {
    const templatePath = path.join(filename, "..", "template.hbs");
    templateToCleanupInfoMap[templatePath] = templateCleanupInfo;
  }

  if (hasFlagKeywordInFile) filesHavingFlagKeyword.push(filename);
  if (changed) {
    allModifiedFiles.push(filename);
  } else {
    continue;
  }

  const output = recast.print(ast).code;
  writeOutput(filename, output);
}

//cleanup templates
for (let filename of templateFiles) {
  const content = fs.readFileSync(filename, 'utf8');

  const cleanupInfo = templateToCleanupInfoMap[filename] || { properties: [] };
  let hasFlagOrProperty = content.includes(flagname) || cleanupInfo.properties.some((prop) => content.includes(prop));
  if(!hasFlagOrProperty) continue;

  const ast = templateRecast.parse(content);

  const engine = new templateRefactor.TemplateRefactorEngine({
    ast,
    properties,
    flagname,
    filename,
    cleanupInfo,
    print_to_console: args.enable_log,
  });

  const { changed, hasFlagKeywordInFile } = engine.refactorPipeline();

  if (hasFlagKeywordInFile) filesHavingFlagKeyword.push(filename);
  if (changed) {
    allModifiedFiles.push(filename);
  } else {
    continue;
  }

  const output = templateRecast.print(ast);
  writeOutput(filename, output);
}

function writeOutput(filename, output) {
  const { base, dir } = path.parse(filename);
  let outputDir = args.output ? path.join(args.output, dir) : dir;
  if (!fs.existsSync(outputDir)) {
    fs.mkdirSync(outputDir, { recursive: true });
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
if (diff1.length) {
  console.log("Attention: These files might need modification, please check them manually", diff1);
}
if (diff2.length) {
  console.log("Attention: These files might not need modification, please check them manually", diff2);
}
console.log(`Total ${allModifiedFiles.length} files are modified by tool cleanup`)