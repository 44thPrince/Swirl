/*
iced Architecture:
    State — the state of your application
    Messages — user interactions or meaningful events that you care about
    View logic — a way to display your state as widgets that may produce messages on user interaction
    Update logic — a way to react to messages and update your state
    (My own addition) Persistence - how the application stores data that persists after the application is closed

Planning - Split into subsections for each component of the architecture. Single indented are necessary features for the application to function, double or more are for unneccessary features or improvements

Make skeleton program
    TODO: Align widgets to proper locations
    TODO: Create pause play widget
    TODO: Fast forward//backward widgets
    TODO: Implement audio

State:
    TODO: Current directory (None if nothing is currently selected)
    TODO: Store index of mp3 files, lazy load file currently playing and the two in front of and behind it using said index
    TODO: Current file playing (None if nothing is selected)
    TODO: Current state (enum) that stores whether song is paused or playing, or loading a new song

Messages:
    TODO: Pause/play event
        TODO: Next/prev track
    TODO: Folder picker
        TODO: Emacs dired style folder picker (replaces above)

View Logic:
    TODO: Pause, play button
        TODO: Next, prev track button
    TODO: Open folder button -> folder picker ->
    TODO: Show list of tracks in folder -> track selector
        TODO: Shuffle play

Update Logic:
    TODO: Change current directory when folder is opened
    TODO: Selecting a track, or selecting next/prev track updates current state enum, current file playing

    Persistence:
        TODO: Save current directory on program exit, load on program start. (optional)
            TODO: Save current track on program exit, load on program start. (optional) (depends on above)
*/

use iced::alignment::{self, Alignment};
use iced::event::{self, Event};
use iced::keyboard;
use iced::subscription;
use iced::theme::{self, Theme};
use iced::widget::{
    self, button, Button, checkbox, column, container, row, scrollable, text, text_input, Text, svg
};
use iced::window;
use iced::{Application, Element};
use iced::{Color, Command, Font, Length, Settings, Subscription, Renderer};

use iced_native::Widget;
use once_cell::sync::Lazy;

use std::fs::{self, File};

use std::io::BufReader;
use std::io::prelude::*;
use std::ops::ShrAssign;
use std::path::{Path, PathBuf};

use glob::glob;


// use rodio::{Decoder, OutputStream, Sink};
// use rodio::source::{SineWave, Source};


static INPUT_ID: Lazy<text_input::Id> = Lazy::new(text_input::Id::unique);

fn main() -> iced::Result {
    Player::run(
        Settings::default()
    )
}

// custom button implementation


// State
#[derive(Debug, Default)]
struct State {
    input_value: String,
    cur_directory: String,
    audio_index: Vec<PathBuf>,
    track_names: Vec<String>,
    // TODO: cur_track: Sink,
    cur_track_name: String,
    playing: bool,
    width: u32,
    height: u32,
}

// Message
#[derive(Debug)]
enum Player {
    Loading,
    Loaded(State),
}

// Message
#[derive(Debug, Clone)]
enum Message {
    LoadDirectory(String),
    LoadTrack(PathBuf),
    InputChanged(String),
    Play,
    Pause,
    // TODO: nextTrack,
    // TODO: prevTrack,
    DownPressed,
    UpPressed,
    EnterPressed,
    Resized(u32, u32)
}

// Multiple
impl Application for Player {
    type Message = Message;
    type Theme = Theme;
    type Flags = ();
    type Executor = iced::executor::Default;

    fn new(_flags: ()) -> (Player, Command<Self::Message>) {
        (
            Player::Loaded(State::default()),
            // TODO: Command::perform( /* add load from save state function */ Message::Loaded)
            Command::none()
        )
    }

    fn title(&self) -> String {
        match self {
            Player::Loading => String::from("Swirl"),
            Player::Loaded(state) => format!("Swirl is Playing: {}", if state.cur_track_name == "".to_string() { "Nothing" } else { &state.cur_track_name })
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
                    Message::LoadDirectory(string) => {
                        state.audio_index = index_cur_directory(&string);
                        state.cur_directory = string;
                        return Command::none()
                    }
                    Message::LoadTrack(path) => {
                        let path = get_file_names(vec![path]);
                        state.cur_track_name = path[0].to_string();
                        return Command::none()
                    }
                    Message::Pause =>{
                        state.playing = false;
                         return Command::none();
                    }
                    Message::Play => {
                        state.playing = true;
                        return Command::none();
                    }
                        //TODO: Implement
                    Message::DownPressed =>
                        widget::focus_next(),
                        // Add check for custom widget containing tracks
                    Message::UpPressed =>
                        widget::focus_previous(),
                        // Add check for custom widget containing tracks
                    Message::EnterPressed =>
                        // Activate the widget focused on or play selected track
                        // To be implemented later
                        Command::none(),
                    Message::InputChanged(value) => {
                        state.input_value = (&value).to_string();
                        Message::LoadDirectory(value);
                        return Command::none()
                    }
                    Message::Resized(width, height) => {
                        state.width = width;
                        state.height = height;
                        return Command::none();
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
                height,
                audio_index,
                ..
            }) => {
                let toggle_play;
                let input = text_input(
                    "Choose a directory",
                    input_value,
                    Message::InputChanged)
                        .id(INPUT_ID.clone())
                        .padding(5);
                audio_index = &index_cur_directory(&input_value);
                match playing {
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
       id='rect167'
       width='2.2962112'
       height='9.0195179'
       x='4.5'
       y='3.5' />
    <rect
       style='fill:#000000'
       id='rect167-3'
       width='2.2962112'
       height='9.0195179'
       x='9.25'
       y='3.5' />
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
   viewBox='0 0 256 256'
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
    <path
       style='fill:#000000'
       id='path442'
       d='m 10.397244,6.0803676 -5.6349021,3.2533125 -5.63490216,3.2533119 0,-6.5066246 0,-6.50662462 5.63490256,3.25331252 z'
       transform='matrix(0.66549512,0,0,0.69160281,4.8306846,3.7948007)' />
  </g>
</svg>
".as_bytes()));
                        toggle_play = button(play)
                            .on_press(Message::Play);
                    }
                }
                let row = row![
                    toggle_play,]
                        .spacing(5)
                        .padding(10)
                        .height(input.height());
                let col = column![];
                let scroll = scrollable(
                    container(col)
                        .padding(20))
                            .height(Length::Fill);
                // WIDGETS NOT ASSIGNED TO A MASTER CONTAINER WILL ERROR!!!!!
                let container_col = column![input, scroll, row]
                    .spacing(5);
                container(container_col).into()
            }
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        subscription::events_with(|event, status| match (event, status) {
            (
                Event::Keyboard(keyboard::Event::KeyPressed {
                   key_code: keyboard::KeyCode::Down,
                   modifiers,
                }),
                event::Status::Ignored,
            ) => Some(Message::DownPressed),
            (
                Event::Keyboard(keyboard::Event::KeyPressed {
                    key_code: keyboard::KeyCode::Up,
                    modifiers,
                }),
                event::Status::Ignored,
            ) => Some(Message::UpPressed),
            (
                Event::Keyboard(keyboard::Event::KeyPressed {
                    key_code: keyboard::KeyCode::Enter,
                    modifiers,
                }),
                event::Status::Captured,
            ) => Some(Message::EnterPressed),
            (
                Event::Window(window::Event::Resized {
                    width,
                    height,
                }),
                event::Status::Captured,
            ) => Some(Message::Resized(width, height)),
            _ => None,
        })
    }
}

fn get_file_names (paths: Vec<PathBuf>) -> Vec<String> {
    let mut return_string = Vec::new();
    for path in paths {
        let full_path = path.to_string_lossy();
        let split_path: Vec<String> = Vec::from_iter(full_path.split(['/', '\\'].as_ref()).map(String::from));
        return_string.push(String::from(&split_path[full_path.len() - 1])); // Shitty code
    }
    return return_string;
}

fn index_cur_directory(cur_directory: &String) -> Vec<PathBuf> {
    let audio_index: Result<Vec<_>, _> = glob(&format!("{}{}", cur_directory, "/*.png")).expect("Failed to read glob pattern").collect();
    let unwrapped_index = audio_index.unwrap();
    return unwrapped_index;
}
