import styled from "@emotion/styled";
import { Spinner } from "baseui/spinner";
import RockboxLogo from "../../Assets/rockbox-logo.svg";

const Container = styled.div`
  background-color: #000;
  height: 100vh;
  width: 100%;
  display: flex;
  justify-content: center;
  align-items: center;
`;

const Loading = styled.div`
  display: flex;
  justify-content: center;
  align-items: center;
  height: 20vh;
  width: 100%;
  position: absolute;
  bottom: 0;
`;

const Logo = styled.img``;

const Splash = () => {
  return (
    <Container>
      <Logo src={RockboxLogo} />
      <Loading>
        <Spinner $color="#fff" />
      </Loading>
    </Container>
  );
};

export default Splash;
