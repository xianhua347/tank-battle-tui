use std::{
    collections::HashSet,
    io::{self, Stdout},
    time::{Duration, Instant},
};

use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use rand::{thread_rng, Rng};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction as LayoutDirection, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame, Terminal,
};

const WIDTH: i32 = 42;
const HEIGHT: i32 = 22;
const TICK_RATE: Duration = Duration::from_millis(90);
const ENEMY_COUNT: usize = 6;

fn main() -> io::Result<()> {
    let mut terminal = setup_terminal()?;
    let result = run(&mut terminal);
    restore_terminal(&mut terminal)?;
    result
}

fn setup_terminal() -> io::Result<Terminal<CrosstermBackend<Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    Terminal::new(CrosstermBackend::new(stdout))
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> io::Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()
}

fn run(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> io::Result<()> {
    let mut app = App::new();
    let mut last_tick = Instant::now();

    loop {
        terminal.draw(|frame| draw(frame, &app))?;

        let timeout = TICK_RATE
            .checked_sub(last_tick.elapsed())
            .unwrap_or(Duration::ZERO);

        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if app.handle_key(key.code) {
                    break;
                }
            }
        }

        if last_tick.elapsed() >= TICK_RATE {
            app.tick();
            last_tick = Instant::now();
        }
    }

    Ok(())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Dir {
    Up,
    Down,
    Left,
    Right,
}

impl Dir {
    fn delta(self) -> (i32, i32) {
        match self {
            Dir::Up => (0, -1),
            Dir::Down => (0, 1),
            Dir::Left => (-1, 0),
            Dir::Right => (1, 0),
        }
    }

    fn symbol(self) -> char {
        match self {
            Dir::Up => '^',
            Dir::Down => 'v',
            Dir::Left => '<',
            Dir::Right => '>',
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Pos {
    x: i32,
    y: i32,
}

impl Pos {
    fn step(self, dir: Dir) -> Self {
        let (dx, dy) = dir.delta();
        Self {
            x: self.x + dx,
            y: self.y + dy,
        }
    }
}

#[derive(Clone, Debug)]
struct Tank {
    pos: Pos,
    dir: Dir,
    cooldown: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Owner {
    Player,
    Enemy,
}

#[derive(Clone, Debug)]
struct Bullet {
    pos: Pos,
    dir: Dir,
    owner: Owner,
}

#[derive(Clone, Debug)]
struct Explosion {
    pos: Pos,
    age: u8,
}

#[derive(Debug)]
struct App {
    player: Tank,
    enemies: Vec<Tank>,
    bullets: Vec<Bullet>,
    explosions: Vec<Explosion>,
    walls: HashSet<Pos>,
    score: u32,
    lives: u8,
    paused: bool,
    game_over: bool,
    tick_count: u64,
}

impl App {
    fn new() -> Self {
        let mut app = Self {
            player: Tank {
                pos: Pos {
                    x: WIDTH / 2,
                    y: HEIGHT - 3,
                },
                dir: Dir::Up,
                cooldown: 0,
            },
            enemies: Vec::new(),
            bullets: Vec::new(),
            explosions: Vec::new(),
            walls: HashSet::new(),
            score: 0,
            lives: 3,
            paused: false,
            game_over: false,
            tick_count: 0,
        };
        app.reset_map();
        app
    }

    fn reset(&mut self) {
        *self = Self::new();
    }

    fn reset_map(&mut self) {
        let mut rng = thread_rng();

        for _ in 0..52 {
            let pos = Pos {
                x: rng.gen_range(2..WIDTH - 2),
                y: rng.gen_range(3..HEIGHT - 4),
            };
            if (pos.x - self.player.pos.x).abs() > 3 || (pos.y - self.player.pos.y).abs() > 3 {
                self.walls.insert(pos);
            }
        }

        for i in 0..ENEMY_COUNT {
            let x = 4 + (i as i32 * 7) % (WIDTH - 8);
            let y = 2 + (i as i32 / 4) * 3;
            let pos = Pos { x, y };
            if !self.walls.contains(&pos) {
                self.enemies.push(Tank {
                    pos,
                    dir: Dir::Down,
                    cooldown: rng.gen_range(8..20),
                });
            }
        }
    }

    fn handle_key(&mut self, key: KeyCode) -> bool {
        match key {
            KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => return true,
            KeyCode::Char('p') | KeyCode::Char('P') => {
                if !self.game_over {
                    self.paused = !self.paused;
                }
            }
            KeyCode::Char('r') | KeyCode::Char('R') => {
                if self.game_over {
                    self.reset();
                }
            }
            KeyCode::Char(' ') => {
                if !self.paused && !self.game_over {
                    self.player_shoot();
                }
            }
            KeyCode::Up | KeyCode::Char('w') | KeyCode::Char('W') => self.move_player(Dir::Up),
            KeyCode::Down | KeyCode::Char('s') | KeyCode::Char('S') => self.move_player(Dir::Down),
            KeyCode::Left | KeyCode::Char('a') | KeyCode::Char('A') => self.move_player(Dir::Left),
            KeyCode::Right | KeyCode::Char('d') | KeyCode::Char('D') => self.move_player(Dir::Right),
            _ => {}
        }

        false
    }

    fn tick(&mut self) {
        if self.paused || self.game_over {
            return;
        }

        self.tick_count += 1;
        self.player.cooldown = self.player.cooldown.saturating_sub(1);
        for enemy in &mut self.enemies {
            enemy.cooldown = enemy.cooldown.saturating_sub(1);
        }

        self.update_enemies();
        self.update_bullets();
        self.update_explosions();

        if self.enemies.is_empty() {
            self.spawn_wave();
        }
    }

    fn move_player(&mut self, dir: Dir) {
        if self.paused || self.game_over {
            return;
        }

        self.player.dir = dir;
        let next = self.player.pos.step(dir);
        if self.can_enter(next, Some(Owner::Player)) {
            self.player.pos = next;
        }
    }

    fn player_shoot(&mut self) {
        if self.player.cooldown > 0 {
            return;
        }

        self.bullets.push(Bullet {
            pos: self.player.pos.step(self.player.dir),
            dir: self.player.dir,
            owner: Owner::Player,
        });
        self.player.cooldown = 5;
    }

    fn update_enemies(&mut self) {
        let mut rng = thread_rng();
        let occupied: HashSet<Pos> = self.enemies.iter().map(|enemy| enemy.pos).collect();
        let mut actions = Vec::new();

        for (index, enemy) in self.enemies.iter().enumerate() {
            let mut dir = enemy.dir;
            if rng.gen_bool(0.18) {
                dir = random_dir(&mut rng);
            }

            let next = enemy.pos.step(dir);
            let can_move = in_bounds(next)
                && !self.walls.contains(&next)
                && next != self.player.pos
                && (!occupied.contains(&next) || next == enemy.pos);
            let should_shoot = enemy.cooldown == 0 && rng.gen_bool(0.42);
            actions.push((index, dir, can_move, should_shoot));
        }

        for (index, dir, can_move, should_shoot) in actions {
            let current = self.enemies[index].pos;
            self.enemies[index].dir = dir;
            if can_move {
                self.enemies[index].pos = current.step(dir);
            }

            if should_shoot {
                let pos = self.enemies[index].pos.step(self.enemies[index].dir);
                self.bullets.push(Bullet {
                    pos,
                    dir: self.enemies[index].dir,
                    owner: Owner::Enemy,
                });
                self.enemies[index].cooldown = 14;
            }
        }
    }

    fn update_bullets(&mut self) {
        let mut next_bullets = Vec::new();
        let mut killed_enemies = HashSet::new();
        let mut damaged_player = false;

        for bullet in &self.bullets {
            if !in_bounds(bullet.pos) {
                continue;
            }

            if self.walls.remove(&bullet.pos) {
                self.explosions.push(Explosion {
                    pos: bullet.pos,
                    age: 0,
                });
                continue;
            }

            match bullet.owner {
                Owner::Player => {
                    if let Some((index, _)) = self
                        .enemies
                        .iter()
                        .enumerate()
                        .find(|(_, enemy)| enemy.pos == bullet.pos)
                    {
                        killed_enemies.insert(index);
                        self.score += 100;
                        self.explosions.push(Explosion {
                            pos: bullet.pos,
                            age: 0,
                        });
                        continue;
                    }
                }
                Owner::Enemy => {
                    if bullet.pos == self.player.pos {
                        damaged_player = true;
                        self.explosions.push(Explosion {
                            pos: bullet.pos,
                            age: 0,
                        });
                        continue;
                    }
                }
            }

            let next = bullet.pos.step(bullet.dir);

            if !in_bounds(next) {
                continue;
            }

            if self.walls.remove(&next) {
                self.explosions.push(Explosion { pos: next, age: 0 });
                continue;
            }

            match bullet.owner {
                Owner::Player => {
                    if let Some((index, _)) = self
                        .enemies
                        .iter()
                        .enumerate()
                        .find(|(_, enemy)| enemy.pos == next)
                    {
                        killed_enemies.insert(index);
                        self.score += 100;
                        self.explosions.push(Explosion { pos: next, age: 0 });
                        continue;
                    }
                }
                Owner::Enemy => {
                    if next == self.player.pos {
                        damaged_player = true;
                        self.explosions.push(Explosion { pos: next, age: 0 });
                        continue;
                    }
                }
            }

            next_bullets.push(Bullet {
                pos: next,
                dir: bullet.dir,
                owner: bullet.owner,
            });
        }

        self.enemies = self
            .enemies
            .iter()
            .enumerate()
            .filter_map(|(index, enemy)| (!killed_enemies.contains(&index)).then(|| enemy.clone()))
            .collect();
        self.bullets = next_bullets;

        if damaged_player {
            self.lives = self.lives.saturating_sub(1);
            self.player.pos = Pos {
                x: WIDTH / 2,
                y: HEIGHT - 3,
            };
            self.player.dir = Dir::Up;
            if self.lives == 0 {
                self.game_over = true;
            }
        }
    }

    fn update_explosions(&mut self) {
        for explosion in &mut self.explosions {
            explosion.age += 1;
        }
        self.explosions.retain(|explosion| explosion.age < 5);
    }

    fn spawn_wave(&mut self) {
        let mut rng = thread_rng();
        let count = ENEMY_COUNT + (self.score / 800) as usize;

        for _ in 0..count.min(10) {
            for _ in 0..30 {
                let pos = Pos {
                    x: rng.gen_range(2..WIDTH - 2),
                    y: rng.gen_range(1..HEIGHT / 2),
                };
                if self.can_enter(pos, Some(Owner::Enemy)) && pos != self.player.pos {
                    self.enemies.push(Tank {
                        pos,
                        dir: Dir::Down,
                        cooldown: rng.gen_range(5..22),
                    });
                    break;
                }
            }
        }
    }

    fn can_enter(&self, pos: Pos, mover: Option<Owner>) -> bool {
        if !in_bounds(pos) || self.walls.contains(&pos) {
            return false;
        }

        match mover {
            Some(Owner::Player) => !self.enemies.iter().any(|enemy| enemy.pos == pos),
            Some(Owner::Enemy) => {
                pos != self.player.pos && !self.enemies.iter().any(|enemy| enemy.pos == pos)
            }
            None => true,
        }
    }
}

fn random_dir(rng: &mut impl Rng) -> Dir {
    match rng.gen_range(0..4) {
        0 => Dir::Up,
        1 => Dir::Down,
        2 => Dir::Left,
        _ => Dir::Right,
    }
}

fn in_bounds(pos: Pos) -> bool {
    pos.x >= 0 && pos.x < WIDTH && pos.y >= 0 && pos.y < HEIGHT
}

fn draw(frame: &mut Frame<'_>, app: &App) {
    let chunks = Layout::default()
        .direction(LayoutDirection::Vertical)
        .constraints([Constraint::Length((HEIGHT + 2) as u16), Constraint::Min(3)])
        .split(frame.area());

    let mut cells = vec![vec![' '; WIDTH as usize]; HEIGHT as usize];

    for wall in &app.walls {
        if in_bounds(*wall) {
            cells[wall.y as usize][wall.x as usize] = '#';
        }
    }

    for bullet in &app.bullets {
        if in_bounds(bullet.pos) {
            cells[bullet.pos.y as usize][bullet.pos.x as usize] = match bullet.owner {
                Owner::Player => '*',
                Owner::Enemy => 'o',
            };
        }
    }

    for explosion in &app.explosions {
        if in_bounds(explosion.pos) {
            cells[explosion.pos.y as usize][explosion.pos.x as usize] = match explosion.age {
                0 | 1 => '@',
                2 | 3 => 'x',
                _ => '.',
            };
        }
    }

    for enemy in &app.enemies {
        if in_bounds(enemy.pos) {
            cells[enemy.pos.y as usize][enemy.pos.x as usize] = enemy.dir.symbol();
        }
    }

    if in_bounds(app.player.pos) {
        cells[app.player.pos.y as usize][app.player.pos.x as usize] = app.player.dir.symbol();
    }

    let board_lines: Vec<Line<'_>> = cells
        .into_iter()
        .map(|row| {
            Line::from(
                row.into_iter()
                    .map(|cell| match cell {
                        '#' => Span::styled("#", Style::default().fg(Color::DarkGray)),
                        '*' => Span::styled("*", Style::default().fg(Color::Yellow)),
                        'o' => Span::styled("o", Style::default().fg(Color::Red)),
                        '@' => Span::styled(
                            "@",
                            Style::default()
                                .fg(Color::LightYellow)
                                .bg(Color::Red)
                                .add_modifier(Modifier::BOLD),
                        ),
                        'x' => Span::styled("x", Style::default().fg(Color::LightRed)),
                        '.' => Span::styled(".", Style::default().fg(Color::Yellow)),
                        '^' | 'v' | '<' | '>' => Span::styled(
                            cell.to_string(),
                            Style::default()
                                .fg(Color::Green)
                                .add_modifier(Modifier::BOLD),
                        ),
                        _ => Span::raw(" "),
                    })
                    .collect::<Vec<_>>(),
            )
        })
        .collect();

    let board = Paragraph::new(board_lines).block(
        Block::default()
            .title(" Tank Battle TUI ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Gray)),
    );
    frame.render_widget(board, chunks[0]);

    let state = if app.game_over {
        "GAME OVER - R restart, Q quit"
    } else if app.paused {
        "PAUSED - P resume"
    } else {
        "WASD/Arrows move, Space fire, P pause, Q quit"
    };

    let hud = Paragraph::new(vec![
        Line::from(vec![
            Span::styled("Score * ", Style::default().fg(Color::Gray)),
            Span::styled(
                app.score.to_string(),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("   "),
            Span::styled("Lives ", Style::default().fg(Color::Gray)),
            Span::styled(
                hearts(app.lives),
                Style::default()
                    .fg(Color::Red)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("   "),
            Span::styled("Enemies ", Style::default().fg(Color::Gray)),
            Span::styled(
                app.enemies.len().to_string(),
                Style::default().fg(Color::Red),
            ),
        ]),
        Line::from(state),
    ])
    .block(Block::default().borders(Borders::ALL).title(" Status "));

    frame.render_widget(hud, chunks[1]);
}

fn hearts(lives: u8) -> String {
    let full = lives.min(3) as usize;
    let empty = 3usize.saturating_sub(full);
    format!("{}{}", "<3 ".repeat(full), "-- ".repeat(empty))
}
