// Copyright (c) 2022 Uber Technologies, Inc.
// 
// <p>Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file
// except in compliance with the License. You may obtain a copy of the License at
// <p>http://www.apache.org/licenses/LICENSE-2.0
// 
// <p>Unless required by applicable law or agreed to in writing, software distributed under the
// License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either
// express or implied. See the License for the specific language governing permissions and
// limitations under the License.

class C1{
    func f1(){
        var a = true
    }
}

class C2{
    func f2(){
        var a = true
        a = false
    }
}

class C3{
    func f3(){
        var a = true
        if a{
         f234()
        }
    }
}

class C4{
    func f4(a: Bool){
        if a{
            f244()
        }
    }
}

class C5{
    func f5(){
        var a = true
        if a {
            a = false
        }
    }
}

class C6 {
    func f6(){
        var a = true
        var b = false
        if a {
            someFunc()
        }
        if b {
            doSomethingElse()
        }
    }
}

class C7 {
    func f7a(){
        var a = true
        if a {
            doSomething()
        }
    }

    func f7b(){
        var a = true
        var b = true
        if b {
            doSomething()
        }
    }
}

class C8{
    var a = true

    init(){}

    func f8(){
        a = true
    }
}

class C9{
    var a = true

    init(){}

    func f9(){
        a = false
    }
}

class C10{
    var a = true

    init(){}

    func f10(){
        self.a = true
    }
}

class C11{
    var a = true

    init(){}

    func f11(){
        self.a = false
    }
}

class C12{
    var a = true
    var b = false
    func f12(){
        if a {
            doSomething()
        }
        if b {
            doSomethingElse()
        }
    }
}

class C13{
    var a = true
    var b = false
    func f13(){
        if self.a {
            doSomething()
        }
        if self.b {
            doSomethingElse()
        }
    }
}

class C14{
    var a = true

    func f14(){
        if a {
            doSomething()
        }
    }
}

class C15{
    var a = true

    func f15(){
        if self.a {
            doSomething()
        }
    }
}

class C16{
    var a = true

    func f4(){
        a = false
    }

    func f16(){
        if a {
            doSomething()
        }
    }
}

class C17{
    var a = true

    func f17a(){
        self.a = false
    }

    func f17b(){
        if self.a {
            doSomething()
        }
    }
}

class C18{
    var a = true

    func f18a(){
        a = true
    }

    func f18b(){
        a = false
    }
}

class C19{
    var a
    init(){
        a = true
        if a {
            doSomething()
        }
    }

    func f19(){
        a = false
    }
}

class C20{
    var a
    init(){
        a = true
        if a {
            doSomething()
        }
    }

    func f20(){
        if a {
            doSomething()
        }
    }
}

class C21{
    var a = true
    func f1(){
        f.subscribe(
            onNext: { x in 
                var b = false
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
            var b = true
            if b {
                a = true
                doSomething()
            }
        } else {
            a = false
        }
    }
}
