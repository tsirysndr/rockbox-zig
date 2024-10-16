import styled from "@emotion/styled";
import { css } from "@emotion/react";

export const Container = styled.div<{ cover?: string }>`
  display: flex;
  flex: 1;
  flex-direction: column;
  position: relative;
  width: calc(100% - 240px);
  background-position: center;
  background-repeat: no-repeat;
  background-size: cover;
  ${({ cover }) => css`
    background-image: url(${cover});
  `}
`;

export const Blur = styled.div`
  background: rgba(256, 256, 256, 0.8);
  backdrop-filter: blur(30px);
  height: 100vh;
`;
