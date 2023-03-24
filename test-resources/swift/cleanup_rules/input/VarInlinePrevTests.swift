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
    var a = true

    func f11(){
        a = true
        b = a // should not have deleted this

        if a {
            c = b
        }
    }
}

class C2{
    var a = true

    func f21(){
        b = a

        if a {
            c = b
        }
    }
}

class C3{
    var a = true
    func f31(){
        a = false
        b = a

        if a {
            c = b
        }
    }
}

class C4{    
    func f41(){
        var a = true
        if a { 
            f42()
        }
    }
}

class C5{
	
	var b 

	init(){
		b = true
        f51()
	}

	func f52 (){
		if b {
			f8()
		}
	}
}

class C6{
	
	var a 

	init(){
		a = true
        f1()
	}

	func f61(){
		a = false
	}
}

class C7{
    var a

    init(){
        f71()
    }

    func f1(){
        a = true
    }

    func f72(){
        a = false
    }
}

class C8{
	
	var a 

	init(){
		a = true
        f82(a)
	}

	func f81(){
		if a {
			f82()
		}
	}
}

class C9{
	
	var a 

	init(){
		a = true
        f92(a)
	}

	func f1(){
        var a = false
		if a {
            b = true
		}else{
            f92()
        }
	}
}

class C10{
	
	var a 

	init(){
		a = true
        f102(a)
	}

	func f101(){
        var b //not able to delete this, why?
		if a {
			b = true
		}else{
            f102()
        }
	}
}

class C11{
	
	var a = true

	init(){
		a = true
        f112(a)
	}

	func f111(){
        var b = true
		if a {
			b = true
		}else{
            f112()
        }
	}
}

class C12{
    

    init() {
    }
    
    func f121(a: Bool){
        if a {
            return
        }
    }
}
