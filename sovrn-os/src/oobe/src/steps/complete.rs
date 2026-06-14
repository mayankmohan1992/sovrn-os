// Setup complete step

use gtk4::prelude::*;
use gtk4::{Box, Label, Button, Orientation};
use adw::StatusPage;

pub fn create() -> Box {
    let page = StatusPage::builder()
        .title("Setup Complete!")
        .description("Your Sovrn OS is ready.
Welcome to the mesh.")
        .icon_name("emblem-default")
        .build();

    let start_btn = Button::with_label("Start Using Sovrn OS");
    start_btn.add_css_class("suggested-action");
    start_btn.add_css_class("pill");
    start_btn.set_margin_top(24);
    start_btn.set_halign(gtk4::Align::Center);

    start_btn.connect_clicked(|_| {
        let _ = std::fs::write("/var/lib/sovrn/oobe-complete", "1");
        let _ = std::process::Command::new("systemctl")
            .args(["start", "sovrn.target"])
            .spawn();
    });

    let box_ = Box::new(Orientation::Vertical, 24);
    box_.set_valign(gtk4::Align::Center);
    box_.set_halign(gtk4::Align::Center);
    box_.set_margin_start(48);
    box_.set_margin_end(48);
    box_.append(&page);
    box_.append(&start_btn);

    box_
}
