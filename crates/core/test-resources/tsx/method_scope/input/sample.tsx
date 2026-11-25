function Banner() {
  const x = true;
  return <div>{x ? "on" : "off"}</div>;
}

const Footer = () => {
  const x = false;
  return <footer>{x ? "on" : "off"}</footer>;
};

class Header extends React.Component {
  render() {
    const x = true;
    return <h1>{x ? "on" : "off"}</h1>;
  }
}
