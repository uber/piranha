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

        b = true

        if true {
            c = b
        }
    }
}

class C2{

    func f2(){

        b = true

        if true {
            c = b
        }
    }
}

class C3{
    var a = true
    func f3(){
        a = false
        b = a

        if a {
            c = b
        }
    }
}

class C4 {
	
	func f4(){
		if true{
			f5()
		}
	}
}

class C5{

        init(){
            f2()
        }

        func f1(){
            if true {
                f2()
            }
        }
}

class C6{
	
	var a 

	init(){
		a = true
        f1()
	}

	func f1(){
		a = false
	}
}

class C7{
    var a

    init(){
        f1()
    }

    func f1(){
        a = true
    }

    func f2(){
        a = false
    }
}

class C8{

	init(){
        f2(true)
	}

	func f1(){
		if true {
			f2()
		}
	}
}

class C9{
	
	var a 

	init(){
		a = true
        f2(a)
	}

	func f1(){
        var a = false
		if a {
			b = true
		}else{
            f2()
        }
	}
}

class C10{
	

	init(){
        f2(true)
	}

	func f1(){
        var b 
		if true {
            b = true
		}else{
            f2()
        }
	}
}

class C11{

    init(){
        f2(true)
    }

    func f1(){
        if true {

        }else{
            f2()
        }
    }
}

class C12{
    

    init() {
    }
    
    func f1(a: Bool){
        if a {
            return
        }
    }
}
