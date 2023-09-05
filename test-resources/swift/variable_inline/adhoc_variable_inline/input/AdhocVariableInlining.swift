class C1{
    var a = placeholder_true
    func f1(){
        f.subscribe(
            onNext: { x in 
                var b = placeholder_false
                if b {
                    doSomething()
                }
            }
        )
    }
}

class C2{
    var a

    init(){
        if something{
            var b = placeholder_true
            if b {
                a = placeholder_true
                doSomething()
            }
        } else {
            a = false
        }
    }
}

class C3 {
    private lazy var someVar: SomeType = {
        let a = placeholder_false
        let b: VectorDrawable = a ? .caseA : .caseB
        let c = vectorImageComboViewBuilder.buildComboView(for: b, anotherFor: someOtherVar)
        c.someAttribute = true
        return c
    }()

    private var someComputedVar: SomeVarType {
        let localVar = placeholder_false
        return shared {
            SomeFunctionCall(a: localVar ? "a" : "b", c: c)
        }
    }

    private var x = !placeholder_false

    private func f3() -> String{
        abc.subscribe(onNext: {(someVar: SomeVarType) in
        switch someVar {
            case .a:
            return "someString"
            case .b: 
            return "someOtherString"
            case .c:
             if self.x{
                return "to_be_retained"
             }
             return "to_be_deleted"
            case .d:
                if x{
                    return "to_be_retained"
                }
            return "to_be_deleted"
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
                    if self.x {
                        someFunctionCall()
                    } else {
                        someOtherFunctionCall()
                    }
                }
            }
        })
    }
}

class C4 {
    private let localVar: Bool 
    var varA: A 
    var varB: B

    private lazy var someComputedProperty: SomePropertyType {
        let a = SomeFunctionCall(
            firstVar: .a,
            localVar: localVar,
            secondVar: .b,
            thirdVar: thirdVar
        )
    }

    init(varA: A, varB: B){
        self.localVar = placeholder_false
        self.varA = varA
        self.varB = varB

        if localVar {
            someFunctionCall(varA, varB, localVar)
        } else {
            someFunctionCall(varB, varA, localVar)
        }
    }

    func ifInSwitch() -> String?{
        switch varA {
            case .a:
                return "a"
            case .b:
                let someNestedVar = "some_nested_var"
                if localVar {
                    doSomething()
                } else {
                    doSOmethingElse()
                }
            default:
                return nil
        }
    }
}
