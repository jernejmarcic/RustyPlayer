# Rustyplayer
 A dead simple (still barely functional) music player written in Rust

## Dependancies
`chafa` for displaying images in the terminal
 terminal compatible with chafa (`kitty`, `konsole`)
`metaflac` for fetching the cover image (and the lyrics in the future)
`rust` obviously



## Features
- [x] Plays music
- [x] Pause
- [x] Next
- [x] Shuffle
- [x] Repeat
- [ ] Previous song (the foundaton exists)
- [x] Lopping
- [ ] MPRIS support
- [x] Album art
- [ ] Audio visualizer

### Planned Features

 1. mpris (d-bus) support
 ~~2. Album images~~ (done)
 3. Better TUI
    - Playlist display
    - Lyrics
    - Adaptive size
    - Progress bar
    - Music visualizer
 4. More (some) CLI options
 5. Better documentation
 6. Previous button

## Limitations
 - It's my first project
 - The code is messy
 - poor UI
 - Too much Chat GPT
 - As of yet the code will only run on linux (due to the dependancies), however without image displaying it could run on Mac and Windows too (in theory)


