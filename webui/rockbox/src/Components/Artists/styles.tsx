import styled from "@emotion/styled";

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
  margin: auto;
  margin-bottom: 20px;
  padding-left: 20px;
  padding-right: 20px;
`;

export const Scrollable = styled.div`
  height: calc(100vh - 60px);
  overflow-y: auto;
`;

export const Wrapper = styled.div`
  margin-top: 34px;
`;

export const ArtistCover = styled.img`
  width: 194px;
  height: 194px;
  border-radius: 97px;
  cursor: pointer;
`;

export const NoArtistCover = styled.div`
  width: 194px;
  height: 194px;
  border-radius: 97px;
  cursor: pointer;
  display: flex;
  justify-content: center;
  align-items: center;
  background-color: #f3f3f3b9;
`;

export const ArtistName = styled.div`
  font-size: 14px;
  text-overflow: ellipsis;
  overflow: hidden;
  white-space: nowrap;
  cursor: pointer;
  margin-top: 20px;
  margin-bottom: 18px;
  text-align: center;
  width: 194px;
  color: #000;
`;
