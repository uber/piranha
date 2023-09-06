class C1 {
    func f1(){
        // some function call
        doSomething() 
        return some_return_value_1
    }


    func f2(){
        return some_return_value_2
    }

    func f3(someParam: String, someOtherParam: SomeType) -> Set<String> {
        guard let abc, case let .caseItem(itemData) = abc else { return [] }
        return []
    }

    func f4(from someVar: SomeType, someOtherVar: String?) -> Set<String> {
        guard let dcx, case let .caseItem(dcx) = dcx else { return [] }
        return []
    }
    
    func f5(vara: String, varb: Int) -> Bool? {
        return nil
    }

    func f6(someVar: SomeType, someOtherVar: SomeOtherType) -> Bool? {
        return nil
    }

    func f7(error: Error?) -> Bool {
        let verifyExceptionCode = "error_verify_someEvent"
        return false
    }

    func f8(forContext someContext: SomeContextType) -> Int? {
        return 5
    }

    func f9()->String?{
        return nil
    }

    func f10()->String?{
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
