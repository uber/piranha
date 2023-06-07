// Copyright (c) 2023 Uber Technologies, Inc.
// 
// <p>Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file
// except in compliance with the License. You may obtain a copy of the License at
// <p>http://www.apache.org/licenses/LICENSE-2.0
// 
// <p>Unless required by applicable law or agreed to in writing, software distributed under the
// License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either
// express or implied. See the License for the specific language governing permissions and
// limitations under the License.

class SampleClass {
    func sampleFunction() {
        isEnabled = v1
        isEnabled = f2()
        isEnabled = v2
        isEnabled = v2
        isEnabled = v2
    }

    func checkOrTrue() {
        //simple
        isEnabled = true
        isEnabled = true
        isEnabled = true

        //nested
        isEnabled = true
        isEnabled = true

        //nested and-or
        isEnabled = v1
        isEnabled = v1
    }
    
    func checkOrFalse() {
        isEnabled = f1()
        isEnabled = v1
        isEnabled = f2()
        isEnabled = v2
    }

    func checkAndFalse() {
        isEnabled = false
        isEnabled = false
        isEnabled = f2() && false
        isEnabled = false
    }
    
    func checkNotCondition() {
        isEnabled = v2
        isEnabled = v2
        isEnabled = v2
    }

    func checkIfTrueCleanup() {
        f1()
        f2()  
        
        if isEnabled {
            f2()
        } else {
            f3()
        }  

        if isEnabled {
            f2()
        } else {
            f3()
        } 

        if isEnabled {
            f2()
        } else if isDisabled {
            f3()
        } else {
            f4()
        } 

        if isEnabled {
            f2()
        } else  {
            f4()
        } 
    }
    
    func checkIfFalse() {
        f2()

        if v1 {
            f2()
        } else {
            f3()
        }

        if v1 {
            f1()
        } else { 
            f3()
        }

        if v1 {
            f1()
        } else if v2 {
            f3()
        } else {
            f4()
        }

        if v1 {
            f1()
        } else { 
        
        }
    }

    func checkIfLetFalse() {
        f2()

        if v1 {
            f2()
        } else {
            f3()
        }

        if v1 {
            f1()
        } else { 
            f3()
        }

        if v1 {
            f1()
        } else if v2 {
            f3()
        } else {
            f4()
        }

        if v1 {
            f1()
        } else { 
        
        }
    }
    
    func checkGaurdTrue() {
        f1()
    }

    func checkGaurdTrueWithAnd() {
        f1()
    }
    func checkGuardFalse() {
        return
    }

    func checkGuardFalseWithAnd() {
        return
    }
    
    func checkTernary() {
        var value = 2
        var value2 = 3
    }

    func checkIfBooleanWithComments(){
        // to be preserved1
        toBePreserved1()
                  
        if a {
            // to be preserved2
            toBePreserved2()
        }

        if b {
            // to be preserved3a
            toBePreserved3a()
        } else {
            // to be preserved3b
            toBePreserved3b()
        }

        // to be preserved4
        toBePreserved4()
                

        // to be preserved5
        toBePreserved5() 
                    
        // to be preserved6
        toBePreserved6()
    }

    func checkIfShortCircuitStatementsWithBooleanPrefix() {
        if  let a1 = something1a{
            // some comment
            // some other comment
            doSomething1a()
            // another comment
            doSomethingElse()
        }

        if  let b1 = something2a(){
            doSomething2a()
        }

        if  c1 == something3a(){
            doSomething3a()
        }

        if  d1 == something4a(){
            doSomething4a()
        }

        if  something5a(){
            doSomething5a()
        }

        if  something6a{
            doSomething6a()
        }
    }

    func checkIfShortCircuitStatementsWithBooleanSuffix() {
        if let a2 = something1{
            // some comment
            // some other comment
            doSomething1b()
            // another comment
            doSomethingElse()
        }

        if let b2 = something2(){
            doSomething2b()
        }

        if c2 == something3(){
            doSomething3b()
        }

        if d2 == something4(){
            doSomething4b()
        }

        if something5a(){
            doSomething5b()
        }

        if something6b{
            doSomething6b()
        }
    }

    func refactorBooleanLiteralEqualityExpressionInIfConditions(){

        toBePreservedEquality1()

        toBePreservedEquality2()

        toBePreservedEquality3() 

        toBePreservedEquality4()

        toBePreservedEquality5()

        toBePreservedEquality6()

        toBePreservedEquality7()

        if a {
        toBePreservedEquality8()
        }

        if b {
        toBePreservedEquality9a()
        } else {
        toBePreservedEquality9b()
        }

        toBePreservedEquality10()

        if c {
            toBePreservedEquality11()
        }

        if d {
            toBePreservedEquality12a()
        } else {
        toBePreservedEquality12b()
        }
    }

    func refactorBooleanLiteralInequalityExpressionInIfConditions(){

        toBePreservedInequality1()

        if a {
            toBePreservedInequality2()
         }

        if b {
            toBePreservedInequality3a()
        } else {
            toBePreservedInequality3b()
        }

        toBePreservedInequality4()

        if c {
            toBePreservedInequality5()
        }

        if d {
            toBePreservedInequality6a()
        } else {
            toBePreservedInequality6b()
        }

        toBePreservedInequality7()

        toBePreservedInequality8() 

        toBePreservedInequality9() 

        toBePreservedInequality10()

        toBePreservedInequality11() 

        toBePreservedInequality12() 
        x = true
    }
}
