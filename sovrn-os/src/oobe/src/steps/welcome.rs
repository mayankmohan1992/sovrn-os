// Welcome step - Introduction to Sovrn OS

use gtk4::prelude::*;
use gtk4::{Box, Label, Orientation};
use adw::StatusPage;

pub fn create() -> Box {
    let page = StatusPage::builder()
        .title("Welcome to Sovrn OS")
        .description("A privacy-first, decentralized operating system.

Your mesh. Your rules.")
        .icon_name("system-software-install")
        .build();

    let box_ = Box::new(Orientation::Vertical, 24);
    box_.set_valign(gtk4::Align::Center);
    box_.set_halign(gtk4::Align::Center);
    box_.append(&page);

    let desc = Label::new(Some(
        "This wizard will guide you through:

         * Choosing your language
         * Creating your mesh identity
         * Registering your .sovrn domain
         * Configuring your network connection

         Let's get started!"
    ));
    desc.set_justify(gtk4::Justification::Center);
    desc.set_margin_top(12);
    box_.append(&desc);

    box_
}
