// Simple flag cleanup in conditional
f2();

// ---------------------------------

// Assignment cleanup
f1();

// ----------------------------------

// function cleanup

if(f1()) {
   f1()   
} else {
   f2()   
}

// ----------------------------------

// Complex cleanup
var c = f1();

// Literal conversion
console.log(false);