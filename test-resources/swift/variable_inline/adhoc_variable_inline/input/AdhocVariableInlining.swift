class C21{
    var a = true
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

class C22{
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
