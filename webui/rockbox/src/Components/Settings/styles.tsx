import styled from "@emotion/styled";

export const Container = styled.div`
  display: flex;
  flex-direction: row;
  width: 100%;
  height: 100%;
`;

export const Title = styled.div`
  font-size: 24px;
  font-family: RockfordSansMedium;
  margin: auto;
  margin-bottom: 40px;
`;

export const Scrollable = styled.div`
  height: calc(100vh - 60px);
  overflow-y: auto;
`;

export const Wrapper = styled.div`
  width: 60vw;
  margin: 0 auto;
  margin-bottom: 100px;
  margin-top: 30px;
  min-width: 435px;
  max-width: 800px;
`;
