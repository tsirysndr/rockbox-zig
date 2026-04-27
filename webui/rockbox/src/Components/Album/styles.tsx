import styled from "@emotion/styled";
import { LazyLoadImage } from "react-lazy-load-image-component";
import { Link as RouterLink } from "react-router-dom";

export const AlbumCover = styled(LazyLoadImage)`
  width: 100%;
  border-radius: 3px;
  cursor: pointer;
`;

export const NoAlbumCover = styled.img`
  width: 100%;
  border-radius: 3px;
  cursor: pointer;
`;

export const Artist = styled(RouterLink)`
  color: ${(props) => props.theme.colors.secondaryText};
  font-size: 14px;
  text-overflow: ellipsis;
  overflow: hidden;
  white-space: nowrap;
  cursor: pointer;
  text-decoration: none;
`;

export const Year = styled.div`
  color: ${(props) => props.theme.colors.secondaryText};
  font-size: 12px;
  font-weight: 400;
  margin-bottom: 56px;
`;

export const AlbumTitle = styled.div`
  font-size: 14px;
  text-overflow: ellipsis;
  overflow: hidden;
  white-space: nowrap;
  cursor: pointer;
  color: ${(props) => props.theme.colors.text};
`;

export const CoverWrapper = styled.div`
  position: relative;
  &:hover .album-footer-menu {
    opacity: 1;
    pointer-events: auto;
  }
`;

export const AlbumFooterMenu = styled.div`
  position: absolute;
  bottom: 0;
  left: 10px;
  height: 60px;
  display: flex;
  flex-direction: row;
  align-items: center;
  justify-content: space-between;
  width: calc(100% - 20px);
  opacity: 0;
  pointer-events: none;
  transition: opacity 0.15s;
  z-index: 1;
`;

export const FloatingButton = styled.button`
  height: 40px;
  width: 40px;
  border-radius: 20px;
  display: flex;
  justify-content: center;
  align-items: center;
  border: none;
  cursor: pointer;
  background-color: transparent;

  &:hover {
    background-color: #434242b5;
  }
`;

export const Link = styled(RouterLink)`
  text-decoration: none;
`;
