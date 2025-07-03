// Copyright (c) 2023 Uber Technologies, Inc.

// <p>Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file
// except in compliance with the License. You may obtain a copy of the License at
// <p>http://www.apache.org/licenses/LICENSE-2.0

// <p>Unless required by applicable law or agreed to in writing, software distributed under the
// License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either
// express or implied. See the License for the specific language governing permissions and
// limitations under the License.

class C1 {
  public var a = placeholder_true

  init() {}

  func f1() {

    a = placeholder_true

  }
}

class C2 {
  var a = placeholder_true

  init() {}

  func f2() {
    a = placeholder_false
  }
}

class C3 {
  open var a = placeholder_true

  init() {}

  func f3() {
    self.a = placeholder_true
  }

}

class C4 {

  fileprivate var a = placeholder_true

  init() {}

  func f4() {
    self.a = placeholder_false
  }
}

class C5 {

  internal var a = placeholder_true

  var b = placeholder_false

  func f5() {

    if a {

      doSomething()

    }

    if b {

      doSomethingElse()

    }

  }

}

class C6 {

  var a = placeholder_true

  var b = placeholder_false

  func f6() {

    if self.a {

      doSomething()

    }

    if self.b {

      doSomethingElse()

    }

  }

}

class C7 {

  public var a = placeholder_true

  func f7() {

    if a {

      doSomething()

    }

  }

}

class C8 {

  var a = placeholder_true

  func f8() {

    if self.a {

      doSomething()

    }

  }

}

class C9 {

  internal var a = placeholder_true

  func f9a() {

    a = placeholder_false

  }

  func f9b() {

    if a {

      doSomething()

    }

  }

}

class C10 {

  open var a = placeholder_true

  func f10a() {

    self.a = placeholder_false

  }

  func f10b() {

    if self.a {

      doSomething()

    }

  }

}

class C11 {

  var a = placeholder_true

  func f11a() {

    a = placeholder_true

  }

  func f11b() {

    a = placeholder_false

  }

}

class C12 {

  fileprivate var a

  init() {

    a = placeholder_true

    if a {

      doSomething()

    }

  }

  func f12() {

    a = placeholder_false

  }

}

class C13 {

  internal var a

  init() {

    a = placeholder_true

    if a {

      doSomething()

    }

  }

  func f13() {

    if a {

      doSomething()

    }

  }

}

class C14 {

  public var a

  init(a: Bool) {

    a = placeholder_true

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

    a = placeholder_true

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

  fileprivate var a

  init() {

    self.a = placeholder_true

    if a {

      doSomething()

    }

  }

  func f16() {

    if a {

      doSomething()

    }

  }

}

class C17 {

  internal var a

  init(a: Bool) {

    self.a = placeholder_true

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

  open var a

  init() {

    self.a = placeholder_true

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
  fileprivate var a = true

  init() {
    a = false
  }
}

class C20 {
  public var a

  init() {
    a = false
  }

  func f1() {
    var a = true
  }
}
