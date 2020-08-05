function get(){
  return isToggleDisabled(featureOne);
}

const a = isToggleDisabled(featureOne);

// This is another comment
console.log("Old feature oldFeat2 is running")

if (false || isFlagTreated(featureOne)) {
  // This is the second comment
  console.log("New feature featureOne is running");
} else {
  // This is the third comment
  console.log("Old feature oldFeat1 is running");
}

// This is the fifth comment
console.log("Old Feature oldFeat2 is running");

if (true) {
  f();
} else {
  g();
}

// This is the sixth comment