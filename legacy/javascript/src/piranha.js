/**
 * Copyright (c) 2019 Uber Technologies, Inc.
 *
 * <p>Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file
 * except in compliance with the License. You may obtain a copy of the License at
 *
 * <p>http://www.apache.org/licenses/LICENSE-2.0
 *
 * <p>Unless required by applicable law or agreed to in writing, software distributed under the
 * License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either
 * express or implied. See the License for the specific language governing permissions and
 * limitations under the License.
 */

// Contributed by Ananye Agarwal (anag004) and Sarthak Behera (s7rthak). The authors are from the Indian Institute of Technology, Delhi (IIT Delhi).

const refactor = require('./refactor'); // functions to carry out different steps of the refactoring scheme
const fs = require('fs'); // Read and write files
const recast = require('recast'); // Parser
const colors = require('colors'); // Print error in red
const ArgumentParser = require('argparse').ArgumentParser; // Parses command-line arguments
const winston = require('winston'); // Logger
const parser = new ArgumentParser({ addHelp: true });
// Large but finite number so that a fixed point is realised but the program doesn't go into an infinite loop in case of a bug
const max_iters = 15;
const config_checker = require('./config_checker'); // Error-checking for the properties file
const source_checker = require('./source_checker');
const process = require('process');
const { parseOptions } = require('../config/utils');

// By default argparse prints all args under 'optional arguments'
// A new argument group is needed to print required args separately
const requiredArgs = parser.addArgumentGroup({ title: 'Required arguments' });

requiredArgs.addArgument(['-s', '--source'], {
    help: 'Path of input file for refactoring',
    required: true,
});

requiredArgs.addArgument(['-f', '--flag'], {
    help: 'Name of the stale flag',
    required: true,
});

requiredArgs.addArgument(['-p', '--properties'], {
    help: 'Path of configuration file for Piranha',
    required: true,
});

parser.addArgument(['-o', '--output'], {
    help: 'Destination of the refactored output. File is modified in-place by default.',
    defaultValue: '',
});

parser.addArgument(['-t', '--treated'], {
    help: 'If this option is supplied, the flag is treated, otherwise it is control.',
    nargs: 'OPTIONAL',
});

parser.addArgument(['-n', '--max_cleanup_steps'], {
    help: 'The number of times literals should be simplified. Runs until fixed point by default.',
    type: parseInt,
});

// TODO implement logging functionality
// parser.addArgument(
//     ["-d", "--debug"],
//     {
//         help: "Log debugging output; Default is false",
//         nargs: 'OPTIONAL',
//     }
// );

parser.addArgument(['-c', '--keep_comments'], {
    help: 'To keep all comments',
    nargs: 'OPTIONAL',
});

const args = parser.parseArgs();
const flagname = args.flag;

let filename, properties;

try {
    filename = source_checker.checkSource(args.source);
    properties = config_checker.parseProperties(args.properties);
} catch (err) {
    console.log(colors.red(err.message));
    process.exit(1);
}

var max_cleanup_steps = max_iters;
if (args.max_cleanup_steps != null) {
    max_cleanup_steps = args.max_cleanup_steps;
}

var behaviour = args.treated != null;

const timestamp = Date.now();

if (args.debug != null) {
    refactor.logger.add(
        new winston.transports.File({
            filename: `error_${timestamp}.log`,
            level: 'error',
        }),
    );

    refactor.logger.add(new winston.transports.File({ filename: `combined_${timestamp}.log` }));
}

const keep_comments = args.keep_comments != null;

const ast = recast.parse(fs.readFileSync(filename, 'utf-8'), parseOptions);

const engine = new refactor.RefactorEngine(
    ast,
    properties,
    behaviour,
    flagname,
    max_cleanup_steps,
    true,
    keep_comments,
    filename,
);
engine.refactorPipeline();

var out_file = args.output;

// Default behavior is to modify in-place
if (out_file === '') {
    out_file = filename;
}

fs.writeFile(out_file, recast.print(ast).code, function (err) {
    if (err) {
        return console.log(err);
    }
    console.log(`Output written to ${out_file}`);
});
