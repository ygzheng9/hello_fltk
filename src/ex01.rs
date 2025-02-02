use std::{fmt::format, thread, time::Duration};

use fltk::{
    app::{self, channel, Scheme},
    browser::HoldBrowser,
    button::Button,
    dialog::alert_default,
    enums::{Align, CallbackTrigger, Color},
    frame::Frame,
    input::Input,
    menu::Choice,
    output::Output,
    prelude::*,
    valuator::HorSlider,
    window::Window,
};

use chrono::{offset::Local, NaiveDate};

const WIDGET_HEIGHT: i32 = 25;
const WIDGET_PADDING: i32 = 10;
const WIDGET_WIDTH: i32 = 70;

const BOOKING_WIDGET_WIDTH: i32 = 200;

const WIDGET_LABEL_WIDTH: i32 = 100;
const PROGRESS_WIDGET_WIDTH: i32 = 200;
const DURATION_DEFAULT: f64 = 15.0;
const DURATION_MAXIMUM: f64 = 30.0;

#[derive(Clone, Copy)]
enum Message {
    SchemeChanged,

    CelsiusChanged,
    FahrenheitChanged,

    BookingUpdate,
    BookingBook,

    TimerReset,
    TimerChangeDuration,
    TimerTick,
    TimerPauseRunning,

    CrudClear,
    CrudCreate,
    CrudUpdate,
    CrudDelete,
    CrudSelect,
    CrudFilter,
}

#[derive(Clone, Copy)]
enum TimerStatus {
    Running,
    Pause,
}

fn main() {
    let mut app = app::App::default().with_scheme(Scheme::Base);

    let mut timer_status = TimerStatus::Running;

    // 消息
    let (sender, receiver) = channel::<Message>();

    thread::spawn(move || loop {
        // todo: 主程序改變 timerStatus，但是這裡接收不到改變的信息
        match timer_status {
            TimerStatus::Running => {
                thread::sleep(Duration::from_millis(100));
                sender.send(Message::TimerTick);
            }
            _ => {}
        }
    });

    let mut wind = Window::new(100, 100, 400, 900, "Hello from rust");

    // frame 就是 label
    let mut frame = Frame::default()
        .with_size(BOOKING_WIDGET_WIDTH, WIDGET_HEIGHT)
        .with_pos(WIDGET_PADDING, WIDGET_PADDING)
        .with_label("Change Me");

    let mut but = Button::default()
        .with_size(WIDGET_WIDTH, WIDGET_HEIGHT)
        .right_of(&frame, WIDGET_PADDING)
        .with_label("Click Me");

    // 1. output 就是 inputbox 的 subclass
    // 指定大小，指定絕對位置，第一排
    let mut output = Output::default()
        .with_size(WIDGET_WIDTH, WIDGET_HEIGHT)
        .below_of(&frame, WIDGET_PADDING * 2);
    output.set_value("0");

    // 橫向排列
    let mut button = Button::default()
        .with_size(WIDGET_WIDTH, WIDGET_HEIGHT)
        .right_of(&output, WIDGET_PADDING)
        .with_label("Count");

    let mut choice_scheme = Choice::default()
        .with_size(BOOKING_WIDGET_WIDTH, WIDGET_HEIGHT)
        .right_of(&button, WIDGET_PADDING);

    let scheme_list: Vec<Scheme> = vec![
        Scheme::Base,
        Scheme::Plastic,
        Scheme::Gtk,
        Scheme::Gleam,
        Scheme::Oxy,
    ];

    ["Base", "Plastic", "Gtk", "Gleam", "Oxy"]
        .iter()
        .for_each(|s| {
            choice_scheme.add_choice(s);
        });

    choice_scheme.set_value(0);
    choice_scheme.emit(sender, Message::SchemeChanged);

    // 2. 温度转换器，第二排
    let mut celsius_input = Input::default()
        .with_size(WIDGET_WIDTH, WIDGET_HEIGHT)
        .below_of(&output, WIDGET_PADDING * 2);
    // 發送消息
    celsius_input.set_trigger(CallbackTrigger::Changed);
    celsius_input.emit(sender, Message::CelsiusChanged);

    let celsius_frame = Frame::default()
        .with_size(WIDGET_WIDTH, WIDGET_HEIGHT)
        .right_of(&celsius_input, WIDGET_PADDING)
        .with_label("Celsius = ");

    let mut fahrenheit_input = Input::default()
        .with_size(WIDGET_WIDTH, WIDGET_HEIGHT)
        .right_of(&celsius_frame, WIDGET_PADDING);
    fahrenheit_input.set_trigger(CallbackTrigger::Changed);
    fahrenheit_input.emit(sender, Message::FahrenheitChanged);

    let _fahrenheit_frame = Frame::default()
        .with_size(WIDGET_WIDTH, WIDGET_HEIGHT)
        .right_of(&fahrenheit_input, WIDGET_PADDING)
        .with_label("Fahrenheit");

    // 3. booking flight
    let frame3 = Frame::default()
        .with_size(BOOKING_WIDGET_WIDTH, WIDGET_HEIGHT)
        .below_of(&celsius_input, WIDGET_PADDING * 2)
        .with_label("Booking Flight");

    let mut choice = Choice::default()
        .with_size(BOOKING_WIDGET_WIDTH, WIDGET_HEIGHT)
        .below_of(&frame3, WIDGET_PADDING);
    choice.add_choice("one-way flight");
    choice.add_choice("return flight");
    let one_way_flight_index = 0;
    choice.set_value(one_way_flight_index);
    choice.emit(sender, Message::BookingUpdate);

    let current_date = Local::now().naive_local().date();

    let mut start_input = Input::default()
        .with_size(BOOKING_WIDGET_WIDTH, WIDGET_HEIGHT)
        .below_of(&choice, WIDGET_PADDING);
    start_input.set_trigger(CallbackTrigger::Changed);
    start_input.emit(sender, Message::BookingUpdate);
    start_input.set_value(&current_date.format("%Y-%m-%d").to_string());

    let mut return_input = Input::default()
        .with_size(BOOKING_WIDGET_WIDTH, WIDGET_HEIGHT)
        .below_of(&start_input, WIDGET_PADDING);
    return_input.deactivate();
    return_input.set_trigger(CallbackTrigger::Changed);
    return_input.emit(sender, Message::BookingUpdate);
    return_input.set_value(&current_date.format("%Y-%m-%d").to_string());

    let mut book_button = Button::default()
        .with_size(WIDGET_WIDTH, WIDGET_HEIGHT)
        .below_of(&return_input, WIDGET_PADDING)
        .with_label("Book");
    book_button.emit(sender, Message::BookingBook);

    // 4. Timer
    let frame4 = Frame::default()
        .with_size(BOOKING_WIDGET_WIDTH, WIDGET_HEIGHT)
        .below_of(&book_button, WIDGET_PADDING * 2)
        .with_label("Timer");

    let mut elapsed_progress = fltk::misc::Progress::default()
        .with_size(PROGRESS_WIDGET_WIDTH, WIDGET_HEIGHT)
        .with_pos(
            WIDGET_PADDING + WIDGET_LABEL_WIDTH,
            frame4.y() + frame4.h() + WIDGET_PADDING,
        )
        .with_align(Align::Left)
        .with_label("Elapsed Time:");
    elapsed_progress.set_selection_color(Color::Blue);
    elapsed_progress.set_maximum(DURATION_DEFAULT);

    let mut elapsed_frame = Frame::default()
        .with_size(PROGRESS_WIDGET_WIDTH, WIDGET_HEIGHT)
        .below_of(&elapsed_progress, WIDGET_PADDING)
        .with_label("0.0s")
        .with_align(Align::Inside | Align::Left);

    let mut duration_slider = HorSlider::default()
        .with_size(PROGRESS_WIDGET_WIDTH, WIDGET_HEIGHT)
        .below_of(&elapsed_progress, WIDGET_HEIGHT + WIDGET_PADDING * 2)
        .with_align(Align::Left)
        .with_label("Duration:");
    duration_slider.set_value(DURATION_DEFAULT);
    duration_slider.set_maximum(DURATION_MAXIMUM);
    duration_slider.emit(sender, Message::TimerChangeDuration);

    let mut reset_button = Button::default()
        .with_size(WIDGET_LABEL_WIDTH + PROGRESS_WIDGET_WIDTH, WIDGET_HEIGHT)
        .with_pos(
            WIDGET_PADDING,
            duration_slider.y() + duration_slider.h() + WIDGET_PADDING,
        )
        .with_label("Reset");
    reset_button.emit(sender, Message::TimerReset);

    let mut pause_running_button = Button::default()
        .with_size(WIDGET_LABEL_WIDTH + PROGRESS_WIDGET_WIDTH, WIDGET_HEIGHT)
        .below_of(&reset_button, 0)
        .with_label("Pause");
    pause_running_button.emit(sender, Message::TimerPauseRunning);

    // 5. CRUD
    let frame5 = Frame::default()
        .with_size(BOOKING_WIDGET_WIDTH, WIDGET_HEIGHT)
        .below_of(&pause_running_button, WIDGET_PADDING * 2)
        .with_label("CRUD");

    let mut filter_input = Input::default()
        .with_size(WIDGET_WIDTH, WIDGET_HEIGHT)
        .with_pos(
            WIDGET_PADDING + WIDGET_WIDTH * 2,
            frame5.y() + frame5.h() + WIDGET_PADDING,
        )
        .with_label("Filter prefix:");
    filter_input.set_trigger(CallbackTrigger::Changed);
    filter_input.emit(sender, Message::CrudFilter);

    let mut list_browser = HoldBrowser::default()
        .with_pos(
            WIDGET_PADDING,
            filter_input.y() + filter_input.height() + WIDGET_PADDING,
        )
        .with_size(WIDGET_WIDTH * 3, WIDGET_HEIGHT * 4);
    list_browser.emit(sender, Message::CrudSelect);

    let mut name_input = Input::default()
        .with_size(WIDGET_WIDTH, WIDGET_HEIGHT)
        .with_pos(
            list_browser.x() + list_browser.width() + WIDGET_PADDING + WIDGET_WIDTH,
            list_browser.y(),
        )
        .with_label("Name:");

    let mut surname_input = Input::default()
        .with_size(WIDGET_WIDTH, WIDGET_HEIGHT)
        .below_of(&name_input, WIDGET_PADDING)
        .with_label("Surname:");

    let mut btn_clear = Button::default()
        .with_size(WIDGET_WIDTH, WIDGET_HEIGHT)
        .below_of(&surname_input, WIDGET_PADDING)
        .with_label("Clear");
    btn_clear.emit(sender, Message::CrudClear);

    let mut create_button = Button::default()
        .with_size(WIDGET_WIDTH, WIDGET_HEIGHT)
        .with_pos(
            WIDGET_PADDING,
            list_browser.y() + list_browser.height() + WIDGET_PADDING,
        )
        .with_label("Create");
    create_button.emit(sender, Message::CrudCreate);

    let mut update_button = Button::default()
        .with_size(WIDGET_WIDTH, WIDGET_HEIGHT)
        .right_of(&create_button, WIDGET_PADDING)
        .with_label("Update");
    update_button.emit(sender, Message::CrudUpdate);
    update_button.deactivate();

    let mut delete_button = Button::default()
        .with_size(WIDGET_WIDTH, WIDGET_HEIGHT)
        .right_of(&update_button, WIDGET_PADDING)
        .with_label("Delete");
    delete_button.emit(sender, Message::CrudDelete);
    delete_button.deactivate();

    let mut model = vec![
        "Babbage, Charles".to_string(),
        "Lovelace, Ada".to_string(),
        "Turing, Alan".to_string(),
    ];
    sender.send(Message::CrudFilter);

    let formatted_name = {
        let surname_input = surname_input.clone();
        let name_input = name_input.clone();

        move || format!("{}, {}", surname_input.value(), name_input.value())
    };

    wind.end();
    wind.show();

    let mut i = 1;
    but.set_callback(move |_| {
        i = if i == 1 { 0 } else { 1 };

        let msg = match i {
            0 => "Hello World!",
            1 => "Really Amazing",
            _ => "BangBang!",
        };

        frame.set_label_color(Color::Red);
        frame.set_color(Color::Blue);
        frame.redraw();

        frame.set_label(msg);
    });

    let mut value = 0;
    button.set_callback(move |_| {
        value += 1;
        output.set_value(&format!("{}", value));
    });

    // app.run().unwrap();
    while app.wait() {
        // 所有的窗口消息，都會走到這
        match receiver.recv() {
            Some(Message::SchemeChanged) => {
                let idx: usize = choice_scheme.value().try_into().unwrap();
                println!("idx = {}", idx);

                let scheme = scheme_list[idx];
                app.set_scheme(scheme);
                app.redraw();
            }
            Some(Message::CelsiusChanged) => {
                if let Ok(celsius) = celsius_input.value().parse::<i32>() {
                    let v = f64::from(celsius) * (9.0 / 5.0) + 32.0;
                    fahrenheit_input.set_value(&format!("{}", v.round()));
                } else {
                    fahrenheit_input.set_value("");
                }
            }
            Some(Message::FahrenheitChanged) => {
                if let Ok(fahrenheit) = fahrenheit_input.value().parse::<i32>() {
                    let v = (f64::from(fahrenheit) - 32.0) * (5.0 / 9.0);
                    celsius_input.set_value(&format!("{}", v.round()))
                } else {
                    celsius_input.set_value("");
                }
            }
            Some(Message::BookingUpdate) => {
                if choice.value() == one_way_flight_index {
                    return_input.deactivate();
                    if get_date(&mut start_input).is_ok() {
                        book_button.activate();
                    } else {
                        book_button.deactivate();
                    }
                } else {
                    return_input.activate();
                    let start_date = get_date(&mut start_input);
                    let return_date = get_date(&mut return_input);
                    if start_date.is_ok()
                        && return_date.is_ok()
                        && start_date.unwrap() <= return_date.unwrap()
                    {
                        book_button.activate();
                    } else {
                        book_button.deactivate();
                    }
                }
            }
            Some(Message::BookingBook) => {
                let msg = if choice.value() == one_way_flight_index {
                    format!(
                        "You have booked a one-way flight for {}.",
                        start_input.value()
                    )
                } else {
                    format!(
                        "You have booked a return flight from {} to {}",
                        start_input.value(),
                        return_input.value()
                    )
                };

                alert_default(&msg);
            }
            Some(Message::TimerReset) => {
                elapsed_progress.set_value(0.0);
            }
            Some(Message::TimerChangeDuration) => {
                elapsed_progress.set_maximum(duration_slider.value());
            }
            Some(Message::TimerTick) => match timer_status {
                TimerStatus::Running => {
                    if duration_slider.value() - elapsed_progress.value() >= 0.01 {
                        elapsed_progress.set_value(elapsed_progress.value() + 0.1);
                        elapsed_frame.set_label(&format!("{:.1}s", elapsed_progress.value()));
                    }
                }
                TimerStatus::Pause => {}
            },
            Some(Message::TimerPauseRunning) => match timer_status {
                TimerStatus::Running => {
                    timer_status = TimerStatus::Pause;
                    pause_running_button.set_label("Resume");
                }
                TimerStatus::Pause => {
                    timer_status = TimerStatus::Running;
                    pause_running_button.set_label("Pause");
                }
            },
            Some(Message::CrudClear) => {
                update_button.deactivate();
                delete_button.deactivate();

                surname_input.set_value("");
                name_input.set_value("");

                create_button.activate();
            }
            Some(Message::CrudCreate) => {
                model.push(formatted_name());
                sender.send(Message::CrudFilter);

                surname_input.set_value("");
                name_input.set_value("");

                update_button.activate();
                delete_button.activate();
            }
            Some(Message::CrudUpdate) => {
                let selected_name = list_browser.text(list_browser.value()).unwrap();
                let index = model.iter().position(|s| s == &selected_name).unwrap();
                model[index] = formatted_name();
                sender.send(Message::CrudFilter);
            }
            Some(Message::CrudDelete) => {
                let selected_name = list_browser.text(list_browser.value()).unwrap();
                let index = model.iter().position(|s| s == &selected_name).unwrap();
                model.remove(index);
                sender.send(Message::CrudFilter);
                sender.send(Message::CrudSelect)
            }
            Some(Message::CrudSelect) => {
                if list_browser.value() == 0 {
                    update_button.deactivate();
                    delete_button.deactivate();
                } else {
                    let selected_name = list_browser.text(list_browser.value()).unwrap();
                    let arr: Vec<&str> = selected_name.split(',').collect();

                    surname_input.set_value(&arr[0]);
                    name_input.set_value(&arr[1].trim());

                    update_button.activate();
                    delete_button.activate();
                }
            }
            Some(Message::CrudFilter) => {
                let prefix = filter_input.value().to_lowercase();
                list_browser.clear();
                for item in &model {
                    if item.to_lowercase().starts_with(&prefix) {
                        list_browser.add(item);
                    }
                }
                sender.send(Message::CrudSelect);

                create_button.deactivate();
            }
            None => {
                // 窗口會有很多消息，這裡不需要 print！
            }
        }
    }
}

fn get_date(input: &mut Input) -> Result<NaiveDate, chrono::ParseError> {
    let date = NaiveDate::parse_from_str(&input.value(), "%Y-%m-%d");
    input.set_color(match date {
        Ok(_) => Color::BackGround2,
        Err(_) => Color::Red,
    });
    input.redraw();
    date
}
