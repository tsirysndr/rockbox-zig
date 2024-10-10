import { FC } from "react";
import { ProgressBar } from "baseui/progress-bar";
import {
  AlbumCover,
  ArtistAlbum,
  Container,
  NoCover,
  ProgressbarContainer,
  Separator,
  styles,
  Time,
  Title,
  TrackInfo,
} from "./styles";
import Track from "../../Icons/Track";
import { useTimeFormat } from "../../../Hooks/useFormat";

export type CurrentTrackProps = {
  nowPlaying?: {
    album?: string;
    artist?: string;
    title?: string;
    cover?: string;
    duration: number;
    progress: number;
    isPlaying?: boolean;
    albumId?: string;
  };
};

const CurrentTrack: FC<CurrentTrackProps> = ({ nowPlaying }) => {
  const { formatTime } = useTimeFormat();
  return (
    <Container>
      {!nowPlaying?.cover && (
        <NoCover>
          <Track color="#b1b2b5" height={28} width={28} />
        </NoCover>
      )}
      {nowPlaying?.cover && (
        <AlbumCover src={nowPlaying.cover} alt="Album cover" />
      )}
      <TrackInfo>
        {!nowPlaying && (
          <div style={{ color: "#b1b2b5", textAlign: "center" }}>
            No track playing
          </div>
        )}
        {nowPlaying && (
          <>
            <Title>{nowPlaying.title}</Title>
            <div
              style={{
                display: "flex",
                flexDirection: "row",
                alignItems: "center",
                justifyContent: "space-between",
              }}
            >
              <Time>{formatTime(nowPlaying.progress)}</Time>
              <ArtistAlbum>
                {nowPlaying.artist}
                <Separator>-</Separator>
                {nowPlaying.album}
              </ArtistAlbum>
              <Time>{formatTime(nowPlaying.duration)}</Time>
            </div>
            <ProgressbarContainer>
              <ProgressBar
                value={
                  nowPlaying!.duration > 0
                    ? (nowPlaying!.progress / nowPlaying!.duration) * 100
                    : 0
                }
                overrides={styles.Progressbar}
              />
            </ProgressbarContainer>
          </>
        )}
      </TrackInfo>
    </Container>
  );
};

export default CurrentTrack;
