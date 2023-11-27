#![allow(non_snake_case)]

use std::process::{Command, Output};
use std::time::{Duration, Instant};

use tray_item::{IconSource, TrayItem};
use std::sync::mpsc::{self};


enum Message {
    Start,
    Stop,
    Skip,
    TimeCheck,
    Quit,
    Debug,
}



#[derive(Clone, PartialEq, Eq, Debug)]
enum TimedState{
    Working,
    Break,
    LongerBreak,
}

impl TimedState{
    pub fn getLength(&self) -> Duration{
        let minLength = match &self {
            TimedState::Working => 25,
            TimedState::Break => 5,
            TimedState::LongerBreak => 15,
        };
        Duration::from_secs(minLength * 60)
    }

    pub fn nextState(&self, isFourthBreak: bool) -> TimedState{
        match self {
            TimedState::Working => {
                if isFourthBreak {
                    TimedState::LongerBreak
                } else {
                    TimedState::Break
                }
            },
            TimedState::Break | TimedState::LongerBreak => {
                TimedState::Working
            },
        }
    }
}

#[derive(Debug)]
enum State{
    Timed(Instant, TimedState),
    Hatled,
}

impl State{
    pub fn getElapsed(&self) -> Option<Duration>{
        let startTime = match &self{
            Self::Hatled => {return None},
            Self::Timed(s, _) => {s.clone()}
        };
        
        println!("{:?}", startTime.elapsed());
        Some(startTime.elapsed())
    }

    pub fn getTechnique(&self) -> Option<TimedState>{
        let technique = match &self{
            Self::Hatled => {return None},
            Self::Timed(_, t) => {t}
        };
        
        Some(technique.clone())
    }

    pub fn getData(&self) -> (IconSource, &str, &str){
        match self{
            Self::Timed(_, TimedState::Working) => (IconSource::Resource("working"), "working", "time to start working!"),
            Self::Timed(_, TimedState::Break) => (IconSource::Resource("break"), "Break Time!", "short break time!"),
            Self::Timed(_, TimedState::LongerBreak) => (IconSource::Resource("longer-break"), "Longer Break Time!", "longer break time! have fun!"),
            Self::Hatled => (IconSource::Resource("default"), "halted", "halted timer"),
        }
    }
}

fn main() {

    println!("welcome to the system tray pomodoro timer");
    println!("this program was written in rust by Otter502");
    println!("here is the link to the github repository: https://github.com/otter502/Pomodoro-System-Tray-Icon");

    // creating the tray
    
    let mut tray = TrayItem::new(
        "Pomodoro Timer",
        IconSource::Resource("default"),
    ).unwrap();

    tray.inner_mut().add_separator().unwrap(); 

    let labelID = tray.inner_mut().add_label_with_id("pomodoro").unwrap();

    tray.inner_mut().add_separator().unwrap();

    let (tx, rx) = mpsc::sync_channel::<Message>(1);

    let start_tx = tx.clone();
    tray.add_menu_item("Start", move || {
        start_tx.send(Message::Start).unwrap();
    }).unwrap();

    let stop_tx = tx.clone();
    tray.add_menu_item("Stop", move || {
        stop_tx.send(Message::Stop).unwrap();
    }).unwrap();

    let skip_tx = tx.clone();
    tray.add_menu_item("Skip", move || {
        skip_tx.send(Message::Skip).unwrap();
    }).unwrap();

    let timeCheck_tx = tx.clone();
    tray.add_menu_item("TimeCheck", move || {
        timeCheck_tx.send(Message::TimeCheck).unwrap();
    }).unwrap();

    tray.inner_mut().add_separator().unwrap();

    let debug_tx = tx.clone();
    tray.add_menu_item("Debug", move || {
        debug_tx.send(Message::Debug).unwrap();
    }).unwrap();

    let quit_tx = tx.clone();
    tray.add_menu_item("Quit", move || {
        quit_tx.send(Message::Quit).unwrap();
    }).unwrap();

    tray.inner_mut().add_separator().unwrap(); //here so its harder to misclick the quit button



    //variables for the loop

    let mut currState: State;
    
    let mut numberOfBreaks: i32 = 1;

    let mut windowVisible = false;

    let button_tx = tx.clone(); //the sender for the loop
    
    currState = State::Hatled;
    updateLabel(&currState, &mut tray, &labelID, false);

    loop {
        match &currState{
            State::Timed(startTime, technique) => {
                // println!("{:?}", startTime.elapsed());
                if startTime.elapsed() > technique.getLength() {
                    button_tx.send(Message::Skip).unwrap();
                }
            },
            State::Hatled => {},
        }


        match rx.recv_timeout(Duration::from_secs(1)) {
            Ok(Message::Quit) => {
                println!("Quit");
                break;
            }

            Ok(Message::TimeCheck) => {
                let elapsed: Duration;
                if let Some(e) = (&currState).getElapsed() {
                    elapsed = e;
                } else {
                    msgAll("currently halted");
                    continue;
                }

                let mut minutesLeft = ((&currState).getTechnique().unwrap().getLength().as_secs() - elapsed.as_secs()) / 60;
                
                minutesLeft += 1; //5-1 instead of 4-0

                let output: String = format!("you have minutes {minutesLeft} left");
                msgAll(&output.as_str());
            }

            Ok(Message::Skip) => {

                let nextState = match &currState{
                    State::Timed(_, t) => t.nextState(numberOfBreaks % 4 == 0),
                    State::Hatled => {
                        msgAll("currently halted");
                        println!("please unhalt the timer");
                        continue;
                    },
                };
                if nextState == TimedState::Break || nextState == TimedState::LongerBreak {
                    numberOfBreaks += 1;
                }

                currState = State::Timed(Instant::now(), nextState);

                updateLabel(&currState, &mut tray, &labelID, true);
            }

            Ok(Message::Start) => {
                let nextState = match &currState{
                    State::Timed(_, t) => t.clone(),
                    State::Hatled => TimedState::Working,
                };
                currState = State::Timed(Instant::now(), nextState);

                updateLabel(&currState, &mut tray, &labelID, true);
            }
            Ok(Message::Stop) => {

                currState = State::Hatled;

                updateLabel(&currState, &mut tray, &labelID, true);
            }
            
            Ok(Message::Debug) => {
                windowVisible = !windowVisible;
                setWindowVisibility(windowVisible);
            }
            _ => {}
        }
    }
}

fn msgAll(content: &str) -> Output{
    Command::new("cmd")
        .args(["/C", ("powershell msg * ".to_string() + content).as_str()])
        .output()
        .expect("failed to execute process")
}

fn updateLabel(currState: &State, tray: &mut TrayItem, labelID: &u32, sendMsg: bool){
    let data = currState.getData();
    if sendMsg {msgAll(data.2);}
    let _ = tray.inner_mut().set_menu_item_label(data.1, *labelID);
    let _ = tray.inner_mut().set_icon(data.0);
    println!("updated tray label: {:?}", currState);
}

fn setWindowVisibility(visible: bool){

    let visibilityText = if visible {
        println!("don't close this window, it will end the program");
        "normal"
    } else {
        "hidden"
    };

    Command::new("cmd")
        .args(["/C", (format!("powershell -WindowStyle {visibilityText}").as_str())])
        .output()
        .expect("failed to execute process");
}