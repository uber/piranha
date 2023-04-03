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
class TestFooBar{
    func foobar(){
        
    }
}

class C1{
    func f1(){
        
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
        
        if true{
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
        
        
        if true {
            someFunc()
        }
        if false {
            doSomethingElse()
        }
    }
}

class C7 {
    func f7a(){
        
        if true {
            doSomething()
        }
    }

    func f7b(){
        
        
        if true {
            doSomething()
        }
    }
}

class C8{
    

    init(){}

    func f8(){
        
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
    

    init(){}

    func f10(){
        
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
    
    
    func f12(){
        if true {
            doSomething()
        }
        if false {
            doSomethingElse()
        }
    }
}

class C13{
    
    
    func f13(){
        if true {
            doSomething()
        }
        if false {
            doSomethingElse()
        }
    }
}

class C14{
    

    func f14(){
        if true {
            doSomething()
        }
    }
}

class C15{
    

    func f15(){
        if true {
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
    
    init(){
        
        if true {
            doSomething()
        }
    }

    func f20(){
        if true {
            doSomething()
        }
    }
}
