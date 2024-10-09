import styled from "@emotion/styled";
import { LazyLoadImage } from "react-lazy-load-image-component";

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

export const Title = styled.div`
  font-size: 24px;
  font-family: RockfordSansMedium;
  max-width: 96%;
  margin: auto;
  margin-bottom: 20px;
  padding-left: 20px;
  padding-right: 20px;
`;

export const AlbumCover = styled(LazyLoadImage)`
  width: 100%;
  border-radius: 3px;
  cursor: pointer;
`;

export const NoAlbumCover = styled.div`
  border-radius: 3px;
  cursor: pointer;
  display: flex;
  justify-content: center;
  align-items: center;
  background-color: #ddaefb14;
  // padding: 20px;
  margin-bottom: 5px;
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

export const AlbumTitle = styled.div`
  font-size: 14px;
  text-overflow: ellipsis;
  overflow: hidden;
  white-space: nowrap;
  cursor: pointer;
  color: #000;
`;

export const Scrollable = styled.div`
  height: calc(100vh - 60px);
  overflow-y: auto;
`;

export const FilterContainer = styled.div`
  margin-top: 30px;
  margin-bottom: 40px;
  padding-left: 20px;
  padding-right: 20px;
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
