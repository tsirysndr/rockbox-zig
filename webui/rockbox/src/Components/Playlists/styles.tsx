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

export const SectionTitle = styled.div`
  font-size: 16px;
  font-family: RockfordSansMedium;
  padding-left: 20px;
  padding-right: 20px;
  margin-bottom: 16px;
  margin-top: 24px;
  color: ${(props) => props.theme.colors.secondaryText};
`;

export const PlaylistGrid = styled.div`
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(160px, 1fr));
  gap: 20px;
  padding-left: 20px;
  padding-right: 20px;
  margin-bottom: 40px;
`;

export const PlaylistCard = styled.div`
  position: relative;
  cursor: pointer;

  &:hover .card-actions {
    opacity: 1;
  }
`;

export const PlaylistCover = styled.div<{ image?: string }>`
  width: 100%;
  aspect-ratio: 1;
  border-radius: 6px;
  background-color: ${(props) => props.theme.colors.cover};
  background-image: ${({ image }) => (image ? `url(${image})` : "none")};
  background-size: cover;
  background-position: center;
  display: flex;
  align-items: center;
  justify-content: center;
  overflow: hidden;
`;

export const PlaylistName = styled.div`
  font-size: 14px;
  font-family: RockfordSansMedium;
  margin-top: 8px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  color: ${(props) => props.theme.colors.text};
`;

export const PlaylistMeta = styled.div`
  font-size: 12px;
  color: ${(props) => props.theme.colors.secondaryText};
  margin-top: 2px;
`;

export const CardActions = styled.div`
  position: absolute;
  bottom: 48px;
  left: 8px;
  right: 8px;
  display: flex;
  flex-direction: row;
  align-items: center;
  justify-content: space-between;
  opacity: 0;
  transition: opacity 0.15s;
  padding-bottom: 6px;
`;

export const CardAction = styled.button`
  height: 36px;
  width: 36px;
  border-radius: 18px;
  border: none;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  background-color: ${(props) => props.theme.colors.surface};
  backdrop-filter: blur(4px);

  &:hover {
    background-color: ${(props) => props.theme.colors.hover};
  }
`;

export const Link = styled(RouterLink)`
  text-decoration: none;
`;
