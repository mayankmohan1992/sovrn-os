// Language selection step

use gtk4::prelude::*;
use gtk4::{Box, Label, Orientation, StringList, ListView, SingleSelection};

pub fn create() -> Box {
    let title = Label::new(Some("Select Your Language"));
    title.add_css_class("title-4");
    title.set_margin_bottom(24);

    let languages = StringList::new(&[
        "English",
        "Hindi",
        "Arabic",
        "Spanish",
        "Portuguese",
        "French",
        "German",
        "Chinese",
        "Japanese",
        "Russian",
    ]);

    let selection = SingleSelection::new(Some(&languages));
    let list_view = ListView::new(Some(selection), None);

    let box_ = Box::new(Orientation::Vertical, 12);
    box_.set_valign(gtk4::Align::Center);
    box_.set_halign(gtk4::Align::Center);
    box_.set_margin_start(48);
    box_.set_margin_end(48);
    box_.append(&title);
    box_.append(&list_view);

    box_
}
