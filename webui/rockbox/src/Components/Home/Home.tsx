import { FC } from "react";
import { Link } from "react-router-dom";
import { LazyLoadImage } from "react-lazy-load-image-component";
import MainView from "../MainView";
import Sidebar from "../Sidebar";
import ControlBar from "../ControlBar";
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
    <div className="flex flex-row w-full h-full">
      <Sidebar active="home" />
      <MainView>
        <ControlBar />
        <div className="h-[calc(100vh-60px)] overflow-y-auto">
          <div className="text-[28px] font-[RockfordSansMedium] ml-[30px] mt-6 mb-6">Home</div>

          {quickPicks.length > 0 && (
            <div className="grid grid-cols-2 gap-[10px] pl-[30px] pr-[30px] mb-5">
              {quickPicks.map((p) => (
                <Link
                  key={p.id}
                  to={p.isSmart ? `/playlists/smart/${p.id}` : `/playlists/${p.id}`}
                  className="flex flex-row items-center gap-3 px-3 py-2 bg-[var(--theme-cover)] rounded-[4px] no-underline text-inherit cursor-pointer hover:brightness-[1.15]"
                >
                  <div className="w-12 h-12 rounded-[3px] bg-[var(--theme-back-button)] flex items-center justify-center flex-shrink-0 overflow-hidden">
                    {p.image ? (
                      <img
                        src={p.image}
                        alt={p.name}
                        style={{ width: "100%", height: "100%", objectFit: "cover" }}
                      />
                    ) : (
                      <TrackIcon width={22} height={22} color="#a4a3a3" />
                    )}
                  </div>
                  <div className="flex-1 text-[13px] font-semibold overflow-hidden text-ellipsis whitespace-nowrap">
                    {p.name}
                  </div>
                </Link>
              ))}
            </div>
          )}

          {recentlyPlayed.length > 0 && (
            <>
              <div className="text-[20px] font-semibold ml-[30px] mt-[30px] mb-[14px]">Recently played</div>
              <div className="flex flex-row gap-4 pl-[30px] pr-[30px] overflow-x-auto pb-2">
                {recentlyPlayed.map((a) => (
                  <Link
                    key={a.id}
                    to={`/albums/${a.id}`}
                    className="flex-none w-40 no-underline text-inherit cursor-pointer [&:hover_.card-overlay]:opacity-100"
                  >
                    {a.cover ? (
                      <LazyLoadImage
                        src={a.cover}
                        alt={a.title}
                        effect="blur"
                        className="w-40 h-40 rounded-[4px] object-cover"
                      />
                    ) : (
                      <div className="w-40 h-40 rounded-[4px] bg-[var(--theme-cover)] flex items-center justify-center">
                        <TrackIcon width={36} height={36} color="#a4a3a3" />
                      </div>
                    )}
                    <div className="mt-2 text-sm font-medium overflow-hidden text-ellipsis whitespace-nowrap max-w-[160px]">
                      {a.title}
                    </div>
                    <div className="mt-[2px] text-xs text-[var(--theme-secondary-text)] overflow-hidden text-ellipsis whitespace-nowrap max-w-[160px]">
                      {a.artist}
                    </div>
                  </Link>
                ))}
              </div>
            </>
          )}

          {madeForYou.length > 0 && (
            <>
              <div className="text-[20px] font-semibold ml-[30px] mt-[30px] mb-[14px]">Made for you</div>
              <div className="flex flex-row gap-4 pl-[30px] pr-[30px] overflow-x-auto pb-2">
                {madeForYou.map((p) => (
                  <Link
                    key={p.id}
                    to={p.isSmart ? `/playlists/smart/${p.id}` : `/playlists/${p.id}`}
                    className="flex-none w-40 no-underline text-inherit cursor-pointer [&:hover_.card-overlay]:opacity-100"
                  >
                    {p.image ? (
                      <LazyLoadImage
                        src={p.image}
                        alt={p.name}
                        effect="blur"
                        className="w-40 h-40 rounded-[4px] object-cover"
                      />
                    ) : (
                      <div className="w-40 h-40 rounded-[4px] bg-[var(--theme-cover)] flex items-center justify-center">
                        <TrackIcon width={36} height={36} color="#a4a3a3" />
                      </div>
                    )}
                    <div className="mt-2 text-sm font-medium overflow-hidden text-ellipsis whitespace-nowrap max-w-[160px]">
                      {p.name}
                    </div>
                    {p.description ? (
                      <div className="mt-[2px] text-xs text-[var(--theme-secondary-text)] overflow-hidden text-ellipsis whitespace-nowrap max-w-[160px]">
                        {p.description}
                      </div>
                    ) : null}
                  </Link>
                ))}
              </div>
            </>
          )}

          {topArtists.length > 0 && (
            <>
              <div className="text-[20px] font-semibold ml-[30px] mt-[30px] mb-[14px]">Your top artists</div>
              <div className="flex flex-row gap-4 pl-[30px] pr-[30px] overflow-x-auto pb-2">
                {topArtists.map((a) => (
                  <Link
                    key={a.id}
                    to={`/artists/${a.id}`}
                    className="flex-none w-[130px] no-underline text-inherit text-center cursor-pointer"
                  >
                    {a.image ? (
                      <LazyLoadImage
                        src={a.image}
                        alt={a.name}
                        effect="blur"
                        className="w-[130px] h-[130px] rounded-full object-cover"
                      />
                    ) : (
                      <div className="w-[130px] h-[130px] rounded-full bg-[var(--theme-cover)] flex items-center justify-center">
                        <ArtistIcon width={48} height={48} color="#bbb" />
                      </div>
                    )}
                    <div className="mt-2 text-sm font-medium overflow-hidden text-ellipsis whitespace-nowrap max-w-[160px]">
                      {a.name}
                    </div>
                    <div className="mt-[2px] text-xs text-[var(--theme-secondary-text)] overflow-hidden text-ellipsis whitespace-nowrap max-w-[160px]">
                      Artist
                    </div>
                  </Link>
                ))}
              </div>
            </>
          )}

          {popularAlbums.length > 0 && (
            <>
              <div className="text-[20px] font-semibold ml-[30px] mt-[30px] mb-[14px]">Popular albums</div>
              <div className="flex flex-row gap-4 pl-[30px] pr-[30px] overflow-x-auto pb-2" style={{ marginBottom: 80 }}>
                {popularAlbums.map((a) => (
                  <Link
                    key={a.id}
                    to={`/albums/${a.id}`}
                    className="flex-none w-40 no-underline text-inherit cursor-pointer [&:hover_.card-overlay]:opacity-100"
                  >
                    {a.cover ? (
                      <LazyLoadImage
                        src={a.cover}
                        alt={a.title}
                        effect="blur"
                        className="w-40 h-40 rounded-[4px] object-cover"
                      />
                    ) : (
                      <div className="w-40 h-40 rounded-[4px] bg-[var(--theme-cover)] flex items-center justify-center">
                        <TrackIcon width={36} height={36} color="#a4a3a3" />
                      </div>
                    )}
                    <div className="mt-2 text-sm font-medium overflow-hidden text-ellipsis whitespace-nowrap max-w-[160px]">
                      {a.title}
                    </div>
                    <div className="mt-[2px] text-xs text-[var(--theme-secondary-text)] overflow-hidden text-ellipsis whitespace-nowrap max-w-[160px]">
                      {a.artist}
                      {a.year ? ` • ${a.year}` : ""}
                    </div>
                  </Link>
                ))}
              </div>
            </>
          )}

          {isEmpty && (
            <div className="p-[30px] text-[var(--theme-secondary-text)] text-sm">
              Library is empty — wait for the daemon to finish scanning.
            </div>
          )}
        </div>
      </MainView>
    </div>
  );
};

export default Home;
