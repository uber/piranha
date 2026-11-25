function Banner() {
  return <div>{true ? "on" : "off"}</div>;
}

const Footer = () => {
  const x = false;
  return <footer>{x ? "on" : "off"}</footer>;
};

class Header extends React.Component {
  render() {
    return <h1>{true ? "on" : "off"}</h1>;
  }
}
