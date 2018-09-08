extern crate rand;
extern crate sfml;

use rand::prelude::*;
use sfml::graphics::{Color, FloatRect, Font, RectangleShape, RenderTarget, RenderWindow, Shape, Text, Transformable};
use sfml::audio::{Sound, SoundBuffer};
use sfml::system::{Clock, Time, Vector2f, Vector2u};
use sfml::window::{Event, Key, Style};

use std::collections::VecDeque;
use std::error::Error;


/// Game configuration.
pub struct Config {
    window_size: Vector2u,  // window size (width, height)
    entity_size: u32,       // entity size (all entities are square)
    fps: u32,               // frames per second
    text_size: u32,         // score character size
    text_color: Color,      // score text color
    snake_color: Color,     // snake color
    food_color: Color,      // snake food color
}

impl Config {

    /// Initializes the game configuration.
    pub fn new(args: &[String]) -> Result<Config, &'static str> {
        if args.len() < 3 {
            return Err("Invalid number of arguments: <width> <height>");
        }
        let width = args[1].parse::<u32>().expect("The window with must be a u32");
        let height = args[2].parse::<u32>().expect("The window height must be a u32");
        Ok(Config {
            window_size: Vector2u::new(width, height),
            entity_size: 40,
            fps: 10,
            text_size: 55,
            text_color: Color::WHITE,
            snake_color: Color::GREEN,
            food_color: Color::RED,
        })
    }

}


/// Game resources.
struct Resources {
    font: Font,                 // text font
    eat_buffer: SoundBuffer,    // eat sound buffer
    over_buffer: SoundBuffer,   // game over sound buffer
}

impl<'a> Resources {

    /// Loads and initializes the game resources.
    fn new() -> Resources {
        // loaf text font
        let filename = "resources/joystix.ttf";
        let font = Font::from_file(filename).expect("Unable to load the font.");
        // load sound buffer
        let filename = "resources/eat.ogg";
        let eat_buffer = SoundBuffer::from_file(filename).expect("Unable to load the eat sound.");
        let filename = "resources/error.ogg";
        let over_buffer = SoundBuffer::from_file(filename).expect("Unable to load the game over sound.");
        Resources { font, eat_buffer, over_buffer }
    }

}


trait Game {

    /// Runs the game main loop.
    fn run(&mut self);

    /// Handles player inputs.
    fn process_events(&mut self);

    /// Updates the game status.
    /// * `time` - Elapsed time between two consecutive frames.
    fn update(&mut self, time: Time);

    /// Renders graphics.
    fn render(&mut self);

}

trait Graphic {

    /// Draws the graphic element.
    fn draw(&self, window: &mut RenderWindow);

}

/// Enumerates all possible snake directions.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Direction {
    Left,
    Up,
    Right,
    Down
}

impl Direction {

    /// Returns true only if the self direction is opposite to
    /// the give one.
    fn is_opposite_to(&self, other: &Option<Direction>) -> bool {
        match other {
            Some(direction) => {
                match *self {
                    Direction::Left => *direction == Direction::Right,
                    Direction::Up => *direction == Direction::Down,
                    Direction::Right => *direction == Direction::Left,
                    Direction::Down => *direction == Direction::Up,
                }
            },
            None => false
        }
    }

}


/// A single game entity.
struct Entity<'a> {
    shape: RectangleShape<'a>,  // shape of the entity
}

impl<'a> Entity<'a> {

    /// Initializes a new entity with the given size and color.
    fn new(size: u32, position: Vector2f, color: &Color) -> Entity<'a> {
        let mut shape = RectangleShape::new();
        shape.set_fill_color(&color);
        shape.set_size(Vector2f::new(size as f32, size as f32));
        shape.set_position(position);
        Entity { shape }
    }

    /// Gets the position of the entity.
    fn position(&self) -> Vector2f {
        self.shape.position()
    }

    /// Sets the position of the entity.
    fn set_position(&mut self, position: Vector2f) {
        self.shape.set_position(position);
    }

    /// Gets the size of the entity.
    fn size(&self) -> Vector2f {
        self.shape.size()
    }

    /// Gets the area of the segment.
    fn area(&self) -> FloatRect {
        let position = self.position();
        let size = self.size();
        FloatRect::new(position.x, position.y, size.x, size.y)
    }
    
    /// Gets the fill color of the entity.
    fn color(&self) -> Color {
        self.shape.fill_color()
    }
}

impl<'a> Graphic for Entity<'a> {

    /// Draws the entity.
    fn draw(&self, window: &mut RenderWindow) {
        window.draw(&self.shape);
    }

}


/// The snake.
struct Snake<'a> {
    segments: VecDeque<Entity<'a>>,     // snake segments
    direction: Option<Direction>,       // snake current direction
    next_direction: Option<Direction>,  // snake next direction
}

impl<'a> Snake<'a> {

    /// Creates a new snake with a single segment.
    fn new(position: Vector2f, size: u32, color: &Color) -> Snake<'a> {
        let mut segments = VecDeque::new();
        // create snake head
        let head = Entity::new(size, position, color);
        segments.push_back(head);
        Snake { segments, direction: None, next_direction: None }
    }

    /// Gets the position of the snake head.
    fn head_position(&self) -> Vector2f {
        // the snake has always at least 1 segment
        self.segments.front().unwrap().position()
    }

    /// Gets the size of each snake segment.
    fn size(&self) -> Vector2f {
        // the snake has always at least 1 segment
        self.segments.front().unwrap().size()
    }

    /// Gets the area of the snake head.
    fn area(&self) -> FloatRect {
        self.segments.front().unwrap().area()
    }

    /// Gets the fill color of each snake segment.
    fn color(&self) -> Color {
        // the snake has always at least 1 segment
        self.segments.front().unwrap().color()
    }

    /// Returns true if the snake head collided with any
    /// of its segments.
    fn self_collision(&self) -> bool {
        // check collision between the head (first segment) and
        // all the followings elements
        self.collision(&self.area(), 1)
    }

    /// Returns true only if the given area collides with any of the
    /// snake segments starting from the `n_skip`th one.
    fn collision(&self, area: &FloatRect, n_skip: usize) -> bool {
        // check the snake segments starting from the `n_skip`th
        for segment in self.segments.iter().skip(n_skip) {
            let seg_position = segment.position();
            let seg_area = FloatRect::new(seg_position.x, seg_position.y, area.width, area.height);
            match area.intersection(&seg_area) {
                Some(_) => return true,
                None => ()
            };
        }
        false
    }

    /// Adds a new segment to the end of the snake.
    fn grow(&mut self) {
        // create a new segment and init with the same position of the last segment
        let segment = Entity::new(self.size().x as u32, self.head_position(), &self.color());
        self.segments.push_back(segment);
    }

    /// Removes all the segments but the head.
    fn reset(&mut self) {
        while self.segments.len() > 1 {
            self.segments.pop_back();
        }
        self.direction = None;
        self.next_direction = None;
    }

    /// Updates the snake position.
    fn advance(&mut self, window_size: Vector2u) {
        // update direction
        self.direction = self.next_direction;
        let window_size = Vector2f::new(window_size.x as f32, window_size.y as f32);
        let front_position = self.head_position();
        let size = self.size().x; // it's a square => x == y
        // the snake has always at least 1 segment
        let mut last = self.segments.pop_back().unwrap();
        let back_position = last.position();
        // move the last segment to the new position of the first segment
        // the old tail becomes the new head, gives the "illusion" of movement
        // the environment is implemented as a Toroid
        // https://en.wikipedia.org/wiki/Toroid
        last.set_position(match self.direction {
            Some(Direction::Left) => {
                Vector2f::new((front_position.x - size + window_size.x) % window_size.x, front_position.y)
            },
            Some(Direction::Up) => {
                Vector2f::new(front_position.x, (front_position.y - size + window_size.y) % window_size.y)
            },
            Some(Direction::Right) => {
                Vector2f::new((front_position.x + size) % window_size.x, front_position.y)
            },
            Some(Direction::Down) => {
                Vector2f::new(front_position.x, (front_position.y + size) % window_size.y)
            },
            _ => back_position
        });
        // the last segment is now the first
        self.segments.push_front(last);
    }
}

impl<'a> Graphic for Snake<'a> {

    /// Draws all the snake segments.
    fn draw(&self, window: &mut RenderWindow) {
        for segment in &self.segments {
            segment.draw(window);
        }
    }

}


struct SnakeGame<'a> {
    window: RenderWindow,
    player: Snake<'a>,
    food: Entity<'a>,
    time_per_frame: Time,
    entity_size: u32,
    score: u32,
    score_text: Text<'a>,
    eat_sound: Sound<'a>,
    over_sound: Sound<'a>,
}

impl<'a> SnakeGame<'a> {

    /// Create a new Snake Game.
    fn new(config: &Config, resources: &'a Resources) -> SnakeGame<'a> {
        // window size multiple of entity_size
        let window_size = Vector2u::new(
            config.window_size.x - config.window_size.x % config.entity_size,
            config.window_size.y - config.window_size.y % config.entity_size);
        // create the window
        let mut window = RenderWindow::new(
            (window_size.x, window_size.y),
            "Snake",
            Style::CLOSE,
            &Default::default());
        // set frame limit
        let time_per_frame = Time::seconds(1.0 / config.fps as f32);
        window.set_framerate_limit(config.fps);

        // initialize the score
        let score = 0;
        let mut score_text = Text::default();
        score_text.set_font(&resources.font);
        score_text.set_character_size(config.text_size);
        score_text.set_position(((window_size.x - config.text_size) as f32, 0.0));
        score_text.set_fill_color(&config.text_color);
        score_text.set_string(&score.to_string());

        // init the audio
        let eat_sound = Sound::with_buffer(&resources.eat_buffer);
        let over_sound = Sound::with_buffer(&resources.over_buffer);

        // initialize the snake
        let player_position = SnakeGame::random_position(window_size, config.entity_size);
        let player = Snake::new(player_position, config.entity_size, &config.snake_color);
        // initialize the food
        let food_position = SnakeGame::random_position(window_size, config.entity_size);
        let food = Entity::new(config.entity_size, food_position, &config.food_color);

        SnakeGame {
            window,
            player,
            food,
            time_per_frame,
            entity_size: config.entity_size,
            score,
            score_text,
            eat_sound,
            over_sound,
        }
    }

    /// Returns a random position within the window that is a multiple
    /// of the given entity_size.
    fn random_position(window_size: Vector2u, entity_size: u32) -> Vector2f {
        let mut rng = thread_rng();
        let x = rng.gen_range(0, window_size.x);
        let x = x - x % entity_size;
        let y = rng.gen_range(0, window_size.y);
        let y = y - y % entity_size;
        Vector2f::new(x as f32, y as f32)
    }

    /// Handles the player input.
    fn handle_input(&mut self, key: Key) {
        let key_direction = || {
            match key {
                Key::A => Some(Direction::Left),
                Key::W => Some(Direction::Up),
                Key::D => Some(Direction::Right),
                Key::S => Some(Direction::Down),
                _ => None
            }
        };
        match key_direction() {
            Some(direction) => {
                // check if going backwards is allowed
                if self.player.segments.len() == 1 || !direction.is_opposite_to(&self.player.direction) {
                    self.player.next_direction = Some(direction);
                }
            },
            None => ()
        };
    }

    /// Reset the game to its original state.
    fn reset_state(&mut self) {
        // reset snake
        self.player.reset();
        self.set_score(0);
    }

    /// Increase player score.
    fn set_score(&mut self, value: u32) {
        // get the number of decimal digits
        let digit_count = |mut n: u32| {
            let mut count = 1;
            while n / 10 != 0 {
                count += 1;
                n = n / 10;
            }
            count
        };
        self.score = value;
        // update score position and text
        let offset = digit_count(self.score) * self.score_text.character_size();
        self.score_text.set_position(((self.window.size().x - offset) as f32, 0.0));
        self.score_text.set_string(&self.score.to_string());
    }

}

impl<'a> Game for SnakeGame<'a> {

    /// Runs the game.
    fn run(&mut self) {
        println!("Hello from Snake!");
        let mut clock = Clock::start();
        let mut time_since_last_update = Time::ZERO;
        // run main loop
        while self.window.is_open() {
            self.process_events();
            time_since_last_update += clock.restart();
            let tpf = self.time_per_frame;
            // fixed time steps
            while time_since_last_update > tpf {
                time_since_last_update -= tpf;
                self.process_events();
                self.update(tpf);
            }
            self.render();
        }
    }

    /// Processes the window events.
    fn process_events(&mut self) {
        while let Some(event) = self.window.poll_event() {
            match event {
                Event::Closed => self.window.close(),
                Event::KeyPressed { code, .. } => self.handle_input(code),
                _ => ()
            };
        }
    }

    /// Update the game state.
    fn update(&mut self, _time: Time) {
        // update the player position
        let window_size = self.window.size();
        self.player.advance(window_size);
        // check collision with itself
        if self.player.self_collision() {
            self.reset_state();
            self.over_sound.play();
        } else {
            // check collision with food
            match self.player.area().intersection(&self.food.area()) {
                Some(_) => { 
                    // increase snake length
                    self.player.grow();
                    // update food position
                    let mut food_position = SnakeGame::random_position(window_size, self.entity_size);
                    let mut food_area = FloatRect::new(
                        food_position.x, food_position.y,
                        self.entity_size as f32, self.entity_size as f32);
                    // try a new position if the new one collides with the snake
                    while self.player.collision(&food_area, 0) {
                        food_position = SnakeGame::random_position(window_size, self.entity_size);
                        food_area.left = food_position.x;
                        food_area.top = food_position.y;
                    }
                    self.food.set_position(food_position);
                    // increase score
                    let new_score = self.score + 10;
                    self.set_score(new_score);
                    self.eat_sound.play();
                },
                None => ()
            };
        }
    }

    /// Draws all the game entities.
    fn render(&mut self) {
        self.window.clear(&Color::BLACK);
        // draw entities
        self.food.draw(&mut self.window);
        self.player.draw(&mut self.window);
        self.window.draw(&mut self.score_text);
        self.window.display();
    }

}

/// Runs the Snake game.
pub fn run(config: Config) -> Result<(), Box<Error>> {
    let resources = Resources::new();
    let mut game = SnakeGame::new(&config, &resources);
    game.run();
    Ok(())
}
