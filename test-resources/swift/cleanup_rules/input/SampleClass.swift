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
        isEnabled = TestEnum.stale_flag_one.isEnabled && v1
        isEnabled = f2() && TestEnum.stale_flag_one.isEnabled 
        isEnabled = v2 && TestEnum.stale_flag_one.isEnabled 
        isEnabled = v2 && (TestEnum.stale_flag_one.isEnabled && true)
        isEnabled = (TestEnum.stale_flag_one.isEnabled && true) && v2
    }

    func checkOrTrue() {
        //simple
        isEnabled = TestEnum.stale_flag_one.isEnabled || v1
        isEnabled = v2 || TestEnum.stale_flag_one.isEnabled 
        isEnabled = f2 || TestEnum.stale_flag_one.isEnabled 

        //nested
        isEnabled = v1 || (TestEnum.stale_flag_one.isEnabled || v2)
        isEnabled = (TestEnum.stale_flag_one.isEnabled || v2) || v1

        //nested and-or
        isEnabled = v1 && (TestEnum.stale_flag_one.isEnabled || v2)
        isEnabled = (TestEnum.stale_flag_one.isEnabled || v2) && v1
    }
    
    func checkOrFalse() {
        isEnabled = placeholder_false || f1()
        isEnabled = placeholder_false || v1
        isEnabled = f2() || placeholder_false 
        isEnabled = v2 || placeholder_false 
    }

    func checkAndFalse() {
        isEnabled = placeholder_false && f1()
        isEnabled = placeholder_false && v1
        isEnabled = f2() && placeholder_false 
        isEnabled = v2 && placeholder_false 
    }
    
    func checkNotCondition() {
        isEnabled = v2 && (TestEnum.stale_flag_one.isEnabled && !false)
        isEnabled = (TestEnum.stale_flag_one.isEnabled && !false) && v2
        isEnabled = v2 || (placeholder_false || !true)
    } 

    func checkIfTrueCleanup() {
        f1()
        if TestEnum.stale_flag_one.isEnabled {
            f2()
        }

        if isEnabled {
            f2()
        } else if TestEnum.stale_flag_one.isEnabled {
            f3()
        } 

        if isEnabled {
            f2()
        } else if TestEnum.stale_flag_one.isEnabled {
            f3()
        } else {
            f4()
        } 

        if isEnabled {
            f2()
        } else if isDisabled {
            f3()
        } else if TestEnum.stale_flag_one.isEnabled {
            f4()
        } else {
            f5()
        }

        if isEnabled {
            f2()
        }  else if TestEnum.stale_flag_one.isEnabled {
            f4()
        } else if isDisabled {
            f3()
        } else {
            f5()
        }
    }
    
    func checkIfFalse() {
        //test comments to be cleaned
        if !TestEnum.stale_flag_one.isEnabled && abc {
            f1()
        }

        if !TestEnum.stale_flag_one.isEnabled && abc {
            f1()
        } else {
            f2()
        }

        if !TestEnum.stale_flag_one.isEnabled && abc {
            f1()
            //test comments to be cleaned
        } else if v1 {
            f2()
        } else {
            f3()
        }

        if v1 {
            f1()
        } else if !TestEnum.stale_flag_one.isEnabled && abc {
            f2()
        } else {
            f3()
        }

        if v1 {
            f1()
        } else if !TestEnum.stale_flag_one.isEnabled && abc {
            f2()
        } else if v2 {
            f3()
        } else {
            f4()
        }

        if v1 {
            f1()
        } else if !TestEnum.stale_flag_one.isEnabled && abc {
            f2()
        }
    }

    func checkIfLetFalse() {
        if let v1 = v1, !TestEnum.stale_flag_one.isEnabled && abc {
            f1()
        }

        if let v1 = v1, !TestEnum.stale_flag_one.isEnabled && abc {
            f1()
        } else {
            f2()
        }

        if let v1 = v1, !TestEnum.stale_flag_one.isEnabled && abc {
            f1()
        } else if v1 {
            f2()
        } else {
            f3()
        }

        if v1 {
            f1()
        } else if let v1 = v1, !TestEnum.stale_flag_one.isEnabled && abc {
            f2()
        } else {
            f3()
        }

        if v1 {
            f1()
        } else if let v1 = v1, !TestEnum.stale_flag_one.isEnabled && abc {
            f2()
        } else if v2 {
            f3()
        } else {
            f4()
        }

        if v1 {
            f1()
        } else if let v1 = v1, !TestEnum.stale_flag_one.isEnabled && abc {
            f2()
        }
    }
    
    func checkGaurdTrue() {
        guard TestEnum.stale_flag_one.isEnabled || f1() else {
            return
        }
        f1()
    }
    
    func checkGaurdTrueWithAnd() {
        guard TestEnum.stale_flag_one.isEnabled && true else {
            return
        }
        f1()
    }
    func checkGuardFalse() {
        guard placeholder_false || false else {
            return
        }
    }

    func checkGuardFalseWithAnd() {
        guard placeholder_false && true else {
            return
        }
    }
    
    func checkTernary() {
        var value = TestEnum.stale_flag_one.isEnabled || v2 ? 2 : 3
        var value2 =  placeholder_false ? 2 : 3
    }

    func checkIfBooleanWithComments(){
         if !TestEnum.stale_flag_one.isEnabled {
            // to be deleted1
            toBeDeleted1()
         }else{
            // to be preserved1
            toBePreserved1()
         }

         if !TestEnum.stale_flag_one.isEnabled{
            // to be deleted2
            toBeDeleted2() 
         } else if a {
            // to be preserved2
            toBePreserved2()
         }

         if !TestEnum.stale_flag_one.isEnabled{
            // to be deleted3
            toBeDeleted3() 
         } else if b {
            // to be preserved3a
            toBePreserved3a()
         } else {
            // to be preserved3b
            toBePreserved3b()
         }

         if TestEnum.stale_flag_one.isEnabled{
            // to be preserved4
            toBePreserved4()
         } else{
            // to be deleted4
            toBeDeleted4()
         }

         if TestEnum.stale_flag_one.isEnabled{
            // to be preserved5
            toBePreserved5() 
         } else if c {
            // to be deleted5
            toBeDeleted5()
         }

         if TestEnum.stale_flag_one.isEnabled{
            // to be preserved6
            toBePreserved6() 
         } else if d {
            // to be deleted6a
            toBeDeleted6a()
         } else {
            // to be deleted6b
            toBeDeleted6b()
         }
    }

    func checkIfShortCircuitStatementsWithBooleanPrefix() {
        if TestEnum.stale_flag_one.isEnabled, let a1 = something1a{
            // some comment
            // some other comment
            doSomething1a()
            // another comment
            doSomethingElse()
        }

        if TestEnum.stale_flag_one.isEnabled, let b1 = something2a(){
            doSomething2a()
        }

        if TestEnum.stale_flag_one.isEnabled, c1 == something3a(){
            doSomething3a()
        }

        if TestEnum.stale_flag_one.isEnabled, d1 == something4a(){
            doSomething4a()
        }

        if TestEnum.stale_flag_one.isEnabled, something5a(){
            doSomething5a()
        }

        if TestEnum.stale_flag_one.isEnabled, something6a{
            doSomething6a()
        }
    }

    func checkIfShortCircuitStatementsWithBooleanSuffix() {
        if let a2 = something1, TestEnum.stale_flag_one.isEnabled{
            // some comment
            // some other comment
            doSomething1b()
            // another comment
            doSomethingElse()
        }

        if let b2 = something2(), TestEnum.stale_flag_one.isEnabled{
            doSomething2b()
        }

        if c2 == something3(), TestEnum.stale_flag_one.isEnabled{
            doSomething3b()
        }

        if d2 == something4(), TestEnum.stale_flag_one.isEnabled{
            doSomething4b()
        }

        if something5a(), TestEnum.stale_flag_one.isEnabled{
            doSomething5b()
        }

        if something6b, TestEnum.stale_flag_one.isEnabled{
            doSomething6b()
        }
    }

    func refactorBooleanLiteralEqualityExpressionInIfConditions(){
         if !TestEnum.stale_flag_one.isEnabled == !TestEnum.stale_flag_one.isEnabled  {
            toBePreservedEquality1()
         }else{
            toBeDeletedEquality1()
         }

         if !TestEnum.stale_flag_one.isEnabled == !TestEnum.stale_flag_one.isEnabled{
            toBePreservedEquality2() 
         } else if a {
            toBeDeletedEquality2()
         }

         if !TestEnum.stale_flag_one.isEnabled == !TestEnum.stale_flag_one.isEnabled{
            toBePreservedEquality3() 
         } else if b {
            toBeDeletedEquality3a()
         } else {
            toBeDeletedEquality3b()
         }

         if TestEnum.stale_flag_one.isEnabled == TestEnum.stale_flag_one.isEnabled{
            toBePreservedEquality4()
         } else{
            toBeDeletedEquality4()
         }

         if TestEnum.stale_flag_one.isEnabled == TestEnum.stale_flag_one.isEnabled{
            toBePreservedEquality5() 
         } else if c {
            toBeDeletedEquality5()
         }

         if TestEnum.stale_flag_one.isEnabled == TestEnum.stale_flag_one.isEnabled{
            toBePreservedEquality6() 
         } else if d {
            toBeDeletedEquality6a()
         } else {
            toBeDeletedEquality6b()
         }

         if !TestEnum.stale_flag_one.isEnabled == TestEnum.stale_flag_one.isEnabled  {
            toBeDeletedEquality7()
         }else{
            toBePreservedEquality7()
         }

         if !TestEnum.stale_flag_one.isEnabled == TestEnum.stale_flag_one.isEnabled{
            toBeDeletedEquality8() 
         } else if a {
            toBePreservedEquality8()
         }

         if !TestEnum.stale_flag_one.isEnabled == TestEnum.stale_flag_one.isEnabled{
            toBeDeletedEquality9() 
         } else if b {
            toBePreservedEquality9a()
         } else {
            toBePreservedEquality9b()
         }

         if TestEnum.stale_flag_one.isEnabled == !TestEnum.stale_flag_one.isEnabled{
            toBeDeletedEquality10()
         } else{
            toBePreservedEquality10()
         }

         if TestEnum.stale_flag_one.isEnabled == !TestEnum.stale_flag_one.isEnabled{
            toBeDeletedEquality11() 
         } else if c {
            toBePreservedEquality11()
         }

         if TestEnum.stale_flag_one.isEnabled == !TestEnum.stale_flag_one.isEnabled{
            toBeDeletedEquality12() 
         } else if d {
            toBePreservedEquality12a()
         } else {
            toBePreservedEquality12b()
         }
    }

    func refactorBooleanLiteralInequalityExpressionInIfConditions(){
         if !TestEnum.stale_flag_one.isEnabled != !TestEnum.stale_flag_one.isEnabled  {
            toBeDeletedInequality1()
         }else{
            toBePreservedInequality1()
         }

         if !TestEnum.stale_flag_one.isEnabled != (!TestEnum.stale_flag_one.isEnabled){
            toBeDeletedInequality2() 
         } else if a {
            toBePreservedInequality2()
         }

         if !TestEnum.stale_flag_one.isEnabled != !TestEnum.stale_flag_one.isEnabled{
            toBeDeletedInequality3() 
         } else if b {
            toBePreservedInequality3a()
         } else {
            toBePreservedInequality3b()
         }

         if (TestEnum.stale_flag_one.isEnabled != TestEnum.stale_flag_one.isEnabled){
            toBeDeletedInequality4()
         } else{
            toBePreservedInequality4()
         }

         if TestEnum.stale_flag_one.isEnabled != TestEnum.stale_flag_one.isEnabled{
            toBeDeletedInequality5() 
         } else if c {
            toBePreservedInequality5()
         }

         if TestEnum.stale_flag_one.isEnabled != TestEnum.stale_flag_one.isEnabled{
            toBeDeletedInequality6() 
         } else if d {
            toBePreservedInequality6a()
         } else {
            toBePreservedInequality6b()
         }

         if !TestEnum.stale_flag_one.isEnabled != TestEnum.stale_flag_one.isEnabled  {
            toBePreservedInequality7()
         }else{
            toBeDeletedInequality7()
         }

         if !TestEnum.stale_flag_one.isEnabled != TestEnum.stale_flag_one.isEnabled{
            toBePreservedInequality8() 
         } else if a {
            toBeDeletedInequality8() 
         }

         if !TestEnum.stale_flag_one.isEnabled != TestEnum.stale_flag_one.isEnabled{
            toBePreservedInequality9() 
         } else if b {
            toBeDeletedInequality9a()
         } else {
            toBeDeletedInequality9b()
         }

         if TestEnum.stale_flag_one.isEnabled != !TestEnum.stale_flag_one.isEnabled{
            toBePreservedInequality10()
         } else{
            toBeDeletedInequality10()
         }

         if TestEnum.stale_flag_one.isEnabled != !TestEnum.stale_flag_one.isEnabled{
            toBePreservedInequality11() 
         } else if c {
            toBeDeletedInequality11()
         }

         if (TestEnum.stale_flag_one.isEnabled) != !TestEnum.stale_flag_one.isEnabled{
            toBePreservedInequality12() 
         } else if d {
            toBeDeletedInequality12a()
         } else {
            toBeDeletedInequality12b()
         }

         x = condition() ? TestEnum.stale_flag_one.isEnabled : true
    }
}
