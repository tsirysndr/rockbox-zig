import styled from "@emotion/styled";
import { css } from "@emotion/react";
import { Link } from "react-router-dom";

export const SettingsButton = styled.button`
  display: flex;
  background-color: transparent;
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
  background-color: ${(props) => props.theme.colors.surface};
  padding: 20px;
  ${(props) =>
    props.cover &&
    css`
      background-color: ${props.theme.colors.background};
    `}
`;

export const MenuItem = styled(Link)<{ active?: boolean }>`
  display: flex;
  align-items: center;
  justify-content: flex-start;
  flex-direction: row;
  padding: 10px;
  cursor: pointer;
  font-size: 14px;
  text-decoration: none;
  border-radius: 8px;
  color: ${(props) => props.active ? props.theme.colors.text : props.theme.colors.icon};
  background-color: ${(props) => props.active ? props.theme.colors.hover : "transparent"};
  &:hover {
    background-color: ${(props) => props.theme.colors.hover};
    color: ${(props) => props.theme.colors.text};
  }
`;
