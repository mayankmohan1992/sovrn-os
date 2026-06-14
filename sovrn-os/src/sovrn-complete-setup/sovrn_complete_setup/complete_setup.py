"""First-boot GTK4/Adwaita dialog for optional package installation.
Launched via XDG autostart (runs as user). Uses pkexec for privilege elevation."""
import sys
import os
import subprocess
import threading
import gi

gi.require_version("Gtk", "4.0")
gi.require_version("Adw", "1")
from gi.repository import Gtk, Adw, GLib

SETUP_COMPLETE_MARKER = "/var/lib/sovrn/setup-complete"

PACKAGE_GROUPS = {
    "Office": ["libreoffice", "evince", "libreoffice-gtk3"],
    "Media": ["vlc", "totem", "gnome-music", "sound-juicer"],
    "Games": ["gnome-games", "five-or-more", "four-in-a-row", "gnome-sudoku"],
    "Development": ["code", "git-gui", "meliae"],
    "Productivity": [
        "gnome-calendar",
        "gnome-contacts",
        "evolution",
        "gnome-todo",
    ],
    "Extras": [
        "gnome-weather",
        "gnome-maps",
        "gnome-clocks",
        "baobab",
        "file-roller",
    ],
}


class CompleteSetup(Adw.Application):
    def __init__(self):
        super().__init__(application_id="org.sovrn.CompleteSetup")
        self.selected_groups = set()
        self.install_running = False

    def do_activate(self):
        if os.path.exists(SETUP_COMPLETE_MARKER):
            self.quit()
            return

        window = Adw.ApplicationWindow(application=self)
        window.set_title("Complete Sovrn OS Installation")
        window.set_default_size(600, 500)

        header = Adw.HeaderBar()
        window.set_titlebar(header)

        box = Gtk.Box(
            orientation=Gtk.Orientation.VERTICAL,
            spacing=12,
            margin_top=24,
            margin_bottom=24,
            margin_start=24,
            margin_end=24,
        )

        title = Adw.StatusPage()
        title.set_title("Complete Your Installation")
        title.set_description(
            "Sovrn OS includes a minimal desktop. "
            "Select additional package groups to install:"
        )
        title.set_icon_name("preferences-system-symbolic")
        box.append(title)

        groups_box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=8)
        for group_name, pkgs in PACKAGE_GROUPS.items():
            row = Adw.ActionRow(
                title=group_name,
                subtitle=f"{len(pkgs)} packages",
            )
            check = Gtk.CheckButton()
            check.connect("toggled", self.on_group_toggled, group_name)
            row.add_prefix(check)
            row.set_activatable_widget(check)
            groups_box.append(row)
        box.append(groups_box)

        btn_box = Gtk.Box(
            orientation=Gtk.Orientation.HORIZONTAL,
            spacing=12,
            halign=Gtk.Align.END,
        )
        skip_btn = Gtk.Button(label="Skip — Minimal is Fine")
        skip_btn.connect("clicked", lambda _: self.on_skip())
        install_btn = Gtk.Button(label="Install Selected")
        install_btn.add_css_class("suggested-action")
        install_btn.connect("clicked", self.on_install)
        btn_box.append(skip_btn)
        btn_box.append(install_btn)
        box.append(btn_box)

        window.set_content(box)
        window.present()

    def on_group_toggled(self, check, group_name):
        if check.get_active():
            self.selected_groups.add(group_name)
        else:
            self.selected_groups.discard(group_name)

    def on_skip(self):
        self.mark_complete()
        self.quit()

    def on_install(self, _):
        if not self.selected_groups:
            self.mark_complete()
            self.quit()
            return

        pkgs = []
        for g in self.selected_groups:
            pkgs.extend(PACKAGE_GROUPS[g])

        self.install_running = True
        self.show_spinner(f"Installing {len(pkgs)} packages...")

        def do_install():
            try:
                subprocess.run(
                    ["pkexec", "apt-get", "update"],
                    check=True,
                    capture_output=True,
                    timeout=120,
                )
                subprocess.run(
                    ["pkexec", "apt-get", "install", "-y", "--no-install-recommends"] + pkgs,
                    check=True,
                    capture_output=True,
                    timeout=600,
                )
                GLib.idle_add(self.show_done, True)
            except Exception as e:
                GLib.idle_add(self.show_done, False, str(e))

        threading.Thread(target=do_install, daemon=True).start()

    def show_spinner(self, message):
        page = Adw.StatusPage()
        page.set_title("Installing Packages")
        page.set_description(message + "\nThis may take a few minutes.")
        page.set_icon_name("media-playback-start-symbolic")
        spinner = Gtk.Spinner()
        spinner.set_size_request(48, 48)
        spinner.start()
        page.set_child(spinner)
        window = self.get_active_window()
        window.set_content(page)

    def show_done(self, success, error=None):
        if success:
            page = Adw.StatusPage()
            page.set_title("Installation Complete")
            page.set_description(
                "Additional packages installed. "
                "You can now enjoy the full desktop experience."
            )
            page.set_icon_name("emblem-ok-symbolic")
        else:
            page = Adw.StatusPage()
            page.set_title("Installation Skipped")
            page.set_description(
                f"Could not install packages: {error}\n\n"
                "You can run 'sovrn-complete-setup' manually later "
                "from the terminal when connected to the internet."
            )
            page.set_icon_name("dialog-warning-symbolic")

        btn = Gtk.Button(label="Close")
        btn.connect("clicked", lambda _: self.quit())
        page.set_child(btn)

        window = self.get_active_window()
        window.set_content(page)
        self.mark_complete()
        self.install_running = False

    @staticmethod
    def mark_complete():
        try:
            os.makedirs("/var/lib/sovrn", exist_ok=True)
            with open(SETUP_COMPLETE_MARKER, "w") as f:
                f.write("complete")
        except Exception:
            pass


def main():
    app = CompleteSetup()
    return app.run(sys.argv)
