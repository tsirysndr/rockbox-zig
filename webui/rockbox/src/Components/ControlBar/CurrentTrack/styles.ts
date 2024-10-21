import styled from "@emotion/styled";
import { Link } from "react-router-dom";

export const Container = styled.div`
  display: flex;
  flex-direction: row;
  min-width: 400px;
  flex: 1;
  height: 60px;
  border-radius: 4px;
  border: 1px solid rgba(177, 178, 181, 0.25);
  margin-left: 20px;
`;

export const NoCover = styled.div`
  width: 60px;
  height: 60px;
  background-color: #f5f5f5;
  display: flex;
  justify-content: center;
  align-items: center;
  border-top-left-radius: 4px;
  border-bottom-left-radius: 4px;
`;

export const AlbumCover = styled.img`
  height: 60px;
  width: 60px;
  border-top-left-radius: 4px;
  border-bottom-left-radius: 4px;
`;

export const ProgressbarContainer = styled.div`
  width: 100%;
  position: absolute;
  bottom: -12px;
`;

export const TrackInfo = styled.div`
  display: flex;
  flex-direction: column;
  justify-content: center;
  width: 100%;
  position: relative;
  font-size: 14px;
`;

export const Separator = styled.span`
  margin-left: 8px;
  margin-right: 8px;
`;

export const Time = styled.div`
  font-size: 10px;
  color: rgba(0, 0, 0, 0.542);
  font-family: RockfordSansRegular;
  text-align: center;
  width: 60px;
`;

export const ArtistAlbum = styled.div`
  text-align: center;
  color: rgba(0, 0, 0, 0.54);
  font-family: RockfordSansLight;
  text-overflow: ellipsis;
  overflow: hidden;
  white-space: nowrap;
  width: calc(100% - 125px);
`;

export const Title = styled.div`
  text-align: center;
  text-overflow: ellipsis;
  overflow: hidden;
  white-space: nowrap;
  width: calc(100% - 20px);
  margin-left: 10px;
  margin-right: 10px;
`;

export const Actions = styled.div`
  width: 60px;
  display: flex;
  align-items: center;
  justify-content: center;
  margin-right: 5px;
  opacity: 0;
  &:hover {
    opacity: 1;
  }
`;

export const Icon = styled.div`
  cursor: pointer;
`;

export const Album = styled(Link)`
  text-decoration: none;
  color: inherit;
  &:hover {
    text-decoration: underline;
  }
`;

export const styles = {
  Progressbar: {
    BarContainer: {
      style: {
        marginLeft: 0,
        marginRight: 0,
      },
    },
    BarProgress: {
      style: () => ({
        backgroundColor: "#fe099c",
      }),
    },
    Bar: {
      style: () => ({
        backgroundColor: "rgba(177, 178, 181, 0.218)",
      }),
    },
  },
};
