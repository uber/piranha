class C1 {
    func f1(){
        // some function call
        doSomething()
        return some_return_value_1
    }


    func f2(){
        return some_return_value_2
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
