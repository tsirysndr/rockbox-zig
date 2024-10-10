import styled from "@emotion/styled";

export const Container = styled.div`
  display: flex;
  flex-direction: row;
  flex: 0.3;
  height: 50px;
  min-width: 160px;
  align-items: center;
  justify-content: flex-end;
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
