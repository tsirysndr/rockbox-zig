import styled from "@emotion/styled";
import Rockbox from "../../Assets/rockbox-icon.png";

const Container = styled.div`
  height: 70px;
  width: 100%;
  padding-left: 5vw;
  padding-right: 5vw;
  display: flex;
  align-items: center;
  margin-top: 20px;
`;

const LogoText = styled.div`
  font-family: RockfordSansBold;
  margin-left: 10px;
`;

const Logo = styled.img`
  height: 40px;
  width: 40px;
`;

const Row = styled.div`
  display: flex;
  flex-direction: row;
  align-items: center;
  height: 70px;
`;

const Header = () => {
  return (
    <Container>
      <Row>
        <Logo src={Rockbox} />
        <LogoText>Rockbox</LogoText>
      </Row>
    </Container>
  );
};

export default Header;
