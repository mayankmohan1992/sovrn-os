// Network configuration step

use gtk4::prelude::*;
use gtk4::{Box, Label, Orientation, Switch};

pub fn create() -> Box {
    let title = Label::new(Some("Network Configuration"));
    title.add_css_class("title-4");
    title.set_margin_bottom(12);

    let description = Label::new(Some(
        "Sovrn OS connects to the mesh via Yggdrasil.
         Configure your network settings below."
    ));
    description.set_justify(gtk4::Justification::Center);
    description.set_margin_bottom(24);

    let ygg_switch = Switch::new();
    ygg_switch.set_active(true);
    let ygg_label = Label::new(Some("Enable Yggdrasil mesh networking"));
    let ygg_box = Box::new(Orientation::Horizontal, 12);
    ygg_box.append(&ygg_label);
    ygg_box.append(&ygg_switch);

    let dns_switch = Switch::new();
    dns_switch.set_active(true);
    let dns_label = Label::new(Some("Use Sovrn DNS resolver (.sovrn domains)"));
    let dns_box = Box::new(Orientation::Horizontal, 12);
    dns_box.append(&dns_label);
    dns_box.append(&dns_switch);

    let fw_switch = Switch::new();
    fw_switch.set_active(true);
    let fw_label = Label::new(Some("Enable Mesh Firewall (nftables)"));
    let fw_box = Box::new(Orientation::Horizontal, 12);
    fw_box.append(&fw_label);
    fw_box.append(&fw_switch);

    let box_ = Box::new(Orientation::Vertical, 24);
    box_.set_valign(gtk4::Align::Center);
    box_.set_halign(gtk4::Align::Center);
    box_.set_margin_start(48);
    box_.set_margin_end(48);
    box_.append(&title);
    box_.append(&description);
    box_.append(&ygg_box);
    box_.append(&dns_box);
    box_.append(&fw_box);

    box_
}
