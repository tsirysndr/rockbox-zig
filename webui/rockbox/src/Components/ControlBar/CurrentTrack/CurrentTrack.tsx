import { FC } from "react";
import { ProgressBar } from "baseui/progress-bar";
import {
  Album,
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
import { CurrentTrack as NowPlaying } from "../../../Types/track";
import _ from "lodash";

export type CurrentTrackProps = {
  nowPlaying?: NowPlaying;
};

const CurrentTrack: FC<CurrentTrackProps> = ({ nowPlaying }) => {
  const { formatTime } = useTimeFormat();
  const album = `${nowPlaying?.artist} - ${nowPlaying?.album}`;
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
        {(!nowPlaying || nowPlaying?.duration === 0) && (
          <div style={{ color: "#b1b2b5", textAlign: "center" }}>
            No track playing
          </div>
        )}
        {nowPlaying && nowPlaying?.duration > 0 && (
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
                {_.get(nowPlaying, "artist.length", 0) > 75
                  ? `${nowPlaying.artist?.substring(0, 54)}...`
                  : nowPlaying.artist}
                <Separator>-</Separator>
                <Album to={`/albums/${nowPlaying.albumId}`}>
                  {album.length > 75
                    ? `${nowPlaying.album?.substring(0, 30)}...`
                    : nowPlaying.album}
                </Album>
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
