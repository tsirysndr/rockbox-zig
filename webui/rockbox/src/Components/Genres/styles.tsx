import styled from "@emotion/styled";
import { Link as RouterLink } from "react-router-dom";

export const Container = styled.div`
  display: flex;
  flex-direction: row;
  width: 100%;
  height: 100%;
`;

export const Scrollable = styled.div`
  height: calc(100vh - 60px);
  overflow-y: auto;
`;

export const Title = styled.div`
  font-size: 24px;
  font-family: RockfordSansMedium;
  max-width: 96%;
  margin: auto;
  margin-bottom: 20px;
  padding-left: 20px;
  padding-right: 20px;
`;

export const Link = styled(RouterLink)`
  text-decoration: none;
  color: inherit;
`;

// Generates a deterministic vivid color for a genre tile from the genre id/name.
export function colorForSeed(seed: string): string {
  let hash = 0;
  for (let i = 0; i < seed.length; i++) {
    hash = (hash * 31 + seed.charCodeAt(i)) >>> 0;
  }
  const hue = hash % 360;
  return `hsl(${hue} 65% 38%)`;
}

export const GenreCard = styled(RouterLink)<{ bg: string }>`
  position: relative;
  display: block;
  border-radius: 8px;
  overflow: hidden;
  height: 110px;
  margin-bottom: 24px;
  text-decoration: none;
  background-color: ${(p) => p.bg};
  color: white;
  cursor: pointer;
  &:hover {
    filter: brightness(1.1);
  }
`;

export const GenreLabel = styled.div`
  position: absolute;
  top: 12px;
  left: 14px;
  right: 14px;
  font-family: RockfordSansMedium;
  font-size: 18px;
  font-weight: 600;
  z-index: 1;
`;

export const GenreCount = styled.div`
  position: absolute;
  bottom: 10px;
  left: 14px;
  font-size: 11px;
  opacity: 0.85;
  z-index: 1;
`;

export const GenreDecoration = styled.div`
  position: absolute;
  right: -16px;
  bottom: -16px;
  font-family: RockfordSansMedium;
  font-size: 36px;
  font-weight: 700;
  opacity: 0.18;
  transform: rotate(-12deg);
  z-index: 0;
`;
