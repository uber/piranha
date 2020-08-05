const b = isToggleDisabled(featureTwo);

if (!false && isToggleDisabled(featureTwo)) { 
  // This is the first comment
  console.log("New feature featureTwo is running")
} else {
  // This is another comment
  console.log("Old feature oldFeat2 is running")
}

if (false) {
  // This is the second comment
  console.log("New feature featureOne is running");
} else {
  // This is the third comment
  console.log("Old feature oldFeat1 is running");
}

if (!(!isToggleDisabled(featureTwo))) {
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