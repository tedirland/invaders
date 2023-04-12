

use std::{io, error::Error, time::{Duration, Instant}, sync::mpsc, thread};

use crossterm::{terminal::{self, EnterAlternateScreen, LeaveAlternateScreen}, ExecutableCommand, cursor::{Hide, Show}, event::{self, Event, KeyCode}};
use invaders::{frame::{self, new_frame, Drawable}, render, player::Player};
use rusty_audio::Audio;

fn main() -> Result<(),Box<dyn Error>> {
 let mut audio = Audio::new();
 audio.add("explode", "explode.wav");
 audio.add("lose", "lose.wav");
 audio.add("move", "move.wav");
 audio.add("pew", "pew.wav");
 audio.add("startup", "startup.wav");
 audio.add("win", "win.wav");
 audio.play("startup");

// Terminal
let mut stdout = io::stdout();
terminal::enable_raw_mode()?;
stdout.execute(EnterAlternateScreen)?;
stdout.execute(Hide)?;

// Render loop in a seperate thread
let (render_tx, render_rx) = mpsc::channel();
let render_handle = thread::spawn(move || {
    let mut last_frame = frame::new_frame();
    let mut stdout = io::stdout();
    render::render(&mut stdout, &last_frame, &last_frame, true);
    loop {
       let current_frame =  match render_rx.recv() {
            Ok(x) => x,
            Err(_) => break,
        };
        render::render(&mut stdout, &last_frame, &current_frame, false);
        last_frame = current_frame;
    }

});

// Game Loop
let mut player = Player::new();
let mut instant = Instant::now();
'gameloop: loop {
    // Per-frame init
    let delta = instant.elapsed();
    instant = Instant::now();
    let mut current_frame = new_frame();
    // Input 
    while event::poll(Duration::default())?  {
        if let Event::Key(key_event) = event::read()?{
            match key_event.code {
                KeyCode::Left => player.move_left(),
                KeyCode::Right => player.move_right(),
                KeyCode::Char(' ') | KeyCode::Enter => {
                    if player.shoot() {
                        audio.play("pew")
                    }
                },
                KeyCode::Esc | KeyCode::Char('q') => {
                    audio.play("lose");
                    break 'gameloop;
                },
                _ => {}                
            }

        }
        
    }
    // Updates
    player.update(delta);
    // Draw and Render
    player.draw(&mut current_frame);
    let _ = render_tx.send(current_frame);
    thread::sleep(Duration::from_millis(1));
}


//  Cleanup
drop(render_tx);
render_handle.join().unwrap();
audio.wait();
stdout.execute(Show)?;
stdout.execute(LeaveAlternateScreen)?;
terminal::disable_raw_mode()?;

Ok(())
 

}
