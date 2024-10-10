import styled from "@emotion/styled";

export const Container = styled.div`
  display: flex;
  flex-direction: row;
  height: 60px;
  margin-top: 5px;
  margin-bottom: 20px;
`;

export const ControlsContainer = styled.div`
  display: flex;
  align-items: center;
  justify-content: space-between;
  flex-direction: row;
  width: 160px;
`;

export const Button = styled.button`
  background-color: transparent;
  cursor: pointer;
  border: none;
  display: flex;
  align-items: center;
  justify-content: center;
  &:hover {
    opacity: 0.6;
  }
`;

export const Controls = styled.div`
  display: flex;
  align-items: center;
  justify-content: center;
  flex: 0.3;
`;
