// Domain registration step

use gtk4::prelude::*;
use gtk4::{Box, Label, Button, Orientation, Entry};

pub fn create() -> Box {
    let title = Label::new(Some("Register Your Domain"));
    title.add_css_class("title-4");
    title.set_margin_bottom(12);

    let description = Label::new(Some(
        "Choose your unique name on the Sovrn mesh.
         Your domain will be: yourname.sovrn

         This is how others find you on the decentralized network."
    ));
    description.set_justify(gtk4::Justification::Center);
    description.set_margin_bottom(24);

    let name_entry = Entry::new();
    name_entry.set_placeholder_text(Some("your-name"));
    name_entry.set_margin_bottom(8);

    let suffix_label = Label::new(Some(".sovrn"));
    suffix_label.add_css_class("title-2");
    suffix_label.set_halign(gtk4::Align::Start);

    let check_btn = Button::with_label("Check Availability");
    check_btn.add_css_class("suggested-action");

    let result_label = Label::new(None);
    result_label.set_margin_bottom(12);

    let register_btn = Button::with_label("Register Domain");
    register_btn.set_sensitive(false);
    register_btn.add_css_class("suggested-action");

    let input_box = Box::new(Orientation::Horizontal, 8);
    input_box.append(&name_entry);
    input_box.append(&suffix_label);

    let box_ = Box::new(Orientation::Vertical, 12);
    box_.set_valign(gtk4::Align::Center);
    box_.set_halign(gtk4::Align::Center);
    box_.set_margin_start(48);
    box_.set_margin_end(48);
    box_.append(&title);
    box_.append(&description);
    box_.append(&input_box);
    box_.append(&check_btn);
    box_.append(&result_label);
    box_.append(&register_btn);

    box_
}
