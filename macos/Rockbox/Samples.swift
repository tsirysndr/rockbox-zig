//
//  Samples.swift
//  Rockbox
//
//  Created by Tsiry Sandratraina on 14/12/2025.
//


let sampleFiles: [FileItem] = [
    FileItem(name: "Music", type: .directory, size: nil, itemCount: 245),
    FileItem(name: "Downloads", type: .directory, size: nil, itemCount: 18),
    FileItem(name: "Playlists", type: .directory, size: nil, itemCount: 12),
    FileItem(name: "Podcasts", type: .directory, size: nil, itemCount: 34),
    FileItem(name: "Recordings", type: .directory, size: nil, itemCount: 8),
    FileItem(name: "Bohemian Rhapsody.mp3", type: .audioFile, size: "8.2 MB", itemCount: nil),
    FileItem(name: "Hotel California.flac", type: .audioFile, size: "42.1 MB", itemCount: nil),
    FileItem(name: "Stairway to Heaven.mp3", type: .audioFile, size: "12.4 MB", itemCount: nil),
    FileItem(name: "Billie Jean.m4a", type: .audioFile, size: "6.8 MB", itemCount: nil),
    FileItem(name: "Purple Rain.mp3", type: .audioFile, size: "9.1 MB", itemCount: nil),
    FileItem(name: "Smells Like Teen Spirit.mp3", type: .audioFile, size: "7.3 MB", itemCount: nil),
    FileItem(name: "The Chain.flac", type: .audioFile, size: "38.5 MB", itemCount: nil),
    FileItem(name: "Come Together.mp3", type: .audioFile, size: "6.2 MB", itemCount: nil),
]
/*
let sampleAlbums: [Album] = [
    Album(title: "A Night at the Opera", artist: "Queen", year: 1975, color: .purple, tracks: [
        Song(title: "Death on Two Legs", artist: "Queen", album: "A Night at the Opera", duration: 223, color: .purple),
        Song(title: "Lazing on a Sunday Afternoon", artist: "Queen", album: "A Night at the Opera", duration: 67, color: .purple),
        Song(title: "I'm in Love with My Car", artist: "Queen", album: "A Night at the Opera", duration: 186, color: .purple),
        Song(title: "You're My Best Friend", artist: "Queen", album: "A Night at the Opera", duration: 170, color: .purple),
        Song(title: "Bohemian Rhapsody", artist: "Queen", album: "A Night at the Opera", duration: 354, color: .purple),
        Song(title: "Love of My Life", artist: "Queen", album: "A Night at the Opera", duration: 219, color: .purple),
    ]),
    Album(title: "Rumours", artist: "Fleetwood Mac", year: 1977, color: .blue, tracks: [
        Song(title: "Second Hand News", artist: "Fleetwood Mac", album: "Rumours", duration: 163, color: .blue),
        Song(title: "Dreams", artist: "Fleetwood Mac", album: "Rumours", duration: 257, color: .blue),
        Song(title: "Never Going Back Again", artist: "Fleetwood Mac", album: "Rumours", duration: 135, color: .blue),
        Song(title: "Don't Stop", artist: "Fleetwood Mac", album: "Rumours", duration: 190, color: .blue),
        Song(title: "Go Your Own Way", artist: "Fleetwood Mac", album: "Rumours", duration: 222, color: .blue),
        Song(title: "The Chain", artist: "Fleetwood Mac", album: "Rumours", duration: 271, color: .blue),
    ]),
    Album(title: "Back in Black", artist: "AC/DC", year: 1980, color: .black, tracks: [
        Song(title: "Hells Bells", artist: "AC/DC", album: "Back in Black", duration: 312, color: .black),
        Song(title: "Shoot to Thrill", artist: "AC/DC", album: "Back in Black", duration: 317, color: .black),
        Song(title: "Back in Black", artist: "AC/DC", album: "Back in Black", duration: 255, color: .black),
        Song(title: "You Shook Me All Night Long", artist: "AC/DC", album: "Back in Black", duration: 210, color: .black),
        Song(title: "Rock and Roll Ain't Noise Pollution", artist: "AC/DC", album: "Back in Black", duration: 262, color: .black),
    ]),
    Album(title: "Thriller", artist: "Michael Jackson", year: 1982, color: .orange, tracks: [
        Song(title: "Wanna Be Startin' Somethin'", artist: "Michael Jackson", album: "Thriller", duration: 363, color: .orange),
        Song(title: "Baby Be Mine", artist: "Michael Jackson", album: "Thriller", duration: 260, color: .orange),
        Song(title: "Thriller", artist: "Michael Jackson", album: "Thriller", duration: 357, color: .orange),
        Song(title: "Beat It", artist: "Michael Jackson", album: "Thriller", duration: 258, color: .orange),
        Song(title: "Billie Jean", artist: "Michael Jackson", album: "Thriller", duration: 294, color: .orange),
        Song(title: "Human Nature", artist: "Michael Jackson", album: "Thriller", duration: 246, color: .orange),
    ]),
    Album(title: "The Dark Side of the Moon", artist: "Pink Floyd", year: 1973, color: .indigo, tracks: [
        Song(title: "Speak to Me", artist: "Pink Floyd", album: "The Dark Side of the Moon", duration: 68, color: .indigo),
        Song(title: "Breathe", artist: "Pink Floyd", album: "The Dark Side of the Moon", duration: 169, color: .indigo),
        Song(title: "Time", artist: "Pink Floyd", album: "The Dark Side of the Moon", duration: 413, color: .indigo),
        Song(title: "Money", artist: "Pink Floyd", album: "The Dark Side of the Moon", duration: 382, color: .indigo),
        Song(title: "Us and Them", artist: "Pink Floyd", album: "The Dark Side of the Moon", duration: 469, color: .indigo),
    ]),
    Album(title: "Abbey Road", artist: "The Beatles", year: 1969, color: .cyan, tracks: [
        Song(title: "Come Together", artist: "The Beatles", album: "Abbey Road", duration: 259, color: .cyan),
        Song(title: "Something", artist: "The Beatles", album: "Abbey Road", duration: 182, color: .cyan),
        Song(title: "Here Comes the Sun", artist: "The Beatles", album: "Abbey Road", duration: 185, color: .cyan),
        Song(title: "Because", artist: "The Beatles", album: "Abbey Road", duration: 165, color: .cyan),
        Song(title: "Golden Slumbers", artist: "The Beatles", album: "Abbey Road", duration: 91, color: .cyan),
    ]),
    Album(title: "Led Zeppelin IV", artist: "Led Zeppelin", year: 1971, color: .brown, tracks: [
        Song(title: "Black Dog", artist: "Led Zeppelin", album: "Led Zeppelin IV", duration: 296, color: .brown),
        Song(title: "Rock and Roll", artist: "Led Zeppelin", album: "Led Zeppelin IV", duration: 220, color: .brown),
        Song(title: "The Battle of Evermore", artist: "Led Zeppelin", album: "Led Zeppelin IV", duration: 352, color: .brown),
        Song(title: "Stairway to Heaven", artist: "Led Zeppelin", album: "Led Zeppelin IV", duration: 482, color: .brown),
        Song(title: "Misty Mountain Hop", artist: "Led Zeppelin", album: "Led Zeppelin IV", duration: 278, color: .brown),
    ]),
    Album(title: "Hotel California", artist: "Eagles", year: 1976, color: .yellow, tracks: [
        Song(title: "Hotel California", artist: "Eagles", album: "Hotel California", duration: 391, color: .yellow),
        Song(title: "New Kid in Town", artist: "Eagles", album: "Hotel California", duration: 310, color: .yellow),
        Song(title: "Life in the Fast Lane", artist: "Eagles", album: "Hotel California", duration: 291, color: .yellow),
        Song(title: "Victim of Love", artist: "Eagles", album: "Hotel California", duration: 249, color: .yellow),
    ]),
    Album(title: "Born to Run", artist: "Bruce Springsteen", year: 1975, color: .red, tracks: [
        Song(title: "Thunder Road", artist: "Bruce Springsteen", album: "Born to Run", duration: 289, color: .red),
        Song(title: "Tenth Avenue Freeze-Out", artist: "Bruce Springsteen", album: "Born to Run", duration: 191, color: .red),
        Song(title: "Born to Run", artist: "Bruce Springsteen", album: "Born to Run", duration: 270, color: .red),
        Song(title: "Jungleland", artist: "Bruce Springsteen", album: "Born to Run", duration: 594, color: .red),
    ]),
    Album(title: "Purple Rain", artist: "Prince", year: 1984, color: .purple, tracks: [
        Song(title: "Let's Go Crazy", artist: "Prince", album: "Purple Rain", duration: 292, color: .purple),
        Song(title: "Take Me with U", artist: "Prince", album: "Purple Rain", duration: 228, color: .purple),
        Song(title: "When Doves Cry", artist: "Prince", album: "Purple Rain", duration: 358, color: .purple),
        Song(title: "Purple Rain", artist: "Prince", album: "Purple Rain", duration: 520, color: .purple),
    ]),
    Album(title: "The Joshua Tree", artist: "U2", year: 1987, color: .gray, tracks: [
        Song(title: "Where the Streets Have No Name", artist: "U2", album: "The Joshua Tree", duration: 336, color: .gray),
        Song(title: "I Still Haven't Found What I'm Looking For", artist: "U2", album: "The Joshua Tree", duration: 276, color: .gray),
        Song(title: "With or Without You", artist: "U2", album: "The Joshua Tree", duration: 296, color: .gray),
        Song(title: "Bullet the Blue Sky", artist: "U2", album: "The Joshua Tree", duration: 262, color: .gray),
    ]),
    Album(title: "Nevermind", artist: "Nirvana", year: 1991, color: .teal, tracks: [
        Song(title: "Smells Like Teen Spirit", artist: "Nirvana", album: "Nevermind", duration: 301, color: .teal),
        Song(title: "In Bloom", artist: "Nirvana", album: "Nevermind", duration: 255, color: .teal),
        Song(title: "Come as You Are", artist: "Nirvana", album: "Nevermind", duration: 219, color: .teal),
        Song(title: "Lithium", artist: "Nirvana", album: "Nevermind", duration: 257, color: .teal),
        Song(title: "Polly", artist: "Nirvana", album: "Nevermind", duration: 177, color: .teal),
    ]),
]
 */

let sampleArtists: [Artist] = [
    Artist(cuid: "", name: "Queen", genre: "Rock", color: .purple),
    Artist(cuid: "", name: "Fleetwood Mac", genre: "Rock", color: .blue),
    Artist(cuid: "", name: "AC/DC", genre: "Hard Rock", color: .black),
    Artist(cuid: "", name: "Michael Jackson", genre: "Pop", color: .orange),
    Artist(cuid: "", name: "Pink Floyd", genre: "Progressive Rock", color: .indigo),
    Artist(cuid: "", name: "The Beatles", genre: "Rock", color: .cyan),
    Artist(cuid: "", name: "Led Zeppelin", genre: "Hard Rock", color: .brown),
    Artist(cuid: "", name: "Eagles", genre: "Rock", color: .yellow),
    Artist(cuid: "", name: "Bruce Springsteen", genre: "Rock", color: .red),
    Artist(cuid: "", name: "Prince", genre: "Pop", color: .purple),
    Artist(cuid: "", name: "U2", genre: "Rock", color: .gray),
    Artist(cuid: "", name: "Nirvana", genre: "Grunge", color: .teal),
]


let sampleSongs: [Song] = [
    Song(cuid: "", title: "Bohemian Rhapsody", artist: "Queen", album: "A Night at the Opera", albumArt: nil, duration: 354, trackNumber: 1, discNumber: 1, color: .purple),
    Song(cuid: "", title: "You're My Best Friend", artist: "Queen", album: "A Night at the Opera", albumArt: nil, duration: 170, trackNumber: 1, discNumber: 1, color: .purple),
    Song(cuid: "", title: "Love of My Life", artist: "Queen", album: "A Night at the Opera", albumArt: nil, duration: 219, trackNumber: 1, discNumber: 1, color: .purple),
    Song(cuid: "", title: "The Chain", artist: "Fleetwood Mac", album: "Rumours", albumArt: nil, duration: 271,trackNumber: 1, discNumber: 1,  color: .blue),
    Song(cuid: "", title: "Dreams", artist: "Fleetwood Mac", album: "Rumours", albumArt: nil, duration: 257,trackNumber: 1, discNumber: 1,  color: .blue),
    Song(cuid: "", title: "Go Your Own Way", artist: "Fleetwood Mac", album: "Rumours", albumArt: nil, duration: 222, trackNumber: 1, discNumber: 1, color: .blue),
    Song(cuid: "", title: "Back in Black", artist: "AC/DC", album: "Back in Black", albumArt: nil, duration: 255,trackNumber: 1, discNumber: 1,  color: .black),
    Song(cuid: "", title: "Hells Bells", artist: "AC/DC", album: "Back in Black", albumArt: nil, duration: 312,trackNumber: 1, discNumber: 1,  color: .black),
    Song(cuid: "", title: "Thriller", artist: "Michael Jackson", album: "Thriller", albumArt: nil, duration: 357,trackNumber: 1, discNumber: 1,  color: .orange),
    Song(cuid: "", title: "Billie Jean", artist: "Michael Jackson", album: "Thriller", albumArt: nil, duration: 294, trackNumber: 1, discNumber: 1, color: .orange),
    Song(cuid: "", title: "Beat It", artist: "Michael Jackson", album: "Thriller", albumArt: nil, duration: 258,trackNumber: 1, discNumber: 1,  color: .orange),
    Song(cuid: "", title: "Time", artist: "Pink Floyd", album: "The Dark Side of the Moon", albumArt: nil, duration: 413, trackNumber: 1, discNumber: 1,  color: .indigo),
    Song(cuid: "", title: "Money", artist: "Pink Floyd", album: "The Dark Side of the Moon", albumArt: nil, duration: 382, trackNumber: 1, discNumber: 1, color: .indigo),
    Song(cuid: "", title: "Come Together", artist: "The Beatles", album: "Abbey Road", albumArt: nil, duration: 259,trackNumber: 1, discNumber: 1,  color: .cyan),
    Song(cuid: "", title: "Here Comes the Sun", artist: "The Beatles", album: "Abbey Road", albumArt: nil, duration: 185, trackNumber: 1, discNumber: 1, color: .cyan),
    Song(cuid: "", title: "Stairway to Heaven", artist: "Led Zeppelin", album: "Led Zeppelin IV", albumArt: nil, duration: 482,trackNumber: 1, discNumber: 1,  color: .brown),
    Song(cuid: "", title: "Rock and Roll", artist: "Led Zeppelin", album: "Led Zeppelin IV", albumArt: nil, duration: 220, trackNumber: 1, discNumber: 1, color: .brown),
    Song(cuid: "", title: "Hotel California", artist: "Eagles", album: "Hotel California", albumArt: nil, duration: 391, trackNumber: 1, discNumber: 1, color: .yellow),
    Song(cuid: "", title: "Born to Run", artist: "Bruce Springsteen", album: "Born to Run", albumArt: nil, duration: 270, trackNumber: 1, discNumber: 1, color: .red),
    Song(cuid: "", title: "Purple Rain", artist: "Prince", album: "Purple Rain", albumArt: nil, duration: 520,trackNumber: 1, discNumber: 1,  color: .purple),
    Song(cuid: "", title: "With or Without You", artist: "U2", album: "The Joshua Tree", albumArt: nil, duration: 296, trackNumber: 1, discNumber: 1, color: .gray),
    Song(cuid: "", title: "Smells Like Teen Spirit", artist: "Nirvana", album: "Nevermind", albumArt: nil, duration: 301, trackNumber: 1, discNumber: 1, color: .teal),
]
