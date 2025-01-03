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
  color: #828282;
  font-size: 14px;
  text-overflow: ellipsis;
  overflow: hidden;
  white-space: nowrap;
  cursor: pointer;
  text-decoration: none;
`;

export const Year = styled.div`
  color: #828282;
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
  color: #000;
`;

export const AlbumFooterMenu = styled.div`
  position: absolute;
  bottom: 60px;
  left: 10px;
  height: 60px;
  display: flex;
  flex-direction: row;
  align-items: center;
  justify-content: space-between;
  width: calc(100% - 20px);
  z-index: 1;
`;

export const Hover = styled.button`
  color: transparent;
  background-color: transparent;
  border: none;
  opacity: 0 !important;
  cursor: pointer;
  &:hover,
  &:focus {
    color: #000;
    opacity: 1 !important;
  }
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
