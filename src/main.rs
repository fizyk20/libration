use std::{
    f64::consts::PI,
    time::{Duration, Instant},
};

use iced::{
    canvas::{event::Status, Canvas, Cursor, Event, Fill, Frame, Geometry, Path, Program, Stroke},
    executor, keyboard, time, Application, Color, Command, Element, Length, Point, Rectangle,
    Subscription, Vector,
};

const EARTH_RADIUS: f32 = 5.0;
const MOON_RADIUS: f32 = 1.5;
const MOON_ORBIT_RADIUS: f64 = 40.0;
const MOON_COLOR: Color = Color {
    r: 0.7,
    g: 0.7,
    b: 0.7,
    a: 1.0,
};

#[derive(Debug, Clone, Copy)]
enum Message {
    Tick,
}

#[derive(Debug, Clone, Copy)]
struct Libration {
    playing: bool,
    scale: f64,
    time: f64,
    period: f64,
    eccentricity: f64,
    last_tick: Option<Instant>,
}

impl Application for Libration {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        (
            Libration {
                playing: false,
                scale: 100.0,
                time: 0.0,
                period: 8.0,
                eccentricity: 0.0,
                last_tick: None,
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        "Libracja Księżyca".to_string()
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Tick if self.playing => {
                let now = Instant::now();
                if let Some(last_tick) = self.last_tick {
                    let time_diff = ((now - last_tick).as_millis() as f64) / 1000.0;
                    self.time += time_diff / self.period;
                    while self.time > 1.0 {
                        self.time -= 1.0;
                    }
                }
                self.last_tick = Some(now);
            }
            _ => (),
        }
        Command::none()
    }

    fn view(&mut self) -> Element<Message> {
        Canvas::new(self)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn subscription(&self) -> Subscription<Message> {
        time::every(Duration::from_millis(30)).map(|_| Message::Tick)
    }
}

impl Program<Message> for Libration {
    fn draw(&self, bounds: Rectangle<f32>, _cursor: Cursor) -> Vec<Geometry> {
        let mut frame = Frame::new(bounds.size());

        let smaller_dim = if bounds.size().width < bounds.size().height {
            bounds.size().width
        } else {
            bounds.size().height
        };

        frame.translate(frame.center() - Point::new(0.0, 0.0));
        frame.scale(smaller_dim / self.scale as f32);

        let earth = Path::circle(Point::new(0.0, 0.0), EARTH_RADIUS);
        frame.fill(
            &earth,
            Fill {
                color: Color::new(0.0, 1.0, 1.0, 1.0),
                ..Default::default()
            },
        );

        self.draw_moon_orbit(&mut frame);

        self.draw_moon(&mut frame);

        vec![frame.into_geometry()]
    }

    fn update(
        &mut self,
        event: Event,
        _bounds: Rectangle<f32>,
        _cursor: Cursor,
    ) -> (Status, Option<Message>) {
        match event {
            Event::Keyboard(keyboard::Event::KeyPressed {
                key_code: keyboard::KeyCode::Space,
                ..
            }) => {
                self.playing = !self.playing;
                if !self.playing {
                    self.last_tick = None;
                }
            }
            Event::Keyboard(keyboard::Event::KeyPressed {
                key_code: keyboard::KeyCode::E,
                ..
            }) => {
                self.eccentricity += 0.1;
                if self.eccentricity > 0.99 {
                    self.eccentricity = 0.99;
                }
            }
            Event::Keyboard(keyboard::Event::KeyPressed {
                key_code: keyboard::KeyCode::Q,
                ..
            }) => {
                self.eccentricity -= 0.1;
                if self.eccentricity < 0.0 {
                    self.eccentricity = 0.0;
                }
            }
            Event::Keyboard(keyboard::Event::KeyPressed {
                key_code: keyboard::KeyCode::Z,
                ..
            }) => {
                self.scale /= 1.1;
            }
            Event::Keyboard(keyboard::Event::KeyPressed {
                key_code: keyboard::KeyCode::X,
                ..
            }) => {
                self.scale *= 1.1;
            }
            _ => (),
        }
        (Status::Ignored, None)
    }
}

impl Libration {
    fn r(&self, p: f64, phi: f64) -> f64 {
        p / (1.0 + self.eccentricity * phi.cos())
    }

    fn rphi_to_xy(r: f64, phi: f64) -> (f32, f32) {
        ((-r * phi.cos()) as f32, (r * phi.sin()) as f32)
    }

    fn moon_pos(&self) -> Vector<f32> {
        let mut mean_anomaly = self.time * 2.0 * PI;
        let ecc = self.eccentricity;

        if mean_anomaly > PI {
            mean_anomaly -= 2.0 * PI;
        }

        let f = |ecc_anom: f64| ecc_anom - ecc * ecc_anom.sin() - mean_anomaly;
        let df = |ecc_anom: f64| 1.0 - ecc * ecc_anom.cos();

        let mut ecc_anom = mean_anomaly;
        while (f(ecc_anom) / df(ecc_anom)).abs() > 1e-10 {
            ecc_anom -= f(ecc_anom) / df(ecc_anom);
        }

        let true_anom = ((1.0 - ecc * ecc).sqrt() * ecc_anom.sin()).atan2(ecc_anom.cos() - ecc);

        let r = self.r(MOON_ORBIT_RADIUS, true_anom);
        let (x, y) = Self::rphi_to_xy(r, true_anom);

        Vector::new(x, y)
    }

    fn draw_moon_orbit(&self, frame: &mut Frame) {
        let mut phi = 0.0;
        while phi < 2.0 * PI {
            let r = self.r(MOON_ORBIT_RADIUS, phi);
            let (x, y) = Self::rphi_to_xy(r, phi);
            let old_point = Point::new(x, y);
            phi += 0.01;
            let r = self.r(MOON_ORBIT_RADIUS, phi);
            let (x, y) = Self::rphi_to_xy(r, phi);
            let new_point = Point::new(x, y);
            let path = Path::line(old_point, new_point);
            frame.stroke(&path, Stroke::default().with_color(MOON_COLOR));
        }
    }

    fn draw_moon(&self, frame: &mut Frame) {
        frame.with_save(|frame| {
            frame.translate(self.moon_pos());
            let moon = Path::circle(Point::new(0.0, 0.0), MOON_RADIUS);
            frame.fill(
                &moon,
                Fill {
                    color: MOON_COLOR,
                    ..Default::default()
                },
            );
        });
    }
}

fn main() {
    Libration::run(Default::default()).expect("should run successfully");
}
