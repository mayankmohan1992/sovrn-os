// Identity creation step - seed phrase generation

use gtk4::prelude::*;
use gtk4::{Box, Label, Button, Orientation, TextView, TextBuffer};

pub fn create() -> Box {
    let title = Label::new(Some("Create Your Identity"));
    title.add_css_class("title-4");
    title.set_margin_bottom(12);

    let description = Label::new(Some(
        "Your Sovrn identity is based on a cryptographic key pair.
         The seed phrase below is the ONLY way to recover your identity.
         Write it down and store it securely - there is no password reset!"
    ));
    description.set_justify(gtk4::Justification::Center);
    description.set_margin_bottom(24);

    let seed_phrase = "abandon ability able about above absent absorb abstract absurd abuse access accident";
    let seed_buffer = TextBuffer::new(None);
    seed_buffer.set_text(seed_phrase);

    let seed_view = TextView::with_buffer(&seed_buffer);
    seed_view.set_editable(false);
    seed_view.set_wrap_mode(gtk4::WrapMode::Word);
    seed_view.set_margin_bottom(12);
    seed_view.add_css_class("card");

    let warning = Label::new(Some("WARNING: Store this seed phrase securely. Never share it with anyone."));
    warning.add_css_class("error");
    warning.set_margin_bottom(12);

    let copy_btn = Button::with_label("Copy to Clipboard");

    let box_ = Box::new(Orientation::Vertical, 12);
    box_.set_valign(gtk4::Align::Center);
    box_.set_halign(gtk4::Align::Center);
    box_.set_margin_start(48);
    box_.set_margin_end(48);
    box_.append(&title);
    box_.append(&description);
    box_.append(&seed_view);
    box_.append(&warning);
    box_.append(&copy_btn);

    box_
}
