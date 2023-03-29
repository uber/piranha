// // // Copyright (c) 2022 Uber Technologies, Inc.
// // // 
// // // <p>Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file
// // // except in compliance with the License. You may obtain a copy of the License at
// // // <p>http://www.apache.org/licenses/LICENSE-2.0
// // // 
// // // <p>Unless required by applicable law or agreed to in writing, software distributed under the
// // // License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either
// // // express or implied. See the License for the specific language governing permissions and
// // // limitations under the License.

class C21{
    func f211(){
        var a = true
    }
}

class C22{
    func f222(){
        var a = true
        a = false
    }
}

class C23{
    func f233(){
        var a = true
        if a{
         f234()
        }
    }
}

class C24{
    func f243(a: Bool){
        if a{
            f244()
        }
    }
}

class C25{
    var a = true
    func f254(){
        a = true
    }
}

class C26{
    var a = true
    func f254(){
        self.a = true
    }
}



class C11{
    var a = true
    var b = false
    func f11(){
        if a {
            doSomething()
        }
        if b {
            doSomethingElse()
        }
    }
}

class C12{
    var a = true
    var b = false
    func f12(){
        if self.a {
            doSomething()
        }
        if self.b {
            doSomethingElse()
        }
    }
}

class C1{
    func fa1(){
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

class C31{
    var a = true

    func f31(){
        if self.a {
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

class C41{
    var a = true

    func f41(){
        self.a = false
    }

    func f51(){
        if self.a {
            doSomething()
        }
    }
}

class C5{
    var a = true

    func f6(){
        a = true
    }

    func f7(){
        a = false
    }
}

class C6{
    var a
    init(){
        a = true
        if a {
            doSomething()
        }
    }
}

class C61{
    var a
    init(){
        a = true
        if a {
            doSomething()
        }
    }

    func f61(){
        a = false
    }
}

class C62{
    var a
    init(){
        a = true
        if a {
            doSomething()
        }
    }

    func f62(){
        if a {
            doSomething()
        }
    }
}

class C61{
    var a = true
    init(){
        self.a = true
    }
}

class C71{
    var a

    init(){
        a = true

        if a {
            var b = false
        }
    }
}

class C7{
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
