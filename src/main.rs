
use fltk::{
    app, button,
    enums::{Color, FrameType},
    frame::{self, Frame},
    group, input,
    prelude::*,
    window::Window,
};

fn style_button(btn: &mut button::Button) {
    btn.set_color(Color::Cyan);
    btn.set_frame(FrameType::RFlatBox);
    btn.clear_visible_focus();
}

fn show_dialog() -> MyDialog {
    MyDialog::default()
}

struct MyDialog {
    inp: input::Input,
}

impl MyDialog {
    pub fn default() -> Self {
        let mut win = Window::default()
            .with_size(400, 100)
            .with_label("My Dialog");
        win.set_color(Color::from_rgb(240, 240, 240));

        let mut pack = group::Pack::default()
            .with_size(300, 30)
            .center_of_parent()
            .with_type(group::PackType::Horizontal);
        pack.set_spacing(20);
        frame::Frame::default()
            .with_size(80, 0)
            .with_label("Enter name:");
        let mut inp = input::Input::default().with_size(100, 0);
        inp.set_frame(FrameType::FlatBox);
        let mut ok = button::Button::default().with_size(80, 0).with_label("Ok");
        style_button(&mut ok);
        pack.end();
        win.end();
        win.make_modal(true);
        win.show();

        ok.set_callback({
            let mut win = win.clone();
            move |_| {
                win.hide();
            }
        });

        while win.shown() {
            app::wait();
        }

        Self { inp }
    }

    pub fn value(&self) -> String {
        self.inp.value()
    }
}

fn main() {
    println!("Hello, world!");

    let a = app::App::default();

    let mut wind = Window::new(100, 100, 400, 300, "My Window");
    let mut frame = Frame::new(20, 20, 200, 40, "Will be replaced");

    let flx = group::Flex::default()
        .with_size(100, 80)
        .column()
        .center_of_parent();
    let btn1 = button::RadioRoundButton::default().with_label("FLTK");
    let mut btn2 = button::RadioRoundButton::default().with_label("egui");
    let mut btn3 = button::Button::default().with_label("提交");
    flx.end();

    let mut btn4 = button::Button::new(160, 260, 80, 20, "弹窗");
    let mut btn5 = button::Button::new(160, 210, 80, 20, "中文");

    wind.end();
    wind.show();

    let mut frame1 = frame.clone();
    btn4.set_callback(move |_btn4| {
        let d = show_dialog();
        frame1.set_label(&d.value());
    });

    let mut frame2 = frame.clone();
    btn5.set_callback(move |_btn| frame2.set_label("Frame 不配有颜色"));

    a.run().unwrap();
}
