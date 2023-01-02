/*
Planning - Split into subsections for each component of the architecture. Single indented are necessary features for the application to function, double or more are for unneccessary features or improvements

Make skeleton program
    TODO: Align widgets to proper locations
    Create pause play widget
    TODO: Fast forward/backward widgets
    TODO: Implement audio

State:
    Current directory (None if nothing is currently selected)
    Store index of mp3 files, lazy load file currently playing and the two in front of and behind it using said index
    Current file playing (None if nothing is selected)
    Current state (enum) that stores whether song is paused or playing, or loading a new song

Messages:
    Pause/play event
        TODO: Next/prev track
    Folder picker
        Emacs dired style folder picker (replaces above)

View Logic:
    Pause, play button
        TODO: Next, prev track button
    File input box
        Show list of tracks in folder -> track selector
    TODO: Shuffle play

Update Logic:
    FIXME: Change current directory when folder is opened
    Selecting a track, or selecting next/prev track updates current state enum, current file playing

    Persistence:
        TODO: Save current directory on program exit, load on program start. (optional)
            TODO: Save current track on program exit, load on program start. (optional) (depends on above)
*/

use iced::event::{self, Event};
use iced::keyboard;
use iced::subscription;
use iced::theme::Theme;
use iced::widget::{
    self, button, column, container, row, scrollable, text, text_input, svg
};

use iced::{Application, Element};
use iced::{Command, Settings, Subscription, Renderer};

use once_cell::sync::Lazy;

use core::fmt;
use std::path::PathBuf;

use glob::glob;

use rodio::{Decoder, OutputStream, Sink};
use rodio::source::{SineWave, Source};

use std::io::BufReader;
use std::fs::File;


static INPUT_ID: Lazy<text_input::Id> = Lazy::new(text_input::Id::unique);

fn main() -> iced::Result {
    Player::run(
        Settings::default()
    )
}

struct State {
    input_value: String,
    cur_directory: PathBuf,
    tracks: Option<Vec<Track>>,
    cur_track_name: Option<String>,
    playing: bool, // Remove, replace with sink.is_paused()
    _stream: rodio::OutputStream,
    handle: rodio::OutputStreamHandle,
    sink: Sink,
}

impl std::fmt::Debug for State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("State")
            .field("input_value", &self.input_value)
            .field("cur_directory", &self.cur_directory)
            .field("tracks", &"I'm not writing this code")
            .field("cur_track_name", self.cur_track_name.as_ref().unwrap())
            .finish()
    }
}


impl Default for State {
    fn default() -> Self {
        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        Self {
            input_value: match dirs::home_dir() {
                Some(home) =>
                    format!("{}/", &home.as_path().display().to_string()),

                None => String::from(""),
            },
            tracks: match dirs::home_dir() {
                Some(home) => Some(index_cur_directory(&home)
                            .iter()
                            .map(|path|{
                                Track::new(path.clone())
                            })
                            .collect()),

                None => None,
            },
            cur_track_name: None,
            cur_directory: match dirs::home_dir() {
                Some(path) => [path.as_path(), &PathBuf::from("/")].iter().collect(),
                None => PathBuf::new()

            },
            playing: false,
            sink: Sink::try_new(&stream_handle).unwrap(),
            _stream: _stream,
            handle: stream_handle,
        }
    }
}


#[derive(Debug)]
enum Player {
    Loading, // Will be unused until persistent state is added
    Loaded(State),
}

#[derive(Debug, Clone)]
enum Message {
    LoadDirectory(PathBuf), //Load directory from directory path
    InputChanged(String), // Change the
    StartSong(PathBuf), //Play
    Play,
    Pause,
    // TODO: nextTrack,
    // TODO: prevTrack,
    DownPressed,
    UpPressed,
    EnterPressed,
}

// Multiple
impl Application for Player {
    type Message = Message;
    type Theme = Theme;
    type Flags = ();
    type Executor = iced::executor::Default;

    fn new(_flags: ()) -> (Player, Command<Self::Message>) {
        Message::LoadDirectory(State::default().cur_directory);
        (
            Player::Loaded(State::default()),
            // TODO: Command::perform( /* add load from save state function */ Message::Loaded)
            Command::none()
        )
    }

    fn title(&self) -> String {
        match self {
            Player::Loading => String::from("Swirl"),
            Player::Loaded(state) => format!("Swirl is Playing: {}", if state.cur_track_name == None { "Nothing" } else { state.cur_track_name.as_ref().unwrap() })
        }
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match self {
            Player::Loading => {
                *self = Player::Loaded(State::default());
                return Command::none();
            }
            Player::Loaded(state) => {
                match message {
                    Message::LoadDirectory(path_buf) => {
                        state.tracks = Some(index_cur_directory(&path_buf)
                            .iter()
                            .map(|path|{
                                Track::new(path.clone())
                            })
                            .collect());
                        state.cur_directory = path_buf;
                        return Command::none()
                    }
                    Message::StartSong(path) => {
                        state.cur_track_name = Some(get_file_name(&path));
                        state.sink.append(Decoder::new(BufReader::new(File::open(path).unwrap())).unwrap());
                        state.playing = true;
                        return Command::none();
                    }
                    Message::Pause =>{
                        state.playing = false;
                        state.sink.pause();
                         return Command::none();
                    }
                    Message::Play => {
                        state.playing = true;
                        state.sink.play();
                        return Command::none();
                    }
                    Message::DownPressed =>
                        widget::focus_next(),
                    Message::UpPressed =>
                        widget::focus_previous(),
                    Message::EnterPressed =>
                        // TODO: Implement
                        Command::none(),
                    Message::InputChanged(value) => {
                        state.input_value = (&value).to_string();
                        state.tracks = Some(index_cur_directory(&PathBuf::from(&value))
                            .iter()
                            .map(|path|{
                                Track::new(path.clone())
                            })
                            .collect());
                        state.cur_directory = PathBuf::from(value);
                        return Command::none()
                    }
                }
            }
        }
    }

    fn view(&self) -> Element<Message, Renderer> {
        match self {
            Player::Loading => text("Loading").into(),
            Player::Loaded(State {
                input_value,
                playing,
                tracks,
                sink,
                ..
            }) => {
                let toggle_play;
                let input = text_input(
                    "Choose a directory",
                    input_value,
                    Message::InputChanged)
                        .id(INPUT_ID.clone())
                        .padding(5);
                match playing { // TODO: replace with sink.playing
                    true => {
                        let pause = svg(svg::Handle::from_memory("
<svg
   fill='#000000'
   class='bi bi-play-fill'
   viewBox='0 0 16 16'
   version='1.1'
   id='svg4'
   width='16'
   height='16'
   xmlns='http://www.w3.org/2000/svg'
   xmlns:svg='http://www.w3.org/2000/svg'>
  <defs
     id='defs8' />
  <g
     id='layer1'>
    <rect
       style='fill:#000000'
       width='4.5'
       height='14'
       x='1.5'
       y='1' />
    <rect
       style='fill:#000000'
       width='4.5'
       height='15'
       x='10'
       y='1' />
  </g>
</svg>
".as_bytes()));
                        toggle_play = button(pause)
                            .on_press(Message::Pause);
                    }
                    false => {
                        let play = svg(svg::Handle::from_memory("
<svg
   fill='#000000'
   class='bi bi-play-fill'
   viewBox='0 0 16 16'
   version='1.1'
   id='svg4'
   width='16'
   height='16'
   xmlns='http://www.w3.org/2000/svg'
   xmlns:svg='http://www.w3.org/2000/svg'>
    <defs id='defs8' />
    <polygon points='1,1 15,8 1,15'/>
</svg>
".as_bytes()));
                        toggle_play = button(play)
                            .on_press(Message::Play);
                    }
                }
                let row = row![
                    toggle_play,]
                        .spacing(1)
                        .padding(1);
                let scroll = match tracks {
                    Some(tracks) =>
                    scrollable(
                        column(
                            tracks
                                .iter()
                                .map(|track|{
                                    track.view()
                                })
                                .collect(),
                        )
                            .padding(1)),
                    None =>
                    scrollable(
                    container(text(""))) //change this later
                };
                // WIDGETS NOT ASSIGNED TO A MASTER CONTAINER WILL ERROR!!!!!
                let container_col = column![input, scroll, row]
                    .spacing(1);
                container(container_col).into()
            }
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        subscription::events_with(|event, status| match (event, status) {
            (
                Event::Keyboard(keyboard::Event::KeyPressed {
                    key_code: keyboard::KeyCode::Down,
                    ..
                }),
                event::Status::Ignored,
            ) => Some(Message::DownPressed),
            (
                Event::Keyboard(keyboard::Event::KeyPressed {
                    key_code: keyboard::KeyCode::Up,
                    ..
                }),
                event::Status::Ignored,
            ) => Some(Message::UpPressed),
            (
                Event::Keyboard(keyboard::Event::KeyPressed {
                    key_code: keyboard::KeyCode::Enter,
                    ..
                }),
                event::Status::Captured,
            ) => Some(Message::EnterPressed),
            _ => None,
        })
    }
}

#[derive(Debug, Clone)]
struct Track {
    path: PathBuf,
    name: String
}

// A singular music track
impl Track {
    fn new(path: PathBuf) -> Self{
        Track {
            name: get_file_name(&path),
            path: path,
        }
    }
    fn view(&self) -> Element<Message>{
        button(text(&self.name))
            .on_press(Message::StartSong(self.path.clone()))
            .into()
    }

}

fn get_file_name(path: &PathBuf) -> String {
    let mut full_path: Vec<String> = path.to_string_lossy().split(['/', '\\'].as_ref()).map(String::from).collect();
    return full_path.pop().unwrap();
}

fn index_cur_directory(cur_directory: &PathBuf) -> Vec<PathBuf> {
    let audio_index: Result<Vec<_>, _> = glob(&format!("{}{}", cur_directory.as_path().display().to_string(), "/*.mp3")).expect("Failed to read glob pattern").collect();
    let unwrapped_index = audio_index.unwrap();
    return unwrapped_index;
}
