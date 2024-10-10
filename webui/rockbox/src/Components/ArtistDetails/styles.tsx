import styled from "@emotion/styled";
import { LazyLoadImage } from "react-lazy-load-image-component";
import { Link as RouterLink } from "react-router-dom";

export const Container = styled.div`
  display: flex;
  flex-direction: row;
  width: 100%;
  height: 100%;
`;

export const MainView = styled.div`
  display: flex;
  flex: 1;
  flex-direction: column;
`;

export const BackButton = styled.button`
  border: none;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  height: 30px;
  width: 30px;
  border-radius: 15px;
  background-color: #f7f7f8;
  margin-top: 26px;
  margin-bottom: 46px;
  position: absolute;
  z-index: 1;
`;

export const ContentWrapper = styled.div`
  padding-left: 30px;
  padding-right: 30px;
  overflow-y: auto;
  height: calc(100vh - 60px);
`;

export const Name = styled.div`
  font-family: RockfordSansMedium;
  font-size: 30px;
  margin-top: 94px;
  margin-bottom: 40px;
`;

export const ButtonGroup = styled.div`
  display: flex;
  flex-direction: row;
  align-items: center;
`;

export const Separator = styled.div`
  width: 26px;
`;

export const Label = styled.div`
  display: flex;
  flex-direction: row;
  align-items: center;
`;

export const IconButton = styled.button`
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

export const Title = styled.div`
  margin-top: 30px;
  font-size: 20px;
  font-weight: 600;
`;

export const SmallAlbumCover = styled(LazyLoadImage)`
  height: 48px;
  width: 48px;
`;

export const Scrollable = styled.div`
  height: calc(100vh - 60px);
  overflow-y: auto;
`;

export const AlbumTitle = styled(RouterLink)`
  font-size: 14px;
  text-overflow: ellipsis;
  overflow: hidden;
  white-space: nowrap;
  cursor: pointer;
  color: #000;
  text-decoration: none;
`;

export const Artist = styled.div`
  color: #828282;
  font-size: 14px;
  text-overflow: ellipsis;
  overflow: hidden;
  white-space: nowrap;
  cursor: pointer;
`;

export const Year = styled.div`
  color: #828282;
  font-size: 12px;
  font-weight: 400;
  margin-bottom: 56px;
`;

export const AlbumCover = styled(LazyLoadImage)`
  width: 100%;
  border-radius: 3px;
  cursor: pointer;
`;

export const Link = styled(RouterLink)`
  color: #000;
  text-decoration: none;
  font-family: RockfordSansRegular;
  &:hover {
    text-decoration: underline;
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
