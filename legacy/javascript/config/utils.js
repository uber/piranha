const babel = require("@babel/core");
const hasOwn = Object.prototype.hasOwnProperty;

const parseOptions = {
  parser: {
    parse: function (source, options) {
      const babelOptions = getBabelOptions(options);
      babelOptions.plugins.push("jsx", "flow", "decoratorAutoAccessors");
      return adjustComments(require("@babel/parser").parse(source, babelOptions));
      // return require("@babel/parser").parse(source, babelOptions);
    }
  }
}

function getOption(options, key, defaultValue) {
  if (options && hasOwn.call(options, key)) {
    return options[key];
  }
  return defaultValue;
}

function getBabelOptions(options) {
  return {
    sourceType: getOption(options, "sourceType", "module"),
    strictMode: getOption(options, "strictMode", false),
    allowImportExportEverywhere: true,
    allowReturnOutsideFunction: true,
    startLine: 1,
    tokens: true,
    plugins: [
      "asyncGenerators",
      "bigInt",
      "classPrivateMethods",
      "classPrivateProperties",
      "classProperties",
      "classStaticBlock",
      "decimal",
      "decorators-legacy",
      "doExpressions",
      "dynamicImport",
      "exportDefaultFrom",
      "exportExtensions",
      "exportNamespaceFrom",
      "functionBind",
      "functionSent",
      "importAssertions",
      "importMeta",
      "nullishCoalescingOperator",
      "numericSeparator",
      "objectRestSpread",
      "optionalCatchBinding",
      "optionalChaining",
      [
        "pipelineOperator",
        {
          proposal: "minimal",
        },
      ],
      [
        "recordAndTuple",
        {
          syntaxType: "hash",
        },
      ],
      "throwExpressions",
      "topLevelAwait",
      "v8intrinsic",
    ],
  };
}

function adjustComments(node) {
  const seen = new WeakSet();

  const copy = (from, to, leading, trailing) => {
    from.forEach(comment => {
      if (!seen.has(comment)) {
        comment.leading = leading;
        comment.trailing = trailing;
        to.push(comment);
        seen.add(comment);
      }
    })
  }

  babel.types.traverseFast(node, (node) => {
    const comments = [];
    if (node.leadingComments) {
      copy(node.leadingComments, comments, true, false);
      delete node.leadingComments;
    }
    if (node.innerComments) {
      copy(node.innerComments, comments, false, false);
      delete node.innerComments;
    }
    node.comments = comments;
  });

  babel.types.traverseFast(node, (node) => {
    if (node.trailingComments) {
      copy(node.trailingComments, node.comments, false, true);
      delete node.trailingComments;
    }
  })

  return node;
}

module.exports = {
  parseOptions,
}