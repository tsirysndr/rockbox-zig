import { FC } from "react";
import MainView from "../MainView";
import Sidebar from "../Sidebar";
import ControlBar from "../ControlBar";
import {
  ArtistCard,
  ArtistImage,
  ArtistImagePlaceholder,
  Card,
  CardSubtitle,
  CardTitle,
  Container,
  Cover,
  CoverPlaceholder,
  Empty,
  QuickPickCard,
  QuickPickName,
  QuickPickThumb,
  QuickPicksGrid,
  Row,
  Scrollable,
  SectionTitle,
  Title,
} from "./styles";
import ArtistIcon from "../Icons/Artist";
import TrackIcon from "../Icons/Track";

export type HomeAlbum = {
  id: string;
  title: string;
  artist: string;
  artistId: string;
  cover?: string;
  year?: number | null;
};

export type HomeArtist = {
  id: string;
  name: string;
  image?: string | null;
};

export type HomePlaylist = {
  id: string;
  name: string;
  description?: string | null;
  image?: string | null;
  isSmart: boolean;
};

export type HomeProps = {
  recentlyPlayed: HomeAlbum[];
  topArtists: HomeArtist[];
  popularAlbums: HomeAlbum[];
  madeForYou: HomePlaylist[];
  quickPicks: HomePlaylist[];
  loading?: boolean;
};

const Home: FC<HomeProps> = ({
  recentlyPlayed,
  topArtists,
  popularAlbums,
  madeForYou,
  quickPicks,
  loading,
}) => {
  const isEmpty =
    !loading &&
    recentlyPlayed.length === 0 &&
    topArtists.length === 0 &&
    popularAlbums.length === 0 &&
    madeForYou.length === 0 &&
    quickPicks.length === 0;

  return (
    <Container>
      <Sidebar active="home" />
      <MainView>
        <ControlBar />
        <Scrollable>
          <Title>Home</Title>

          {quickPicks.length > 0 && (
            <QuickPicksGrid>
              {quickPicks.map((p) => (
                <QuickPickCard
                  key={p.id}
                  to={p.isSmart ? `/playlists/smart/${p.id}` : `/playlists/${p.id}`}
                >
                  <QuickPickThumb>
                    {p.image ? (
                      <img
                        src={p.image}
                        alt={p.name}
                        style={{ width: "100%", height: "100%", objectFit: "cover" }}
                      />
                    ) : (
                      <TrackIcon width={22} height={22} color="#a4a3a3" />
                    )}
                  </QuickPickThumb>
                  <QuickPickName>{p.name}</QuickPickName>
                </QuickPickCard>
              ))}
            </QuickPicksGrid>
          )}

          {recentlyPlayed.length > 0 && (
            <>
              <SectionTitle>Recently played</SectionTitle>
              <Row>
                {recentlyPlayed.map((a) => (
                  <Card key={a.id} to={`/albums/${a.id}`}>
                    {a.cover ? (
                      <Cover src={a.cover} alt={a.title} effect="blur" />
                    ) : (
                      <CoverPlaceholder>
                        <TrackIcon width={36} height={36} color="#a4a3a3" />
                      </CoverPlaceholder>
                    )}
                    <CardTitle>{a.title}</CardTitle>
                    <CardSubtitle>{a.artist}</CardSubtitle>
                  </Card>
                ))}
              </Row>
            </>
          )}

          {madeForYou.length > 0 && (
            <>
              <SectionTitle>Made for you</SectionTitle>
              <Row>
                {madeForYou.map((p) => (
                  <Card
                    key={p.id}
                    to={p.isSmart ? `/playlists/smart/${p.id}` : `/playlists/${p.id}`}
                  >
                    {p.image ? (
                      <Cover src={p.image} alt={p.name} effect="blur" />
                    ) : (
                      <CoverPlaceholder>
                        <TrackIcon width={36} height={36} color="#a4a3a3" />
                      </CoverPlaceholder>
                    )}
                    <CardTitle>{p.name}</CardTitle>
                    {p.description ? (
                      <CardSubtitle>{p.description}</CardSubtitle>
                    ) : null}
                  </Card>
                ))}
              </Row>
            </>
          )}

          {topArtists.length > 0 && (
            <>
              <SectionTitle>Your top artists</SectionTitle>
              <Row>
                {topArtists.map((a) => (
                  <ArtistCard key={a.id} to={`/artists/${a.id}`}>
                    {a.image ? (
                      <ArtistImage src={a.image} alt={a.name} effect="blur" />
                    ) : (
                      <ArtistImagePlaceholder>
                        <ArtistIcon width={48} height={48} color="#bbb" />
                      </ArtistImagePlaceholder>
                    )}
                    <CardTitle>{a.name}</CardTitle>
                    <CardSubtitle>Artist</CardSubtitle>
                  </ArtistCard>
                ))}
              </Row>
            </>
          )}

          {popularAlbums.length > 0 && (
            <>
              <SectionTitle>Popular albums</SectionTitle>
              <Row style={{ marginBottom: 80 }}>
                {popularAlbums.map((a) => (
                  <Card key={a.id} to={`/albums/${a.id}`}>
                    {a.cover ? (
                      <Cover src={a.cover} alt={a.title} effect="blur" />
                    ) : (
                      <CoverPlaceholder>
                        <TrackIcon width={36} height={36} color="#a4a3a3" />
                      </CoverPlaceholder>
                    )}
                    <CardTitle>{a.title}</CardTitle>
                    <CardSubtitle>
                      {a.artist}
                      {a.year ? ` • ${a.year}` : ""}
                    </CardSubtitle>
                  </Card>
                ))}
              </Row>
            </>
          )}

          {isEmpty && (
            <Empty>
              Library is empty — wait for the daemon to finish scanning.
            </Empty>
          )}
        </Scrollable>
      </MainView>
    </Container>
  );
};

export default Home;
