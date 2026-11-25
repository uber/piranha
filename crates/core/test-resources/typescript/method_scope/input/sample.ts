function foo() {
  const x = true;
  if (x) {
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
  const x = true;
  yield x;
}

class Toggles {
  m(flag: boolean) {
    const x = true;
    return x && flag;
  }

  n(flag: boolean) {
    const x = false;
    return x || flag;
  }

  handler = () => {
    const x = true;
    return x;
  };

  otherHandler = () => {
    const x = false;
    return x;
  };
}

const arrow = () => {
  const x = true;
  return x;
};

const otherArrow = () => {
  const x = false;
  return x;
};

const expr = function () {
  const x = true;
  return x;
};
