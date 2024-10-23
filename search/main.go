package main

import (
	"C"
	"fmt"
	"unsafe"

	"github.com/blevesearch/bleve/v2"
	"github.com/mitchellh/go-homedir"
	"github.com/mitchellh/mapstructure"
	pb "github.com/tsirysndr/rockbox-zig/search/gen/rockbox/search/v1alpha1"
	"google.golang.org/protobuf/proto"
)
import (
	"strings"
	"sync"
)

func main() {
}

func buildBlevePath(name string) string {
	homedir, _ := homedir.Dir()
	rockboxDir := homedir + "/.config/rockbox.org"
	return rockboxDir + "/" + name + ".bleve"
}

func setupBleve(name string) (bleve.Index, error) {
	blevePath := buildBlevePath(name)
	mapping := bleve.NewIndexMapping()
	index, err := bleve.New(blevePath, mapping)

	if err == bleve.ErrorIndexPathExists {
		index, err = bleve.Open(blevePath)
	}

	return index, err
}

//export IndexAlbum
func IndexAlbum(data *C.char, size C.int) {
	index, err := setupBleve("albums")
	if err != nil {
		panic(err)
	}

	messageBytes := C.GoBytes(unsafe.Pointer(data), size)
	var message pb.Album

	if err := proto.Unmarshal(messageBytes, &message); err != nil {
		fmt.Println(err)
		return
	}

	defer index.Close()

	index.Index(message.Id, message)
}

//export IndexArtist
func IndexArtist(data *C.char, size C.int) {
	index, err := setupBleve("artists")
	if err != nil {
		panic(err)
	}

	messageBytes := C.GoBytes(unsafe.Pointer(data), size)
	var message pb.Artist

	if err := proto.Unmarshal(messageBytes, &message); err != nil {
		fmt.Println(err)
		return
	}

	defer index.Close()

	index.Index(message.Id, message)
}

//export IndexFile
func IndexFile(data *C.char, size C.int) {
	index, err := setupBleve("files")
	if err != nil {
		panic(err)
	}

	messageBytes := C.GoBytes(unsafe.Pointer(data), size)
	var message pb.File

	if err := proto.Unmarshal(messageBytes, &message); err != nil {
		fmt.Println(err)
		return
	}

	defer index.Close()

	index.Index(message.Id, message)
}

//export IndexPlaylist
func IndexPlaylist() {
}

//export IndexLikedTrack
func IndexLikedTrack(data *C.char, size C.int) {
	index, err := setupBleve("liked_tracks")
	if err != nil {
		panic(err)
	}

	messageBytes := C.GoBytes(unsafe.Pointer(data), size)
	var message pb.Track

	if err := proto.Unmarshal(messageBytes, &message); err != nil {
		fmt.Println(err)
		return
	}

	defer index.Close()

	index.Index(message.Id, message)
}

//export IndexLikedAlbum
func IndexLikedAlbum(data *C.char, size C.int) {
	index, err := setupBleve("liked_albums")
	if err != nil {
		panic(err)
	}

	messageBytes := C.GoBytes(unsafe.Pointer(data), size)
	var message pb.Album

	if err := proto.Unmarshal(messageBytes, &message); err != nil {
		fmt.Println(err)
		return
	}

	defer index.Close()

	index.Index(message.Id, message)
}

//export IndexTrack
func IndexTrack(data *C.char, size C.int) {
	index, err := setupBleve("tracks")
	if err != nil {
		panic(err)
	}

	messageBytes := C.GoBytes(unsafe.Pointer(data), size)
	var message pb.Track

	if err := proto.Unmarshal(messageBytes, &message); err != nil {
		fmt.Println(err)
		return
	}

	defer index.Close()

	err = index.Index(message.Id, message)
	fmt.Println(err)
}

//export IndexAlbums
func IndexAlbums(data *C.char, size C.int) {
	index, err := setupBleve("albums")
	if err != nil {
		panic(err)
	}

	messageBytes := C.GoBytes(unsafe.Pointer(data), size)
	var message pb.AlbumList

	if err := proto.Unmarshal(messageBytes, &message); err != nil {
		fmt.Println(err)
		return
	}

	var wg sync.WaitGroup
	defer index.Close()

	for _, album := range message.Albums {
		wg.Add(1)
		go func(album *pb.Album) {
			defer wg.Done()
			err := index.Index(album.Id, album)
			if err != nil {
				fmt.Printf("Failed to index album %s: %v\n", album.Id, err)
			}
		}(album)
	}

	wg.Wait()
}

//export IndexArtists
func IndexArtists(data *C.char, size C.int) {
	index, err := setupBleve("artists")
	if err != nil {
		panic(err)
	}

	messageBytes := C.GoBytes(unsafe.Pointer(data), size)
	var message pb.ArtistList

	if err := proto.Unmarshal(messageBytes, &message); err != nil {
		fmt.Println(err)
		return
	}

	var wg sync.WaitGroup
	defer index.Close()

	for _, artist := range message.Artists {
		wg.Add(1)
		go func(artist *pb.Artist) {
			defer wg.Done()
			err := index.Index(artist.Id, artist)
			if err != nil {
				fmt.Printf("Failed to index artist %s: %v\n", artist.Id, err)
			}
		}(artist)
	}

	wg.Wait()
}

//export IndexFiles
func IndexFiles(data *C.char, size C.int) {
	index, err := setupBleve("files")
	if err != nil {
		panic(err)
	}

	messageBytes := C.GoBytes(unsafe.Pointer(data), size)
	var message pb.FileList

	if err := proto.Unmarshal(messageBytes, &message); err != nil {
		fmt.Println(err)
		return
	}

	var wg sync.WaitGroup
	defer index.Close()

	for _, file := range message.Files {
		wg.Add(1)
		go func(file *pb.File) {
			defer wg.Done()
			err := index.Index(file.Id, file)
			if err != nil {
				fmt.Printf("Failed to index file %s: %v\n", file.Id, err)
			}
		}(file)
	}

	wg.Wait()
}

//export IndexPlaylists
func IndexPlaylists() {
}

//export IndexLikedTracks
func IndexLikedTracks(data *C.char, size C.int) {
	index, err := setupBleve("liked_tracks")
	if err != nil {
		panic(err)
	}

	messageBytes := C.GoBytes(unsafe.Pointer(data), size)
	var message pb.LikedTrackList

	if err := proto.Unmarshal(messageBytes, &message); err != nil {
		fmt.Println(err)
		return
	}

	var wg sync.WaitGroup
	defer index.Close()

	for _, track := range message.Tracks {
		wg.Add(1)
		go func(track *pb.LikedTrack) {
			defer wg.Done()
			err := index.Index(track.Id, track)
			if err != nil {
				fmt.Printf("Failed to index track %s: %v\n", track.Id, err)
			}
		}(track)
	}

	wg.Wait()
}

//export IndexLikedAlbums
func IndexLikedAlbums(data *C.char, size C.int) {
	index, err := setupBleve("liked_albums")
	if err != nil {
		panic(err)
	}

	messageBytes := C.GoBytes(unsafe.Pointer(data), size)
	var message pb.LikedAlbumList

	if err := proto.Unmarshal(messageBytes, &message); err != nil {
		fmt.Println(err)
		return
	}

	var wg sync.WaitGroup
	defer index.Close()

	for _, album := range message.Albums {
		wg.Add(1)
		go func(album *pb.LikedAlbum) {
			defer wg.Done()
			err := index.Index(album.Id, album)
			if err != nil {
				fmt.Printf("Failed to index album %s: %v\n", album.Id, err)
			}
		}(album)
	}

	wg.Wait()
}

//export IndexTracks
func IndexTracks(data *C.char, size C.int) {
	index, err := setupBleve("tracks")
	if err != nil {
		panic(err)
	}

	messageBytes := C.GoBytes(unsafe.Pointer(data), size)
	var message pb.TrackList

	if err := proto.Unmarshal(messageBytes, &message); err != nil {
		fmt.Println(err)
		return
	}

	// Use WaitGroup to ensure all goroutines finish before closing the index
	var wg sync.WaitGroup
	defer index.Close()

	for _, track := range message.Tracks {
		wg.Add(1)
		// Launch goroutine to index each track
		go func(track *pb.Track) {
			defer wg.Done()
			err := index.Index(track.Id, track)
			if err != nil {
				fmt.Printf("Failed to index track %s: %v\n", track.Id, err)
			}
		}(track)
	}

	// Wait for all goroutines to finish
	wg.Wait()
}

//export SearchAlbum
func SearchAlbum(term *C.char) *C.char {
	index, err := setupBleve("albums")
	if err != nil {
		panic(err)
	}

	defer index.Close()

	query := bleve.NewPrefixQuery(C.GoString(term))
	searchRequest := bleve.NewSearchRequest(query)

	if strings.Contains(C.GoString(term), " ") {
		query := bleve.NewMatchQuery(C.GoString(term))
		searchRequest = bleve.NewSearchRequest(query)
	}

	searchRequest.Fields = []string{"*"}
	searchResult, _ := index.Search(searchRequest)
	// b, _ := json.MarshalIndent(searchResult, "", " ")
	// fmt.Println(string(b))

	var results []*pb.Album
	for _, hit := range searchResult.Hits {
		var album pb.Album
		_ = mapstructure.Decode(hit.Fields, &album)
		results = append(results, &album)
	}

	response, _ := proto.Marshal(&pb.AlbumList{Albums: results})

	return C.CString(string(response))
}

//export SearchArtist
func SearchArtist(term *C.char) *C.char {
	index, err := setupBleve("artists")
	if err != nil {
		panic(err)
	}

	defer index.Close()

	query := bleve.NewPrefixQuery(C.GoString(term))
	searchRequest := bleve.NewSearchRequest(query)

	if strings.Contains(C.GoString(term), " ") {
		query := bleve.NewMatchQuery(C.GoString(term))
		searchRequest = bleve.NewSearchRequest(query)
	}

	searchRequest.Fields = []string{"*"}
	searchResult, _ := index.Search(searchRequest)
	// b, _ := json.MarshalIndent(searchResult, "", " ")
	// fmt.Println(string(b))

	var results []*pb.Artist
	for _, hit := range searchResult.Hits {
		var artist pb.Artist
		_ = mapstructure.Decode(hit.Fields, &artist)
		results = append(results, &artist)
	}

	response, _ := proto.Marshal(&pb.ArtistList{Artists: results})

	return C.CString(string(response))
}

//export SearchFile
func SearchFile(term *C.char) *C.char {
	index, err := setupBleve("files")
	if err != nil {
		panic(err)
	}

	defer index.Close()

	query := bleve.NewPrefixQuery(C.GoString(term))
	searchRequest := bleve.NewSearchRequest(query)

	if strings.Contains(C.GoString(term), " ") {
		query := bleve.NewMatchQuery(C.GoString(term))
		searchRequest = bleve.NewSearchRequest(query)
	}

	searchRequest.Fields = []string{"*"}
	searchResult, _ := index.Search(searchRequest)

	var results []*pb.File

	for _, hit := range searchResult.Hits {
		var file pb.File
		_ = mapstructure.Decode(hit.Fields, &file)
		results = append(results, &file)
	}

	response, _ := proto.Marshal(&pb.FileList{Files: results})

	return C.CString(string(response))
}

//export SearchTrack
func SearchTrack(term *C.char) *C.char {
	index, err := setupBleve("tracks")
	if err != nil {
		panic(err)
	}

	defer index.Close()

	query := bleve.NewPrefixQuery(C.GoString(term))
	searchRequest := bleve.NewSearchRequest(query)

	if strings.Contains(C.GoString(term), " ") {
		query := bleve.NewMatchQuery(C.GoString(term))
		searchRequest = bleve.NewSearchRequest(query)
	}

	searchRequest.Fields = []string{"*"}
	searchResult, _ := index.Search(searchRequest)

	var results []*pb.Track

	for _, hit := range searchResult.Hits {
		var track pb.Track
		_ = mapstructure.Decode(hit.Fields, &track)
		results = append(results, &track)
	}

	response, _ := proto.Marshal(&pb.TrackList{Tracks: results})

	return C.CString(string(response))
}

//export SearchLikedTrack
func SearchLikedTrack(term *C.char) *C.char {
	index, err := setupBleve("liked_tracks")
	if err != nil {
		panic(err)
	}

	defer index.Close()

	query := bleve.NewPrefixQuery(C.GoString(term))
	searchRequest := bleve.NewSearchRequest(query)

	if strings.Contains(C.GoString(term), " ") {
		query := bleve.NewMatchQuery(C.GoString(term))
		searchRequest = bleve.NewSearchRequest(query)
	}

	searchRequest.Fields = []string{"*"}
	searchResult, _ := index.Search(searchRequest)

	var results []*pb.LikedTrack

	for _, hit := range searchResult.Hits {
		var track pb.LikedTrack
		_ = mapstructure.Decode(hit.Fields, &track)
		results = append(results, &track)
	}

	response, _ := proto.Marshal(&pb.LikedTrackList{Tracks: results})

	return C.CString(string(response))
}

//export SearchLikedAlbum
func SearchLikedAlbum(term *C.char) *C.char {
	index, err := setupBleve("liked_albums")
	if err != nil {
		panic(err)
	}

	defer index.Close()

	query := bleve.NewPrefixQuery(C.GoString(term))
	searchRequest := bleve.NewSearchRequest(query)

	if strings.Contains(C.GoString(term), " ") {
		query := bleve.NewMatchQuery(C.GoString(term))
		searchRequest = bleve.NewSearchRequest(query)
	}

	searchRequest.Fields = []string{"*"}
	searchResult, _ := index.Search(searchRequest)

	var results []*pb.LikedAlbum

	for _, hit := range searchResult.Hits {
		var album pb.LikedAlbum
		_ = mapstructure.Decode(hit.Fields, &album)
		results = append(results, &album)
	}

	response, _ := proto.Marshal(&pb.LikedAlbumList{Albums: results})

	return C.CString(string(response))
}
