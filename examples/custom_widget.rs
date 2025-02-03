use std::ops::{Deref, DerefMut};

use fltk::{
    app,
    button,
    enums::{self, Align, Color, Event, FrameType},
    group,
    prelude::*,
    window,
};

// 2025-02-03

// todo: FLTK Rust: Create your own events 
// https://www.youtube.com/watch?v=e3CrpWNU8qY&list=PLHqrrowPLkDu9U-uk60sGM-YWLOJFfLoE&index=27

// https://github.com/wyhinton/FLTK-RS-Examples
// todo: Crossbeam Channels
// todo: Background Fill for Group

// 看似 Button，本质是 Group
struct MyButton {
    grp: group::Group,
    btn: button::Button,
}

impl MyButton {
    pub fn new(x: i32, y: i32, w: i32, h: i32, label: &str) -> MyButton {
        let mut grp = group::Group::new(x, y, w, h, label);
        grp.set_frame(enums::FrameType::RFlatBox);
        grp.set_color(Color::from_u32(0xb3e5fc));
        grp.set_align(Align::Center);

        let mut btn = button::Button::new(grp.x() + 420, grp.y() + 35, 18, 18, "X");
        btn.set_frame(FrameType::RFlatBox);
        btn.set_color(Color::from_u32(0xf49da9));
        grp.end();

        btn.set_callback(move |b| {
            b.parent().unwrap().hide();
        });

        grp.handle(|g, ev| match ev {
            // 鼠标点击事件
            Event::Push => {
                g.do_callback();
                true
            }
            _ => false,
        });

        MyButton { grp, btn }
    }
}

impl Deref for MyButton {
    type Target = group::Group;

    fn deref(&self) -> &Self::Target {
        &self.grp
    }
}

impl DerefMut for MyButton {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.grp
    }
}

fn main() {
    let app = app::App::default();
    app::set_visible_focus(false);

    let mut win = window::Window::default().with_size(500, 400);
    win.set_color(Color::White);

    // Pack 一定要设置大小
    let mut grp = group::Pack::default().size_of(&win); 
    grp.set_spacing(10);

    let mut btn1 = MyButton::new(0, 0, 500, 100, "Btn-1");
    btn1.set_callback(|_| println!("btn1 clicked."));

    let mut btn2 = MyButton::new(0, 0, 500, 100, "Btn-2");
    btn2.set_callback(|_| println!("btn2 clicked."));

    let mut btn3 = MyButton::new(0, 0, 500, 100, "Btn-3");
    btn3.set_callback(|_| println!("btn3 clicked."));

    grp.end();

    win.end();
    win.show();

    app.run().unwrap();
}
