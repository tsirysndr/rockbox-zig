import styled from '@emotion/styled';
import { css } from '@emotion/react';

export const SidebarContainer = styled.div`
  display: flex;
  flex-direction: column;
  height: 100vh;
  width: 222px;
  background-color: #f6f9fc;
  padding: 20px;
`;

export const MenuItem = styled.a<{ color?: string }>`
  display: flex;
  align-items: center;
  justify-content: flex-start;
  flex-direction: row;
  padding: 10px;
  cursor: pointer;
  font-size: 14px;
  text-decoration: none;
  ${({ color }) => color && css`
    color: ${color};
  `}
`;