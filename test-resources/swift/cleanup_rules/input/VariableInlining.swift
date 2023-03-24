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
        var a = true // again, no invocation
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
