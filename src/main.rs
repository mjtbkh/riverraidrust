use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{poll, read, Event, KeyCode, KeyEventKind},
    style::Print,
    terminal::{disable_raw_mode, enable_raw_mode, size, Clear, ClearType},
    ExecutableCommand, QueueableCommand,
};
use rand::{thread_rng, Rng};
use std::{
    cmp::Ordering::*,
    io::{stdout, Stdout, Write},
};
use std::{
    thread,
    time::{self, Duration},
};

#[derive(PartialEq, Eq)]
enum PlayerStatus {
    Dead,
    Alive,
    Animation,
    Paused,
}

struct World {
    player_c: u16,
    player_l: u16,
    maxc: u16,
    maxl: u16,
    status: PlayerStatus,
    map: Vec<(u16, u16)>,
    next_right: u16,
    next_left: u16,
    ship: String,
}

impl World {
    fn new(maxc: u16, maxl: u16) -> World {
        World {
            player_c: maxc / 2,
            player_l: maxl - 1,
            map: vec![(maxc / 2 - 5, maxc / 2 + 5); maxl as usize],
            maxc,
            maxl,
            status: PlayerStatus::Alive,
            next_left: maxc / 2 - 7,
            next_right: maxc / 2 + 7,
            ship: '⛵'.to_string(),
        }
    }
}

fn draw(mut sc: &Stdout, world: &World) -> std::io::Result<()> {
    sc.queue(Clear(ClearType::All))?;

    // draw the map
    for l in 0..world.map.len() {
        sc.queue(MoveTo(0, l as u16))?
            .queue(Print("*".repeat(world.map[l].0 as usize)))?
            .queue(MoveTo(world.map[l].1, l as u16))?
            .queue(Print("*".repeat((world.maxc - world.map[l].1) as usize)))?;
    }

    // draw the player
    sc.queue(MoveTo(world.player_c, world.player_l))?
        .queue(Print(world.ship.as_str()))?
        .flush()?;

    Ok(())
}

fn physics(world: &mut World) {
    let mut rng = thread_rng();

    // check if player has hit the ground
    if world.player_c <= world.map[world.player_l as usize].0
        || world.player_c >= world.map[world.player_l as usize].1
    {
        world.status = PlayerStatus::Dead
    }

    // move map downward
    for l in (1..world.map.len()).rev() {
        world.map[l] = world.map[l - 1]
    }

    let (left, right) = &mut world.map[0];
    match world.next_left.cmp(left) {
        Greater => *left += 1,
        Less => *left -= 1,
        Equal => {}
    };

    match world.next_right.cmp(right) {
        Greater => *right += 1,
        Less => *right -= 1,
        Equal => {}
    };

    if world.next_left == world.map[0].0 && rng.gen_range(0..10) >= 7 {
        world.next_left = rng.gen_range(world.next_left - 5..world.next_left + 5)
    }
    if world.next_right == world.map[0].1 && rng.gen_range(0..10) >= 7 {
        world.next_right = rng.gen_range(world.next_right - 5..world.next_right + 5)
    }

    if world.next_right.abs_diff(world.next_left) < 3 {
        world.next_right += 3;
    }
}

fn main() -> std::io::Result<()> {
    let mut sc = stdout();
    enable_raw_mode()?;
    let (maxc, maxl) = size().unwrap();
    sc.execute(Hide)?;

    let slowness = 100;
    let mut world = World::new(maxc, maxl);

    while world.status == PlayerStatus::Alive {
        if poll(Duration::from_millis(10))? {
            let key = read().unwrap();
            while poll(Duration::from_millis(0)).unwrap() {
                let _ = read();
            }
            match key {
                Event::Key(event) => {
                    if event.kind == KeyEventKind::Press {
                        match event.code {
                            KeyCode::Char('q') => break,
                            KeyCode::Char('w') => {
                                if world.player_l > 1 {
                                    world.player_l -= 1
                                }
                            }
                            KeyCode::Char('s') => {
                                if world.player_l < maxl - 1 {
                                    world.player_l += 1
                                }
                            }
                            KeyCode::Char('d') => {
                                if world.player_c < maxc - 1 {
                                    world.player_c += 1
                                }
                            }
                            KeyCode::Char('a') => {
                                if world.player_c > 1 {
                                    world.player_c -= 1
                                }
                            }
                            KeyCode::Up => {
                                if world.player_l > 1 {
                                    world.player_l -= 1
                                }
                            }
                            KeyCode::Down => {
                                if world.player_l < maxl - 1 {
                                    world.player_l += 1
                                }
                            }
                            KeyCode::Left => {
                                if world.player_c > 1 {
                                    world.player_c -= 1
                                }
                            }
                            KeyCode::Right => {
                                if world.player_c < maxc - 1 {
                                    world.player_c += 1
                                }
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }

        physics(&mut world);

        draw(&sc, &world)?;

        std::thread::sleep(std::time::Duration::from_millis(slowness));
    }
    sc.queue(Clear(ClearType::All))?;
    sc.queue(MoveTo(maxc / 2, maxl / 2))?;
    sc.queue(Print("Good game! Thanks.\n"))?;
    thread::sleep(time::Duration::from_millis(3000));
    sc.queue(Clear(ClearType::All))?;
    sc.execute(Show)?;
    disable_raw_mode()?;
    Ok(())
}
