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
    }


    func f2(){
        // some comment
        var b = placeholder_false

        guard b else {
            return some_return_value_2
        }

        // some function call
        doSomething()
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
