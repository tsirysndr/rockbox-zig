import { BrowserRouter, Routes, Route } from "react-router-dom";
import AlbumsPage from "./Containers/Albums";
import ArtistsPage from "./Containers/Artists/ArtistsPage";
import TracksPage from "./Containers/Tracks";
import AlbumDetails from "./Components/AlbumDetails";
import ArtistDetails from "./Components/ArtistDetails";
import FilesPage from "./Containers/Files";
import LikesPage from "./Containers/Likes";

function App() {
  return (
    <BrowserRouter>
      <Routes>
        <Route path="/" element={<AlbumsPage />} />
        <Route path="/albums" element={<AlbumsPage />} />
        <Route path="/albums/:id" element={<AlbumDetails />} />
        <Route path="/artists" element={<ArtistsPage />} />
        <Route path="/artists/:id" element={<ArtistDetails />} />
        <Route path="/tracks" element={<TracksPage />} />
        <Route path="/files" element={<FilesPage />} />
        <Route path="/likes" element={<LikesPage />} />
      </Routes>
    </BrowserRouter>
  );
}

export default App;
