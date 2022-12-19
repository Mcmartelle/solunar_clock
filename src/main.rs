use chrono::{Date, DateTime, Local, TimeZone, Timelike, Utc};
use circadia::{time_of_event, Event, GlobalPosition, SunEvent, Zenith};
use esbat::{daily_lunar_phase, Phase};
use geodate::{moon_transit, sun_transit};
use iced::{
    alignment,
    canvas::{self, path::Arc, Cache, Canvas, Cursor, Geometry, LineCap, Path, Stroke, Text},
    executor, time, Alignment, Application, Color, Column, Command, Container, Element, Image,
    Length, Point, Rectangle, Settings, Subscription, Vector,
};

pub fn main() -> iced::Result {
    Clock::run(Settings {
        antialiasing: true,
        ..Settings::default()
    })
}

struct Clock {
    now: DateTime<Local>,
    today: Date<Local>,
    clock: Cache,
    moonphase: Phase,
    moonrise: DateTime<Local>,
    moonset: DateTime<Local>,
    nautical_sunrise: DateTime<Local>,
    sunrise: DateTime<Local>,
    noon: DateTime<Local>,
    sunset: DateTime<Local>,
    nautical_sunset: DateTime<Local>,
    midnight: DateTime<Local>,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    Tick(DateTime<Local>),
}

impl Application for Clock {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        let _latitude: f64 = 34.0522; // change to your coordinates.
        let _longitude: f64 = -118.2437; // change to your coordinates, remember that West is negative.
        let _pos = GlobalPosition::at(_latitude, _longitude);
        let _today = Utc::today();
        let _sunrise_nt_utc =
            time_of_event(_today, &_pos, SunEvent::new(Zenith::Civil, Event::Sunrise)).unwrap();
        let _sunrise_nt_local: DateTime<Local> = DateTime::from(_sunrise_nt_utc);
        let _sunrise_utc = time_of_event(_today, &_pos, SunEvent::SUNRISE).unwrap();
        let _sunrise_local: DateTime<Local> = DateTime::from(_sunrise_utc);
        let _sunset_utc = time_of_event(_today, &_pos, SunEvent::SUNSET).unwrap();
        let _sunset_local: DateTime<Local> = DateTime::from(_sunset_utc);
        let _sunset_nt_utc =
            time_of_event(_today, &_pos, SunEvent::new(Zenith::Civil, Event::Sunset)).unwrap();
        let _sunset_nt_local: DateTime<Local> = DateTime::from(_sunset_nt_utc);
        let _moonrise: DateTime<Local> = DateTime::from(Utc.timestamp(
            moon_transit::get_moonrise(Utc::now().timestamp(), _longitude, _latitude).unwrap(),
            0,
        ));
        let _moonset: DateTime<Local> = DateTime::from(Utc.timestamp(
            moon_transit::get_moonset(Utc::now().timestamp(), _longitude, _latitude).unwrap(),
            0,
        ));
        let _noon: DateTime<Local> = DateTime::from(
            Utc.timestamp(sun_transit::get_noon(Utc::now().timestamp(), _longitude), 0),
        );
        let _midnight: DateTime<Local> = DateTime::from(Utc.timestamp(
            sun_transit::get_midnight(Utc::now().timestamp(), _longitude),
            0,
        ));

        (
            Clock {
                now: Local::now(),
                today: Local::today(),
                clock: Default::default(),
                moonphase: daily_lunar_phase(Local::now().date()),
                moonrise: _moonrise,
                moonset: _moonset,
                nautical_sunrise: _sunrise_nt_local,
                sunrise: _sunrise_local,
                noon: _noon,
                sunset: _sunset_local,
                nautical_sunset: _sunset_nt_local,
                midnight: _midnight,
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Solunar Clock")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Tick(local_time) => {
                let now = local_time;

                if local_time.minute() != self.now.minute() {
                    self.now = now;
                    self.clock.clear();
                    if now.date() != self.today {
                        self.today = now.date();
                        self.moonphase = daily_lunar_phase(now.date());
                    }
                }
            }
        }

        Command::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        time::every(std::time::Duration::from_millis(30000))
            .map(|_| Message::Tick(chrono::Local::now()))
    }

    fn view(&mut self) -> Element<Message> {
        let moonpath = match self.moonphase {
            Phase::NewMoon => "resources/new_moon.png",
            Phase::WaxingCrescent => "resources/waxing_crescent_moon.png",
            Phase::FirstQuarter => "resources/first_quarter_moon.png",
            Phase::WaxingGibbous => "resources/waxing_gibbous_moon.png",
            Phase::FullMoon => "resources/full_moon.png",
            Phase::WaningGibbous => "resources/waning_gibbous_moon.png",
            Phase::LastQuarter => "resources/last_quarter_moon.png",
            Phase::WaningCrescent => "resources/waning_crescent_moon.png",
        };
        // println!("Today is {}", self.today.weekday());
        // println!("nautical_sunrise: {}", self.nautical_sunrise.format("%r"));
        // println!("sunrise: {}", self.sunrise.format("%r"));
        // println!("noon: {}", self.noon.format("%r"));
        // println!("sunset: {}", self.sunset.format("%r"));
        // println!("nautical_sunset: {}", self.nautical_sunset.format("%r"));
        // println!("midnight: {}", self.midnight.format("%r"));
        // println!("moonphase: {}", moonpath);
        // println!("moonrise: {}", self.moonrise.format("%r"));
        // println!("moonset: {}", self.moonset.format("%r"));

        let canvas = Canvas::new(self).width(Length::Fill).height(Length::Fill);

        let column = Column::new()
            .padding(20)
            .align_items(Alignment::Center)
            .push(
                Container::new(canvas)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .padding(20)
                    .center_x()
                    .center_y(),
            )
            .push(Image::new(moonpath));

        let main_container = Container::new(column).style(style::Container);
        main_container.into()
    }
}

impl canvas::Program<Message> for Clock {
    fn draw(&self, bounds: Rectangle, _cursor: Cursor) -> Vec<Geometry> {
        // use chrono::Timelike;

        let clock = self.clock.draw(bounds.size(), |frame| {
            let center = frame.center();
            // let background_radius = frame.width().max(frame.height());
            let number_radius = frame.width().min(frame.height()) / 2.1;
            let moon_radius = frame.width().min(frame.height()) / 2.3;
            let sun_radius = frame.width().min(frame.height()) / 2.6;
            let hand_center_radius = frame.width().min(frame.height()) / 36.0;
            let noon_radius = frame.width().min(frame.height()) / 2.56;

            let nautical_sunrise_angle = arc_angle(self.nautical_sunrise);
            let sunrise_angle = arc_angle(self.sunrise);
            let noon_angle = offset_arc_angle(self.noon);
            let sunset_angle = arc_angle(self.sunset);
            let nautical_sunset_angle = arc_angle(self.nautical_sunset);
            let midnight_angle = offset_arc_angle(self.midnight);
            let moonrise_angle = arc_angle(self.moonrise);
            let moonset_angle = arc_angle(self.moonset);

            // let background_background = Path::circle(center, background_radius);
            // frame.fill(&background_background, Color::BLACK);
            let moon_background = Path::circle(center, moon_radius);
            frame.fill(&moon_background, Color::from_rgb8(0x52, 0x4b, 0xb3));

            let moon_stroke = Stroke {
                width: moon_radius - sun_radius + 2.0,
                color: Color::from_rgb8(0x57, 0x96, 0xA1),
                line_cap: LineCap::Butt,
                ..Stroke::default()
            };
            let moon_arc = Path::new(|p| {
                p.arc(Arc {
                    center: center,
                    radius: ((moon_radius + sun_radius) / 2.0) - 1.0,
                    start_angle: moonrise_angle,
                    end_angle: moonset_angle,
                });
            });
            frame.stroke(&moon_arc, moon_stroke);

            let sun_background = Path::circle(center, sun_radius * 1.03);
            frame.fill(&sun_background, Color::from_rgb8(0x47, 0x1b, 0x6e));
            let hand_center_background = Path::circle(center, hand_center_radius);

            let day_stroke = Stroke {
                width: sun_radius,
                color: Color::from_rgb8(0x8B, 0xc7, 0xBF),
                line_cap: LineCap::Butt,
                ..Stroke::default()
            };
            let day_arc = Path::new(|p| {
                p.arc(Arc {
                    center: center,
                    radius: sun_radius / 2.0,
                    start_angle: sunrise_angle - std::f32::consts::PI / 36.0,
                    end_angle: sunset_angle + std::f32::consts::PI / 36.0,
                });
            });

            frame.stroke(&day_arc, day_stroke);

            let twilight_stroke = Stroke {
                width: sun_radius,
                color: Color::from_rgb8(0xDE, 0x8B, 0x6F),
                line_cap: LineCap::Butt,
                ..Stroke::default()
            };

            let sunrise_arc = Path::new(|p| {
                p.arc(Arc {
                    center: center,
                    radius: sun_radius / 2.0,
                    start_angle: nautical_sunrise_angle,
                    end_angle: sunrise_angle,
                });
            });

            frame.stroke(&sunrise_arc, twilight_stroke);

            let sunset_arc = Path::new(|p| {
                p.arc(Arc {
                    center: center,
                    radius: sun_radius / 2.0,
                    start_angle: nautical_sunset_angle,
                    end_angle: sunset_angle,
                });
            });

            frame.stroke(&sunset_arc, twilight_stroke);

            frame.fill(&hand_center_background, Color::from_rgb8(0xEB, 0xD6, 0x94));

            let hand = Path::line(Point::ORIGIN, Point::new(0.0, number_radius));
            let hour_notch =
                Path::line(Point::new(0.0, moon_radius), Point::new(0.0, number_radius));
            let half_hour_notch = Path::line(
                Point::new(0.0, moon_radius),
                Point::new(0.0, (number_radius + moon_radius) / 2.0),
            );
            let quarter_hour_notch = Path::line(
                Point::new(0.0, moon_radius),
                Point::new(0.0, (number_radius + moon_radius * 2.0) / 3.0),
            );
            let noon_notch = Path::line(Point::new(0.0, noon_radius), Point::new(0.0, noon_radius));

            let hand_stroke = Stroke {
                width: sun_radius / 36.0,
                color: Color::from_rgb8(0xEB, 0xD6, 0x94),
                line_cap: LineCap::Round,
                ..Stroke::default()
            };
            let notch_stroke = Stroke {
                width: sun_radius / 30.0,
                color: Color::from_rgb8(0x70, 0x27, 0x82),
                line_cap: LineCap::Round,
                ..Stroke::default()
            };
            let noon_stroke = Stroke {
                width: sun_radius / 20.0,
                color: Color::from_rgb8(0xEB, 0xD6, 0x94),
                line_cap: LineCap::Round,
                ..Stroke::default()
            };

            frame.translate(Vector::new(center.x, center.y));

            frame.with_save(|frame| {
                frame.rotate(noon_angle);
                frame.stroke(&noon_notch, noon_stroke);
            });
            frame.with_save(|frame| {
                frame.rotate(midnight_angle);
                frame.stroke(&noon_notch, noon_stroke);
            });

            for n in 1..=24 {
                frame.with_save(|frame| {
                    frame.rotate(number_angle(n as f32));
                    frame.stroke(&hour_notch, notch_stroke);
                    frame.fill_text(Text {
                        content: n.to_string(),
                        position: Point::new(0.0, number_radius * 1.05),
                        color: Color::WHITE,
                        size: 16.0,
                        horizontal_alignment: alignment::Horizontal::Center,
                        vertical_alignment: alignment::Vertical::Center,
                        ..Text::default()
                    });
                    frame.rotate(number_angle(0.25));
                    frame.stroke(&quarter_hour_notch, notch_stroke);
                    frame.rotate(number_angle(0.25));
                    frame.stroke(&half_hour_notch, notch_stroke);
                    frame.rotate(number_angle(0.25));
                    frame.stroke(&quarter_hour_notch, notch_stroke);
                });
            }

            frame.with_save(|frame| {
                frame.rotate(hand_rotation(
                    self.now.hour() * 60 + self.now.minute(),
                    1440,
                ));
                frame.stroke(&hand, hand_stroke);
            });
        });

        vec![clock]
    }
}

mod style {
    use iced::{container, Color};
    pub struct Container;

    impl container::StyleSheet for Container {
        fn style(&self) -> container::Style {
            container::Style {
                background: Color::from_rgb8(0x36, 0x39, 0x3F).into(),
                text_color: Color::WHITE.into(),
                border_radius: 0.0,
                border_width: 0.0,
                border_color: Color::BLACK,
            }
        }
    }
}

fn hand_rotation(n: u32, total: u32) -> f32 {
    let turns = n as f32 / total as f32;

    2.0 * std::f32::consts::PI * turns
}

fn arc_angle(t: DateTime<Local>) -> f32 {
    let n = t.hour() * 60 + t.minute();
    let total = 1440;
    let turns = n as f32 / total as f32;

    2.0 * std::f32::consts::PI * turns + std::f32::consts::PI / 2.0
}

fn offset_arc_angle(t: DateTime<Local>) -> f32 {
    let n = t.hour() * 60 + t.minute();
    let total = 1440;
    let turns = n as f32 / total as f32;

    2.0 * std::f32::consts::PI * turns
}

fn number_angle(n: f32) -> f32 {
    2.0 * std::f32::consts::PI * n / 24.0
}
