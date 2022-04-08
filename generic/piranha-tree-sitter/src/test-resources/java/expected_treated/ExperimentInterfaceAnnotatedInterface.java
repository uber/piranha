@ParameterDefinition(namespace="some_long_name") interface SomeParameter { 
  
 @BoolParam(key="other_flag", namespace = "some_long_name")
    BoolParameter isOtherFlag(); 
}