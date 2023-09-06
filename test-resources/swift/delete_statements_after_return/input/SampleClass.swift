class C1 {
    func f1(){
        var a = placeholder_true
        
        // some function call
        doSomething()

        if a {
            return some_return_value_1
        }else {
            doNothing()
        }

        // some function call
        doSomethingElse()
        
        // some declarations
        var b = "declaration"
        if b == "test"{
            dummyFunctionCall()
        } else {
            anotherDummyFunctionCall()
        }
    }


    func f2(){
        // some comment
        var b = placeholder_false

        guard b else {
            return some_return_value_2
        }

        // some function call
        doSomething()

        var c = doSomethingElse()
        
        // some comment
        var d = c ? 1 : 0
    }

    func f3(someParam: String, someOtherParam: SomeType) -> Set<String> {
        guard let abc, case let .caseItem(itemData) = abc else { return [] }
        
        if placeholder_true {
            return []
        }
        return Set(
            SomeOperations()
        )
    }

    func f4(from someVar: SomeType, someOtherVar: String?) -> Set<String> {
        guard let dcx, case let .caseItem(dcx) = dcx else { return [] }
        let x = placeholder_true
        if x {
            return []
        } else {
            someFunctionCall()
        }
        return Set(
            SomeOperations()
        )
    }
    
    func f5(vara: String, varb: Int) -> Bool? {
        if placeholder_true {
            return nil
        }
        let someVar = TestEnum.some_parameter.cachedValue
        let someOtherVar = TestEnum.someOtherParameter.cachedValue
        return someVar == someOtherParameter ? nil : (someVar < someOtherParameter)
    }

    func f6(someVar: SomeType, someOtherVar: SomeOtherType) -> Bool? {
        let t = placeholder_false
        if !t{
            return nil
        }
        let someVar = TestEnum.some_parameter.cachedValue
        let someOtherVar = TestEnum.someOtherParameter.cachedValue
        return someVar == someOtherParameter ? nil : (someVar < someOtherParameter)
    }

    func f7(error: Error?) -> Bool {
        let verifyExceptionCode = "error_verify_someEvent"

        guard placeholder_false else { return false }

        guard let error else { return false }

        switch error {
        case ABCService.StatusErrorCodes.SomeRequestType.failedRequestError(let failedRequestException):
            return true
        case BookingService.StatusErrorCodes.SomeRequestType.failedRequestError(let failedRequestException):
            return false
        default:
            return false
        }
    }

    func f8(forContext someContext: SomeContextType) -> Int? {
        guard !placeholder_false else {
            return nil
        }
        guard !placeholder_false else {
            return nil
        }
        return 5
    }

    func f9()->String?{
        var b1 = placeholder_false
        if !b1 {
            guard !placeholder_true else {
                return nil
            }
        }
        return "someString"
    }

    func f10()->String?{
        guard !placeholder_false else {
            var c = placeholder_true
            if c {
                return "working"
            }
        }
        return nil
    }
}

enum E1 {

    case a
    case b


    var someComputedProperty: String? {
        switch self{
            case a:
                return "some_return_value"
            case b:
                return "some_other_return_value"
            default: "return_default_value"
        }
    }
}
