mod api;
mod steps;

use gtk4::prelude::*;
use gtk4::Application;
use adw::ApplicationWindow;
use std::rc::Rc;
use std::cell::RefCell;

const APP_ID: &str = "org.sovrn.oobe";

fn main() {
    tracing_subscriber::fmt::init();

    let app = Application::builder()
        .application_id(APP_ID)
        .build();

    app.connect_activate(|app| {
        let _client = api::OobeClient::new();

        let window = ApplicationWindow::builder()
            .application(app)
            .title("Sovrn OS Setup")
            .default_width(640)
            .default_height(480)
            .build();

        let stack = gtk4::Stack::new();
        stack.set_transition_type(gtk4::StackTransitionType::Crossfade);

        stack.add_titled(&steps::welcome::create(), Some("welcome"), "Welcome");
        stack.add_titled(&steps::language::create(), Some("language"), "Language");
        stack.add_titled(&steps::identity::create(), Some("identity"), "Identity");
        stack.add_titled(&steps::domain::create(), Some("domain"), "Domain");
        stack.add_titled(&steps::network::create(), Some("network"), "Network");
        stack.add_titled(&steps::complete::create(), Some("complete"), "Complete");

        stack.set_visible_child_name("welcome");
        window.set_child(Some(&stack));
        window.present();
    });

    app.run();
}
