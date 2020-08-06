// JS style imports
import "module1";
import "module2";
import "module3";

// CommonJS style imports
const module4 = require("module4");
const module5 = require("module5");

for (var i = 0; i == 0; i++) {
  // Simple flag cleanup in conditional
  if (isFlagTreated(featureFlag)) {
    f1();
  } else {
    f2();
  }
}

// Deep clean of flag
var a = isToggleDisabled(featureFlag)
  ? f1()
  : isToggleDisabled(featureFlag)
  ? f2()
  : isFlagTreated(featureFlag)
  ? f3()
  : f4();

// Assignment cleanup
var b = isToggleDisabled(featureFlag);

// conditional cleanup
let c = isFlagTreated(featureFlag) && b;

// function cleanup
function d() {
  return isFlagTreated(featureFlag);
}

let e = true && d();
