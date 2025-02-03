use std::fs::File;
use std::io::BufReader;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use fltk::prelude::{InputExt, WidgetBase, WidgetExt};
use fltk::{
    app,
    app::MouseWheel,
    button::Button,
    enums,
    enums::{Color, Event, Key},
    frame::Frame,
    group,
    group::Flex,
    input,
    input::IntInput,
    prelude::*,
    window::Window,
};

use rodio::{source::Source, Decoder, OutputStream};

const WINDOW_WIDTH: i32 = 300;
const WINDOW_HEIGHT: i32 = 140;
pub const EXPANDED_WINDOW_HEIGHT: i32 = 190;

#[derive(Debug, Copy, Clone)]
pub enum ChannelMessage {
    StartCounter(Duration),

    UpdateCountdown(u32, bool),

    PauseCountdown,
    StopCountdown,

    StartClicked,
}

#[derive(Debug, Copy, Clone)]
enum State {
    Start,
    Pause,
    Resume(u64),
}

fn to_minutes_seconds(countdown: u32) -> (String, String) {
    let mut seconds = countdown;
    let mut minutes = 0;

    if seconds >= 60 {
        minutes = seconds / 60;
    }

    seconds = seconds - (minutes * 60);

    let mut minutes_string = minutes.to_string();
    if minutes < 10 {
        minutes_string = "0".to_string() + &minutes.to_string();
    }

    let mut seconds_string = seconds.to_string();
    if seconds < 10 {
        seconds_string = "0".to_string() + &seconds.to_string();
    }

    (minutes_string, seconds_string)
}

pub fn play_sound() {
    std::thread::spawn(|| {
        let alarm_file_path = "./assets/default_alarm.wav".to_owned();
        let (_stream, stream_handle) = OutputStream::try_default().unwrap();

        // done: BufReader 和 File 都没有 clone
        for _ in 1..=10 {
            let file = BufReader::new(File::open(alarm_file_path.clone()).unwrap());

            let source = Decoder::new(file).unwrap();
            stream_handle
                .play_raw(source.convert_samples())
                .expect("Failed to play audio");

            std::thread::sleep(std::time::Duration::from_millis(167));
        }
    });
}

fn update_countdown(frame: &mut Frame, countdown: u32, update_background: bool) {
    let (minutes, seconds) = to_minutes_seconds(countdown);

    frame.set_label(&format!("{}:{}", minutes, seconds));

    if update_background == false {
        return;
    }

    if countdown > 20 {
        frame.set_label_color(Color::White);
    } else if countdown >= 10 {
        frame.set_label_color(Color::Blue);
    } else if countdown > 0 {
        frame.set_label_color(Color::Red);
    } else {
        play_sound();
    }
}

fn style_input_fields(input: &mut input::IntInput) {
    input.set_color(Color::DarkMagenta);
    input.set_text_color(Color::White);
    input.set_selection_color(Color::Blue);
    input.set_text_size(22);
}

// DONE: 状态改变
// 1. start, set;
// 2. start -> pause -> resume; start + set -> stop;
// 3. stop/set -> disable + 显示 input；

// todo: 如果当 button 用，而不是先取得 member
struct StartButton {
    button: Button,
}

impl StartButton {
    fn new(
        input_minutes: &IntInput,
        input_seconds: &IntInput,
        window: &Window,
        flex: &Flex,
        thread_tx: mpsc::Sender<ChannelMessage>,
        tx: app::Sender<ChannelMessage>,
        remain_millis: Arc<Mutex<u64>>,
    ) -> Self {
        let mut button = Button::new(0, 0, 0, 0, "Start");
        let mut state = State::Pause;

        button.set_color(Color::Blue);
        button.set_frame(enums::FrameType::PlasticThinUpBox);

        button.set_label_color(Color::Black);
        button.set_label_font(enums::Font::HelveticaBold);
        button.set_label_size(18);

        // 供 callback 使用；如果 new 的参数不是 ref，那就需要在调用 new 的地方 clone，这里就不需要再 clone；
        let input_mm = input_minutes.clone();
        let input_ss = input_seconds.clone();
        let mut window_clone = window.clone();
        let mut flex_clone = flex.clone();

        button.set_callback(move |_button| {
            _button.set_color(Color::Blue);

            if _button.label() == "Start" {
                state = State::Start;
            }

            // 只要点击了，就窗口变小，隐藏输入框
            flex_clone.hide();
            window_clone.set_size(WINDOW_WIDTH, WINDOW_HEIGHT);
            window_clone.set_color(Color::Black);

            tx.send(ChannelMessage::StartClicked);

            // 1. start -> (pause -> resume)，2. set 按钮，才会重置成 start
            match state {
                State::Start => {
                    _button.set_label("Pause");
                    state = State::Pause;

                    _button.set_color(Color::Green);

                    let duration = Duration::from_secs(
                        (input_mm.value().parse::<i32>().unwrap() * 60
                            + input_ss.value().parse::<i32>().unwrap())
                            as u64,
                    );

                    // 通知线程 计时开始
                    thread_tx
                        .send(ChannelMessage::StartCounter(duration))
                        .expect("Failed to start the counter.");
                }

                State::Pause => {
                    // 状态可附加数据
                    state = State::Resume(*(remain_millis.lock().unwrap()));

                    _button.set_label("Resume");

                    // 发送消息
                    thread_tx
                        .send(ChannelMessage::PauseCountdown)
                        .expect("Failed to get stop countdown message");

                    return;
                }

                State::Resume(mills) => {
                    state = State::Pause;

                    println!("resume: the counter left: {}", mills);

                    _button.set_label("Pause");
                    _button.set_color(Color::Green);

                    // 发送消息, resume 和 start 完全一样，因为 剩余时间是全局变量
                    thread_tx
                        .send(ChannelMessage::StartCounter(Duration::from_millis(mills)))
                        .expect("Failed to get stop countdown message");
                }
            }
        });

        Self { button }
    }
}

// todo: 带有状态 button
struct ResetButton {
    input_show: bool,
    button: Button,
}

// todo: 参数 &impl WindowExt 则不行；
impl ResetButton {
    fn new(
        start_button: &Button,
        window: &Window,
        flex: &Flex,
        thread_tx: mpsc::Sender<ChannelMessage>,
    ) -> Self {
        let mut button = Button::new(0, 0, 0, 0, "Set");

        button.set_color(Color::Red);
        button.set_frame(enums::FrameType::PlasticThinUpBox);

        button.set_label_color(Color::Black);
        button.set_label_font(enums::Font::HelveticaBold);
        button.set_label_size(18);

        let mut start_button = start_button.clone();
        let mut local_window = window.clone();
        let mut flex = flex.clone();

        button.set_callback(move |_button| {
            // 只把 按钮文本 改了，并没有 改变按钮的 状态；向 channel 发送消息改变；
            start_button.set_label("Start");
            start_button.set_color(Color::Blue);

            // 把自己 disable 掉
            _button.deactivate();
            _button.set_color(Color::Gray0);

            // 如果 hide 再 show，布局会乱掉
            // _button.hide();

            // 显示下方的 输入框
            flex.show();
            local_window.set_size(WINDOW_WIDTH, EXPANDED_WINDOW_HEIGHT);

            thread_tx
                .send(ChannelMessage::StopCountdown)
                .expect("Failed to get stop countdown message");
        });

        Self {
            input_show: false,
            button,
        }
    }
}

struct InputDeviceEvent {}

impl InputDeviceEvent {
    pub fn new(
        window: &mut Window,
        start_button: &Button,
        input_minutes: &IntInput,
        input_seconds: &IntInput,
    ) {
        let mut mm_input = input_minutes.clone();
        let mut ss_input = input_seconds.clone();

        // 这是一个内部函数，closure
        let change_countdown = move |inc_mm: i32, inc_ss: i32| {
            let mut new_mm = mm_input.value().parse::<i32>().unwrap() + inc_mm;

            if new_mm < 0 {
                new_mm = 0;
            }
            if new_mm > 99 {
                new_mm = 99;
            }
            mm_input.set_value(format!("{}", new_mm).as_str());

            let mut new_ss = ss_input.value().parse::<i32>().unwrap() + inc_ss;

            if new_ss < 0 {
                new_ss = 0;
            }
            if new_ss >= 60 {
                new_ss = 59;
            }

            ss_input.set_value(format!("{}", new_ss).as_str());

            // let countdown = (new_mm * 60 + new_ss) as u32;

            // tx.send(ChannelMessage::UpdateCountdown(countdown, false));
        };

        // 这是两个 partial
        let mut change_countdown_minutes = change_countdown.clone();
        let mut change_minutes = move |minutes: i32| {
            change_countdown_minutes(minutes, 0);
        };

        let mut change_countdown_seconds = change_countdown.clone();
        let mut change_seconds = move |seconds: i32| {
            change_countdown_seconds(0, seconds);
        };

        const MIDDLE_OF_WINDOW: i32 = WINDOW_WIDTH / 2;
        const SCROLL_REST_TIME: u64 = 30;

        let mut input_minutes_clone = input_minutes.clone();
        let mut start_button = start_button.clone();

        window.handle(move |local_window, ev| match ev {
            Event::MouseWheel => {
                if start_button.label() != "Start" {
                    return false;
                }

                let mouse_pos_x = app::event_x();

                // 鼠标的滚动，往上 or 往下；鼠标当前位置在窗口的左半边 or 右半边
                match app::event_dy() {
                    MouseWheel::Up => {
                        if mouse_pos_x < MIDDLE_OF_WINDOW {
                            change_minutes(-1);
                        } else {
                            change_seconds(-1);
                        }
                        thread::sleep(Duration::from_millis(SCROLL_REST_TIME));
                    }
                    MouseWheel::Down => {
                        if mouse_pos_x < MIDDLE_OF_WINDOW {
                            change_minutes(1);
                        } else {
                            change_seconds(1);
                        }

                        thread::sleep(Duration::from_millis(SCROLL_REST_TIME));
                    }
                    _ => {}
                }
                true
            }

            Event::KeyUp => {
                if app::event_key() == Key::Enter {
                    start_button.do_callback();
                }

                if start_button.label() != "Start" {
                    return false;
                }

                let event_key = app::event_key();

                (0..=9)
                    .map(|digit| digit.to_string().parse::<char>().unwrap())
                    .for_each(|ch| {
                        // 键盘输入 0-9 ，每输入 1 位就处理一次；所以，不能输入 2 位数；
                        if event_key == Key::from_char(ch) {
                            let mut countdown = ch.to_string();

                            if countdown == "0" {
                                countdown = "10".to_string();
                            }

                            if local_window.pixel_h() < EXPANDED_WINDOW_HEIGHT - 1 {
                                input_minutes_clone.set_value(format!("{}", countdown).as_str());

                                // tx.send(ChannelMessage::UpdateCountdown(
                                //     60 * countdown.parse::<u32>().unwrap(),
                                //     false,
                                // ));
                            }
                        }
                    });

                let is_ctrl = app::is_event_ctrl();
                let is_shift = app::is_event_shift();
                let is_ctrl_shift = is_ctrl && is_shift;

                if is_ctrl_shift && event_key == Key::Up {
                    change_seconds(5);
                } else if is_ctrl_shift && event_key == Key::Down {
                    change_seconds(-5);
                } else if is_shift && event_key == Key::Up {
                    change_seconds(1);
                } else if is_shift && event_key == Key::Down {
                    change_seconds(-1);
                } else if is_ctrl && event_key == Key::Up {
                    change_minutes(5);
                } else if is_ctrl && event_key == Key::Down {
                    change_minutes(-5);
                } else if event_key == Key::Up {
                    change_minutes(1);
                } else if event_key == Key::Down {
                    change_minutes(-1);
                }
                false
            }
            _ => false,
        });
    }
}

fn main() {
    let app = app::App::default();

    let font = app.load_font("./assets/FiraCode-Regular.ttf").unwrap();

    // rx 用以更新 ui
    let (tx, rx) = app::channel::<ChannelMessage>();

    // 两个按钮之间的消息传递，channel 不需要 mutex，只需要传递 clone
    let (thread_tx, thread_rx) = mpsc::channel::<ChannelMessage>();

    // 全局就此一份，在多个线程中共享的（ui + spawn的多个）
    // 类似 指针 + mutex（lock）
    let remain_millis = Arc::new(Mutex::new(0 as u64));

    let mut main_wnd = Window::default()
        .with_size(WINDOW_WIDTH, WINDOW_HEIGHT)
        .with_label("Timer")
        .with_pos(50, 160);
    main_wnd.set_color(Color::Black);

    // 整个窗口的 布局，列布局，3 行
    let mut layout = group::Flex::default().column().size_of_parent();
    layout.set_margins(0, 20, 0, 20);
    layout.end();

    // 1. frame 只是显示数字
    // let mut frame = Frame::default().with_pos(0, -23).size_of(&main_wnd);
    let mut title_label = Frame::default();

    title_label.set_label_size(70);
    title_label.set_label_color(Color::White);
    title_label.set_label_font(enums::Font::by_name(&font));

    update_countdown(&mut title_label, 5, false);

    // 2. 最底部 flex 包含两个数字输入框；创建时，y 的位置在窗口的下侧，所以看不见；
    // DONE: 因为都是 flex 布局，所以不用设定大小 和 位置
    let mut input_group = group::Flex::default().row();
    // .with_pos(0, WINDOW_HEIGHT);
    input_group.set_pad(12);
    input_group.set_margins(20, 0, 20, 0);

    let mut input_minutes = input::IntInput::default();
    let mut input_seconds = input::IntInput::default();
    input_minutes.set_value(&"0".to_owned());
    input_seconds.set_value(&"5".to_owned());

    style_input_fields(&mut input_minutes);
    style_input_fields(&mut input_seconds);

    input_group.end();
    input_group.hide();

    // 3. 中间，两个按钮
    let mut button_group = group::Flex::default().row();
    button_group.set_pad(12);
    button_group.set_margins(20, 0, 20, 0);

    let start_button = StartButton::new(
        &input_minutes,
        &input_seconds,
        &main_wnd,
        &input_group,
        thread_tx.clone(),
        tx.clone(),
        remain_millis.clone(),
    );

    let reset_button = ResetButton::new(&start_button.button, &main_wnd, &input_group, thread_tx);

    button_group.end();
    button_group.show();

    // row 布局
    layout.add(&title_label);
    layout.add(&button_group);
    layout.add(&input_group);
    // 固定 2/3 高度，变动时，只变动 1 的高度
    layout.fixed(&button_group, 30);
    layout.fixed(&input_group, 30);

    layout.show();

    // 没有这个，则显示不了 input；layout 是 main_wnd 的最外层
    main_wnd.resizable(&layout);

    InputDeviceEvent::new(
        &mut main_wnd,
        &start_button.button,
        &input_minutes,
        &input_seconds,
    );

    main_wnd.end();
    main_wnd.show();

    // 第二个窗口
    make_another_window();

    // 启动一个后台线程，负责计时任务
    std::thread::spawn(move || {
        while let Ok(msg) = thread_rx.recv() {
            match msg {
                ChannelMessage::StartCounter(remains) => {
                    println!("MSG: StartCounter.");

                    // 通过循环来计时
                    let start_time = Instant::now();
                    loop {
                        let elapsed_time = start_time.elapsed();

                        // 注意：remains 是不变的，但是 rem 每次循环都减少；
                        let rem = remains
                            .checked_sub(elapsed_time)
                            .unwrap_or_else(|| Duration::new(0, 0));

                        // 改变全局变量，resume 时的剩余时间
                        {
                            *(remain_millis.lock().unwrap()) = rem.as_millis() as u64;
                        }

                        // 向 ui 线程发送消息，附带参数；
                        tx.send(ChannelMessage::UpdateCountdown(rem.as_secs() as u32, true));

                        // todo: 读写的互斥量：暂停 就拿走，这里判断是否有，没有就等待，而不是退出循环；
                        // todo: 外层循环 + revc ，里面又是 循环 + recv，对吗？
                        // 退出内部循环，但是仍在 大的消息处理循环中
                        if let Ok(ChannelMessage::PauseCountdown) = thread_rx.try_recv() {
                            println!("in start loop: Got message to pause countdown");

                            break;
                        }

                        // 先发送消息，再退出计时，保证 最终计时显示是 0
                        if rem.as_secs() == 0 {
                            break;
                        }

                        // todo: rx.recv 处理中，能否 sleep
                        thread::sleep(Duration::from_millis(100));
                    }
                }

                ChannelMessage::StopCountdown => {
                    println!("MSG: StopCountdown.");
                    println!("in msg loop: Got message to stop countdown");

                    // 向 ui 线程发送消息，附带参数；
                    tx.send(ChannelMessage::UpdateCountdown(0, true));
                    // break;
                }

                _ => {
                    println!("in msg loop: unknown msg");
                }
            }
        }

        println!("thread exit.");
    });

    let mut reset_button_clone = reset_button.button.clone();

    while app.wait() {
        if let Some(msg) = rx.recv() {
            match msg {
                ChannelMessage::UpdateCountdown(countdown, update_background) => {
                    update_countdown(&mut title_label, countdown, update_background);
                }

                ChannelMessage::StartClicked => {
                    reset_button_clone.set_label("STOP");
                    reset_button_clone.set_color(Color::Red);
                    reset_button_clone.activate();
                }

                _ => {
                    println!("got something new.");
                }
            }
        }
    }

    // app.run().unwrap();
}

fn make_another_window() {
    let mut another_wnd = Window::default()
        .with_size(300, 500)
        .with_label("Another")
        .with_pos(300, 400);

    another_wnd.set_color(Color::Blue);

    let mut label = Frame::default().with_label("Hello FLTK").with_pos(100, 50);
    label.set_label_color(Color::Green);
    label.set_label_size(40);

    let mut row = group::Flex::default()
        .column()
        .with_pos(10, 100)
        .with_size(100, 250);
    row.set_margin(5);
    row.set_pad(5);

    // todo: 不起作用
    row.set_color(Color::Yellow);

    let mut b1 = Button::default().with_label("first @->");
    let b2 = Button::default().with_label("second");
    let mut b3 = Button::default().with_label("@<- third");

    b1.set_color(Color::Red);
    b1.set_label_color(Color::White);
    b1.set_label_size(28);

    b1.set_callback({
        let mut b2_clone = b2.clone();
        let row_clone = row.clone();
        let mut wnd = another_wnd.clone();
        let mut label1 = label.clone();
        
        move |_btn| {
            b2_clone.hide();
            row_clone.recalc();

            wnd.set_color(Color::DarkMagenta);

            label1.set_label_color(Color::White);
            label1.set_color(Color::White);
            label1.set_damage(true);
        }
    });

    b3.set_callback({
        let mut b2_another_clone = b2.clone();
        let row_clone = row.clone();
        let mut wnd = another_wnd.clone();
        let mut label1 = label.clone();

        move |_btn| {
            b2_another_clone.show();
            row_clone.recalc();

            wnd.set_color(Color::DarkCyan);

            label1.set_label_color(Color::Black);
            label1.set_color(Color::White);
            label1.set_damage(true);

            // 需要调用上层的 redraw
            // row_clone.redraw();
            wnd.redraw();
        }
    });

    row.fixed(&b3, 25);

    row.end();

    another_wnd.end();
    another_wnd.show();
}
