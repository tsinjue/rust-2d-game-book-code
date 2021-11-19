// warn strict grammar
#[warn(clippy::pedantic, clippy::all)]
// warn unused mut
#[warn(unused_mut)]
// warn unused code
#[warn(dead_code)]
use bracket_lib::prelude::*;

// default game screen width
const SCREEN_WIDTH: i32 = 80;

// default game screen height
const SCREEN_HEIGHT: i32 = 50;

// default frame duration: float type
const FRAME_DURATION: f32 = 75.0;

struct Player {
    // x position(line position)
    // default: 0
    x: i32,
    //y position (vertical position)
    y: i32,
    //vertical velocity
    velocity: f32,
}

impl Player {
    //player constructor to initialize instance
    fn new(x: i32, y: i32) -> Self {
        Player {
            // x position of player: a world-space positon
            x,
            // y: vertical position of player in screen
            y,
            //velocity: player's vertical velocity
            velocity: 0.0,
        }
    }

    fn gravity_and_move(&mut self) {
        // Increment gravity
        if self.velocity < 2.0 {
            self.velocity += 0.2;
        }

        // Apply gravity
        self.y += self.velocity as i32;
        if self.y < 0 {
            self.y = 0;
        }

        // Move the player
        self.x += 1;
    }

    fn flap(&mut self) {
        self.velocity = -2.0;
    }

    fn render(&mut self, ctx: &mut BTerm) {
        ctx.set(0, self.y, YELLOW, BLACK, to_cp437('@'));
    }
}

struct Obstacle {
    // world-space to match the player’s world-space x value
    x: i32,
    // obstacle center position
    gap_y: i32,
    // the maximum (obtained via i32::max) of 20 minus the player’s score
    // increasing game difficulty
    size: i32,
}

impl Obstacle {
    fn new(x: i32, score: i32) -> Self {
        let mut random = RandomNumberGenerator::new();
        Obstacle {
            // world-space: x coordination
            x,
            // gap center y position
            gap_y: random.range(10, 40),
            //gap size. smaller when player winning more score
            size: i32::max(2, 20 - score),
        }
    }

    fn render(&mut self, ctx: &mut BTerm, player_x: i32) {
        let screen_x = self.x - player_x;
        let half_size = self.size / 2;

        // Draw the top half of the obstacle
        for y in 0..self.gap_y - half_size {
            ctx.set(screen_x, y, RED, BLACK, to_cp437('|'));
        }

        // Draw the bottom half of the obstacle
        for y in self.gap_y + half_size..SCREEN_HEIGHT {
            ctx.set(screen_x, y, RED, BLACK, to_cp437('|'));
        }
    }

    fn hit_obstacle(&self, player: &Player) -> bool {
        let half_size = self.size / 2;
        let does_x_match = player.x == self.x; // (1)
        let player_above_gap = player.y < self.gap_y - half_size; // (2)
        let player_below_gap = player.y > self.gap_y + half_size;
        does_x_match && (player_above_gap || player_below_gap) // (3)
    }
}

#[derive(Debug)]
enum GameMode {
    Menu,
    Playing,
    End,
}

struct State {
    // player
    player: Player,
    // frame time
    frame_time: f32,
    // obstacle
    obstacle: Obstacle,
    // game mode
    mode: GameMode,
    // player score
    score: i32,
}

impl State {
    // game state constructor to initialize instance
    fn new() -> Self {
        State {
            player: Player::new(5, 25),
            //default frame time
            frame_time: 0.0,
            // obstacle construction
            obstacle: Obstacle::new(SCREEN_WIDTH, 0),
            // default entering menu mode
            mode: GameMode::Menu,
            // default score
            score: 0,
        }
    }

    fn restart(&mut self) {
        // construct new player and make initialization
        self.player = Player::new(5, 25);
        //clear frame time
        self.frame_time = 0.0;
        //construct obstacle
        self.obstacle = Obstacle::new(SCREEN_WIDTH, 0);
        //update game status
        self.mode = GameMode::Playing;
        //clear score
        self.score = 0;
    }

    fn main_menu(&mut self, ctx: &mut BTerm) {
        // clear game window
        ctx.cls();
        // print line(x coordinate) center
        ctx.print_centered(5, "Welcome to Flappy Dragon");
        ctx.print_centered(8, "(P) Play Game");
        ctx.print_centered(9, "(Q) Quit Game");

        if let Some(key) = ctx.key {
            // handle key incident
            match key {
                // restart game
                VirtualKeyCode::P => self.restart(),

                // quit game
                VirtualKeyCode::Q => ctx.quitting = true,

                // other keys: do nothing
                _ => {}
            }
        }
    }

    fn dead(&mut self, ctx: &mut BTerm) {
        //clear window text
        ctx.cls();
        // print center text on vertical y position
        ctx.print_centered(5, "You are dead!");
        ctx.print_centered(6, &format!("You earned {} points", self.score));
        ctx.print_centered(8, "(P) Play Again");
        ctx.print_centered(9, "(Q) Quit Game");

        // deal with key incident
        // if let and match work in the same way
        if let Some(key) = ctx.key {
            match key {
                // restart game
                VirtualKeyCode::P => self.restart(),
                // quit game
                VirtualKeyCode::Q => ctx.quitting = true,
                // do nothing
                _ => {}
            }
        }
    }

    fn play(&mut self, ctx: &mut BTerm) {
        // clear window with specified background color
        ctx.cls_bg(NAVY);

        self.frame_time += ctx.frame_time_ms;


        if self.frame_time > FRAME_DURATION {
            self.frame_time = 0.0;

            self.player.gravity_and_move();
        }
        // press space key to flap
        if let Some(VirtualKeyCode::Space) = ctx.key {
            self.player.flap();
        }

        // update player position information
        self.player.render(ctx);

        // print hint message and player total score
        ctx.print(0, 0, "Press SPACE to flap.");
        ctx.print(0, 1, &format!("Score: {}", self.score)); // (4)

        self.obstacle.render(ctx, self.player.x); // (5)
        if self.player.x > self.obstacle.x {
            // (6)
            self.score += 1;
            self.obstacle = Obstacle::new(self.player.x + SCREEN_WIDTH, self.score);
        }
        if self.player.y > SCREEN_HEIGHT || self.obstacle.hit_obstacle(&self.player) {
            self.mode = GameMode::End;
        }
    }
}

impl GameState for State {
    // must implement tick method
    // &mut self: allows to change game status instance
    // ctx: provide a window into current running bracket-terminal, accessing information like mouse,
    // keyboard etc and sending commands to draw the window
    // short for "context", interacting with game display
    fn tick(&mut self, ctx: &mut BTerm) {
        match self.mode {
            GameMode::Menu => self.main_menu(ctx),
            GameMode::End => self.dead(ctx),
            GameMode::Playing => self.play(ctx),
        }
    }
}

fn main() -> BError {
    //Result.unwrap
    // build 80*50 terminal area
    let context = BTermBuilder::simple80x50()
        .with_title("Flappy Dragon")
        .build()?;

    main_loop(context, State::new())
}
