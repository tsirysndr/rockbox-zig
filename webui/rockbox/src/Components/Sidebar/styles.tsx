import styled from "@emotion/styled";
import { css } from "@emotion/react";
import { Link } from "react-router-dom";

export const SettingsButton = styled.button`
  display: flex;
  background-color: #f6f9fc;
  border: none;
  cursor: pointer;
  margin-top: 3px;
  height: 64px;
`;

export const Header = styled.div`
  display: flex;
  flex-direction: row;
  align-items: center;
  justify-content: space-between;
`;

export const SidebarContainer = styled.div<{ cover?: string }>`
  display: flex;
  flex-direction: column;
  height: 100vh;
  width: 222px;
  background-color: #f6f9fc;
  padding: 20px;
  ${(props) =>
    props.cover &&
    css`
      background-color: #fff;
    `}
`;

export const MenuItem = styled(Link)<{ color?: string }>`
  display: flex;
  align-items: center;
  justify-content: flex-start;
  flex-direction: row;
  padding: 10px;
  cursor: pointer;
  font-size: 14px;
  text-decoration: none;
  ${({ color }) =>
    color &&
    css`
      color: ${color};
    `}
`;
