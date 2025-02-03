use std::{cell::RefCell, rc::Rc, thread, time::Duration};

use fltk::{
    app::{self, channel, Scheme},
    browser::HoldBrowser,
    button::*,
    dialog::alert_default,
    draw,
    enums::{self, Align, CallbackTrigger, Color, FrameType},
    frame::Frame,
    group::{Flex, Pack, PackType, Tabs},
    input::Input,
    menu::{Choice, MenuButton},
    output::Output,
    prelude::*,
    valuator::HorSlider,
    widget::{self, Widget},
    widget_extends,
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

    let mut wind = Window::new(100, 100, 400, 1200, "Hello from rust");

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

    draw_gallery(delete_button.h() + delete_button.y());

    wind.end();
    wind.show();

    show_frame();

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
            // 窗口會有很多消息，這裡不需要 print！
            None => (),
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

fn draw_gallery(y_pos: i32) {
    let mut tab = Tabs::default()
        .with_size(BOOKING_WIDGET_WIDTH + 20, BOOKING_WIDGET_WIDTH * 2)
        .with_pos(WIDGET_PADDING, y_pos + WIDGET_PADDING);

    let mut grp1 = Flex::default_fill().with_label("Tab1\t\t").row();
    let mut col = Pack::default();
    grp1.fixed(&col, 200);
    grp1.set_margin(10);
    col.set_spacing(5);

    let _but1 = Button::default()
        .with_size(WIDGET_WIDTH, WIDGET_HEIGHT)
        .with_label("Button");

    // 单选框，必须要有一个 Group
    let mut pack1: Pack = Pack::default().with_size(200, WIDGET_HEIGHT);
    pack1.set_spacing(5);

    let _but11 = RadioRoundButton::default()
        .with_size(55, WIDGET_HEIGHT)
        .with_label("radio1");
    let _but12 = RadioRoundButton::default()
        .with_size(55, WIDGET_HEIGHT)
        .with_label("radio2");
    let _but13 = RadioRoundButton::default()
        .with_size(55, WIDGET_HEIGHT)
        .with_label("radio3");

    pack1.end();
    pack1.set_type(PackType::Horizontal);

    // RoundButton, CheckButton 都是 LightButton 的 subclass，主打一个 on 的状态，没有 Group 的概念
    let _but21 = RoundButton::default()
        .with_size(WIDGET_WIDTH, WIDGET_HEIGHT)
        .with_label("Round1");

    // todo: 把 label 显示在左边
    let _but31 = CheckButton::default()
        .with_size(WIDGET_WIDTH, WIDGET_HEIGHT)
        .with_pos(WIDGET_PADDING + WIDGET_LABEL_WIDTH, 0)
        .with_align(Align::Left)
        .with_label("Check1");

    let _but4 = LightButton::default()
        .with_size(WIDGET_WIDTH, WIDGET_HEIGHT)
        .with_label("Light");

    // 对应 回车键
    let _but6 = ReturnButton::default()
        .with_size(WIDGET_WIDTH, WIDGET_HEIGHT)
        .with_label("Return");

    let _dummy1 = Frame::default();
    col.end();
    col.resizable(&_dummy1);
    grp1.end();

    let mut grp2 = Flex::default_fill().with_label("Tab2\t\t").row();
    let mut col2 = Pack::default();
    grp2.fixed(&col2, 200);
    grp2.set_margin(10);

    col2.set_spacing(10);

    let mut but5 = MenuButton::default()
        .with_size(WIDGET_WIDTH, WIDGET_HEIGHT)
        .with_label("Menu");
    but5.add_choice("Hello|World|From|Rust");

    let mut chce = Choice::default().with_size(WIDGET_WIDTH, WIDGET_HEIGHT);
    chce.add_choice("Hello|FLTK|From|Rust");

    let _inp = Input::default().with_size(WIDGET_WIDTH, WIDGET_HEIGHT);
    let mut out = Output::default();
    out.set_value("output");

    col2.resizable(&out);
    col2.end();
    grp2.end();

    let mut grp3 = Flex::default_fill().with_label("Tab3\t\t").row();
    let mut col3 = Flex::default().column();
    grp3.fixed(&col3, 200);
    grp3.set_margin(10);

    let mut btn_custom = MyCustomButton::new(50, "Click");
    btn_custom.set_color(enums::Color::Cyan);

    let mut out1 = Output::default();
    out1.set_value(&format!("{}", btn_custom.num_clicks()));

    // btn.set_callback(move |_| {
    //     println!("Clicked");

    //     // todo: 如何把 参数 转成 MyCustomButton 类型
    //     // out1.set_value(&format!("{}", b.num_clicks()));
    // });

    col3.end();
    grp3.end();

    tab.end();
    tab.auto_layout();

    col3.handle(move |_, ev| match ev {
        enums::Event::Push => {
            out1.set_value(&format!("Got {}", btn_custom.num_clicks()));
            true
        }
        _ => false,
    });
}

// 自定义的 widget，不响应 event
struct MyFrame {
    #[allow(dead_code)]
    f: Frame,
}

impl MyFrame {
    pub fn new(idx: usize) -> MyFrame {
        let mut f = Frame::default();
        // Normally you would use the FrameType enum, for example:
        // some_widget.set_frame(FrameType::DownBox);
        f.set_frame(FrameType::by_index(idx));
        f.set_color(Color::from_u32(0x7FFFD4));
        let f_name = format!("{:?}", f.frame());
        f.set_label(&f_name);
        f.set_label_size(12);
        Self { f }
    }
}

fn show_frame() {
    let mut win = Window::default()
        .with_size(1000, 800)
        .with_label("Frames")
        .center_screen();

    let mut col = Flex::default_fill().column();
    col.set_margin(20);

    let mut row = Flex::default();
    col.fixed(&row, 75);
    for i in 0..8 {
        let _ = MyFrame::new(i);
    }
    row.end();
    row.set_pad(10);

    let mut row = Flex::default();
    col.fixed(&row, 75);
    for i in 8..17 {
        let _ = MyFrame::new(i);
    }
    row.end();
    row.set_pad(10);

    let mut row = Flex::default();
    col.fixed(&row, 75);
    for i in 17..26 {
        let _ = MyFrame::new(i);
    }
    row.end();
    row.set_pad(10);

    let mut row = Flex::default();
    col.fixed(&row, 75);
    for i in 26..35 {
        let _ = MyFrame::new(i);
    }
    row.end();
    row.set_pad(10);

    let mut row = Flex::default();
    col.fixed(&row, 75);
    for i in 35..44 {
        let _ = MyFrame::new(i);
    }
    row.end();
    row.set_pad(10);

    let mut row = Flex::default();
    col.fixed(&row, 75);
    for i in 44..53 {
        let _ = MyFrame::new(i);
    }
    row.end();
    row.set_pad(10);

    col.end();
    col.set_pad(30);

    win.end();
    win.show();
    win.set_color(Color::White);
}

struct MyCustomButton {
    inner: Widget,
    num_clicks: Rc<RefCell<i32>>,
}

impl MyCustomButton {
    // our constructor
    pub fn new(radius: i32, label: &str) -> Self {
        let mut inner = Widget::default()
            .with_size(radius * 2, radius * 2)
            .with_label(label)
            .center_of_parent();
        inner.set_frame(enums::FrameType::OFlatBox);

        inner.draw(|i| {
            // we need a draw implementation
            draw::draw_box(i.frame(), i.x(), i.y(), i.w(), i.h(), i.color());
            draw::set_draw_color(enums::Color::Black); // for the text
            draw::set_font(enums::Font::Helvetica, app::font_size());
            draw::draw_text2(&i.label(), i.x(), i.y(), i.w(), i.h(), i.align());
        });

        let num_clicks = 0;
        let num_clicks = Rc::from(RefCell::from(num_clicks));

        inner.handle({
            let clicks = num_clicks.clone();

            move |i, ev| match ev {
                enums::Event::Push => {
                    *clicks.borrow_mut() += 1; // increment num_clicks

                    i.set_label(&format!("Clicked: {}", *clicks.borrow()));

                    i.do_callback(); // do the callback which we'll set using set_callback().
                    true
                }
                _ => false,
            }
        });

        Self { inner, num_clicks }
    }

    // get the times our button was clicked
    pub fn num_clicks(&self) -> i32 {
        *self.num_clicks.borrow()
    }
}

// Extend widget::Widget via the member `inner` and add other initializers and constructors
widget_extends!(MyCustomButton, widget::Widget, inner);
