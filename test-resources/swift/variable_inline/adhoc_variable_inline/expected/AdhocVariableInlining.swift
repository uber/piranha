// Copyright (c) 2023 Uber Technologies, Inc.

// <p>Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file
// except in compliance with the License. You may obtain a copy of the License at
// <p>http://www.apache.org/licenses/LICENSE-2.0

// <p>Unless required by applicable law or agreed to in writing, software distributed under the
// License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either
// express or implied. See the License for the specific language governing permissions and
// limitations under the License.
class C1{
    var a = true
    func f1(){
        f.subscribe(
            onNext: { x in 
                
            }
        )
    }
}

class C2{
    var a

    init(){
        if something{
            var b = true
            if b {
                a = true
                doSomething()
            }
        } else {
            a = false
        }
    }

    func f2a(){
        if something{
            
            a2 = true
                doSomethingElse()
            
        }
    }
}

class C3 {
    private lazy var someVar: SomeType = {
        let a = false
        let b: SomeContextType = a ? .caseA : .caseB
        let c = SomeNavigationParent.buildComboView(for: b, anotherFor: someOtherVar)
        c.someAttribute = true
        return c
    }()

    private var someComputedVar: SomeVarType {
        let localVar = false
        return shared {
            SomeFunctionCall(a: localVar ? "a" : "b", c: c)
        }
    }

    

    private func f3() -> String{
        abc.subscribe(onNext: {(someVar: SomeVarType) in
        switch someVar {
            case .caseA:
            return "someString"
            case .caseB: 
            return "someOtherString"
            case .caseC:
             return "to_be_retained"
            case .caseD:
             return "to_be_retained"

            default: "another_test_case"
        }})
    }

    private func f4() -> String{
        abc.subscribe(onNext: {(someVar: SomeVarType) in
            if someCondition {
                doSomeCalls()
            } else {
                let someInternalVar = "some_internal_var"
                if someOtherCondition {
                    someFunctionCall()
                    
                }
            }
        })
    }
}

class C4 {
     
    var varA: A 
    var varB: B

    private lazy var someComputedProperty: SomePropertyType {
        let a = SomeFunctionCall(
            firstVar: .a,
            localVar: false,
            secondVar: .b,
            thirdVar: thirdVar
        )
    }

    init(varA: A, varB: B){
        
        self.varA = varA
        self.varB = varB

        someFunctionCall(varB, varA, false)
        
    }

    func ifInSwitch() -> String?{
        switch varA {
            case .a:
                return "a"
            case .b:
                let someNestedVar = "some_nested_var"
                doSOmethingElse()
                
            default:
                return nil
        }
    }
}
