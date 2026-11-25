function foo() {
  if (true) {
    console.log("foo on");
  }
}

function bar() {
  const x = false;
  if (x) {
    console.log("bar on");
  }
}

function* gen() {
  yield true;
}

class Toggles {
  m(flag: boolean) {
    return true && flag;
  }

  n(flag: boolean) {
    const x = false;
    return x || flag;
  }

  handler = () => {
    return true;
  };

  otherHandler = () => {
    const x = false;
    return x;
  };
}

const arrow = () => {
  return true;
};

const otherArrow = () => {
  const x = false;
  return x;
};

const expr = function () {
  return true;
};

function pick(kind: "a" | "b", separator = "\\") {
  return true ? kind : separator;
}
