const b = isToggleDisabled(featureTwo);

// This is another comment
console.log("Old feature oldFeat2 is running");
// This is the second comment
console.log("New feature featureOne is running");

if (!!isToggleDisabled(featureTwo)) {
  // This is the fourth comment
  console.log("New Feature featureTwo is running");
} else {
  // This is the fifth comment
  console.log("Old Feature oldFeat2 is running");
}

if (true) {
  f();
} else {
  g();
}

// This is the sixth comment