import styled from "@emotion/styled";

export const Container = styled.div`
  max-height: calc(100vh - 153px);
  padding-top: 15px;
  padding-bottom: 15px;
  overflow-y: auto;
  width: 280px;
  min-height: 120px;
`;

export const List = styled.div`
  max-height: calc(100vh - 273px);
  padding-left: 15px;
  padding-right: 15px;
  overflow-y: auto;
  min-height: 80px;
`;

export const Title = styled.div`
  margin: 10px;
  margin-left: 25px;
  margin-right: 25px;
  font-family: "RockfordSansBold";
`;

export const Placeholder = styled.div`
  display: flex;
  align-items: center;
  justify-content: center;
  height: 120px;
  text-align: center;
  padding-left: 20px;
  padding-right: 20px;
  font-size: 14px;
`;
