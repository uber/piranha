// Copyright (c) 2023 Uber Technologies, Inc.

// <p>Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file
// except in compliance with the License. You may obtain a copy of the License at
// <p>http://www.apache.org/licenses/LICENSE-2.0

// <p>Unless required by applicable law or agreed to in writing, software distributed under the
// License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either
// express or implied. See the License for the specific language governing permissions and
// limitations under the License.

class C1 {

  init() {}

  func f1() {

  }
}

class C2 {
  var a = true

  init() {}

  func f2() {
    a = false
  }
}

class C3 {

  init() {}

  func f3() {

  }
}

class C4 {
  var a = true

  init() {}

  func f4() {
    self.a = false
  }
}

class C5 {

  func f5() {
    doSomething()
  }
}

class C6 {

  func f6() {
    doSomething()
  }
}

class C7 {

  func f7() {
    doSomething()
  }
}

class C8 {

  func f8() {
    doSomething()
  }
}

class C9 {
  var a = true

  func f9a() {
    a = false
  }

  func f9b() {
    if a {
      doSomething()
    }
  }
}

class C10 {
  var a = true

  func f10a() {
    self.a = false
  }

  func f10b() {
    if self.a {
      doSomething()
    }
  }
}

class C11 {
  var a = true

  func f11a() {
    a = true
  }

  func f11b() {
    a = false
  }
}

class C12 {
  var a
  init() {
    a = true
    if a {
      doSomething()
    }
  }

  func f12() {
    a = false
  }
}

class C13 {

  init() {
    doSomething()
  }

  func f13() {
    doSomething()
  }

}

class C14 {

  var a

  init(a: Bool) {

    a = true

    if a {

      doSomething()

    }

  }

  func f14() {

    if a {

      doSomething()

    }

  }

}

class C15 {

  var a

  init() {

    a = true

    if a {

      doSomething()

    }

  }

  func f15(a: Bool) {

    if a {

      doSomething()

    }

  }

}

class C16 {

  init() {
    doSomething()
  }

  func f16() {
    doSomething()

  }

}

class C17 {

  var a

  init(a: Bool) {

    self.a = true

    if a {

      doSomething()

    }

  }

  func f17() {

    if a {

      doSomething()

    }

  }

}

class C18 {

  var a

  init() {

    self.a = true

    if a {

      doSomething()

    }

  }

  func f18(a: Bool) {

    if a {

      doSomething()

    }

  }

}

class C19 {
  var a = true

  init() {
    a = false
  }
}

class C20 {
  var a

  init() {
    a = false
  }

  func f1() {
    var a = true
  }
}

// test for edge from variable_inline_cleanup to boolean_literal_cleanup
class C21 {
  init() {
    super.init(someParameter: someOtherVar)
  }
}

class C22 {
  init() {
    super.init(someParameter: someVar)
  }
}

// test for edge from boolean_literal_cleanup to variable_inline_cleanup
class C23 {
  init() {
    super.init(someParameter: someOtherVar)
  }
}

class C24 {
  init() {
    super.init(someParameter: someVar)
  }
}
