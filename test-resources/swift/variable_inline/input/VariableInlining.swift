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
        var a = "foobar"
    }
}

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
