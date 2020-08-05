// Simple flag cleanup in conditional
if(isFlagTreated(featureFlag)) {
    f1();
 } else {
    f2();
 }
 
 // ---------------------------------
 
 // Assignment cleanup
 var a = isToggleDisabled(featureFlag);
 if(a) {
    f1(); 
 } else {
    f2();
 }
 
 // ----------------------------------
 
 // function cleanup
 function b() {
     return isFlagTreated(featureFlag);
 }
 
 if(b() || f1()) {
    f1()   
 } else {
    f2()   
 }
 
 // ----------------------------------
 
 // Complex cleanup
 var c = isToggleDisabled(featureFlag) ? f1() : isToggleDisabled(featureFlag) ? f2() : isFlagTreated(featureFlag) ? f3() : f4(); 

// Literal conversion
console.log(isFlagTreated(featureFlag));