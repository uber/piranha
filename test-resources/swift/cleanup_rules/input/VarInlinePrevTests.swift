class C1{
    func f1(){
        var a = true
        var b = false
        if a {
            doSomething()
        }
        if b {
            doSomethingElse()
        }
    }
}

class C2{
    func f2(){
        var a = true
        if a {
            a = false
        }
    }
}

class C3{
    var a = true

    func f3(){
        if a {
            doSomething()
        }
    }
}

class C4{
    var a = true

    func f4(){
        a = false
    }

    func f5(){
        if a {
            doSomething()
        }
    }
}

class C5{
    var a

    func f6(){
        a = true
    }

    func f7(){
        a = false
    }
}

class C6{
    var a
    func f8(){
        a = true
    }
}

class C7{
    var a = true

    func f9(){
        var b = a

        if b{
            var c = b
        }
    }
}
