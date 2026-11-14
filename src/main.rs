use adw::prelude::*;
use pole_plugin_api::{PoleClient, RobotState};
use std::rc::Rc;
use std::time::Duration;

struct PluginUi {
    client: PoleClient,
    row_coords: adw::ActionRow,
    row_cell: adw::ActionRow,
    title_widget: adw::WindowTitle,
    entry_msg: adw::EntryRow,
}

impl PluginUi {
    fn update_state(&self) {
        if !self.client.is_connected() {
            self.title_widget.set_subtitle("Socket not found (Pole IDE not running)");
            self.row_coords.set_subtitle("Disconnected");
            self.row_cell.set_subtitle("—");
            return;
        }

        match self.client.robot_get_state() {
            Ok(state) => {
                let status_text = format!("X: {} | Y: {}", state.x, state.y);
                let crash_info = if state.crashed { " (Robot crashed!)" } else { "" };
                self.title_widget.set_subtitle(&format!("Connected to Pole IDE | {}{}", status_text, crash_info));
                self.row_coords.set_subtitle(&format!("Column {}, Row {}", state.x, state.y));
                self.row_cell.set_subtitle(if state.is_painted { "Painted" } else { "Clean" });
            }
            Err(e) => {
                self.title_widget.set_subtitle(&format!("Error: {}", e));
            }
        }
    }

    fn move_robot(&self, dir: &str) {
        let res: Result<RobotState, _> = match dir {
            "up" => self.client.robot_up(),
            "down" => self.client.robot_down(),
            "left" => self.client.robot_left(),
            "right" => self.client.robot_right(),
            _ => return,
        };

        if let Ok(_) = res {
            self.update_state();
        }
    }

    fn paint_cell(&self) {
        if let Ok(_) = self.client.robot_paint() {
            self.update_state();
        }
    }

    fn reset_robot(&self) {
        if let Ok(_) = self.client.robot_reset() {
            self.update_state();
        }
    }

    fn send_console_msg(&self) {
        let text = self.entry_msg.text().to_string();
        if !text.trim().is_empty() {
            let msg = format!("[RustPlugin] {}\n", text.trim());
            let _ = self.client.console_print(&msg);
        }
    }
}

fn main() -> glib::ExitCode {
    let app = adw::Application::builder()
        .application_id("io.github.neoomir.plugin.rust_example")
        .flags(gio::ApplicationFlags::NON_UNIQUE)
        .build();

    app.connect_activate(build_ui);
    app.run()
}

fn build_ui(app: &adw::Application) {
    let window = adw::ApplicationWindow::builder()
        .application(app)
        .title("Pole Rust Native Plugin")
        .default_width(520)
        .default_height(580)
        .build();

    let client = match PoleClient::new() {
        Ok(c) => c,
        Err(_) => PoleClient::with_path("/tmp/neomir.sock"),
    };

    let main_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
    window.set_content(Some(&main_box));

    let header = adw::HeaderBar::new();
    let title_widget = adw::WindowTitle::new("Rust Plugin for Pole", "Connecting to Pole IPC...");
    header.set_title_widget(Some(&title_widget));

    let btn_refresh = gtk::Button::from_icon_name("view-refresh-symbolic");
    btn_refresh.set_tooltip_text(Some("Update Robot state"));
    header.pack_end(&btn_refresh);

    main_box.append(&header);

    let pref_page = adw::PreferencesPage::new();
    main_box.append(&pref_page);

    // Group 1: Robot Status
    let group_status = adw::PreferencesGroup::new();
    group_status.set_title("Robot Actor State (Rust IPC)");
    group_status.set_description(Some("Live telemetry directly from Pole GUI process via socket"));
    pref_page.add(&group_status);

    let row_coords = adw::ActionRow::new();
    row_coords.set_title("Robot Position (X, Y)");
    row_coords.set_subtitle("Unknown");
    row_coords.add_prefix(&gtk::Image::from_icon_name("find-location-symbolic"));
    group_status.add(&row_coords);

    let row_cell = adw::ActionRow::new();
    row_cell.set_title("Current Cell State");
    row_cell.set_subtitle("Unknown");
    row_cell.add_prefix(&gtk::Image::from_icon_name("format-text-color-symbolic"));
    group_status.add(&row_cell);

    // Group 2: Control buttons
    let group_ctrl = adw::PreferencesGroup::new();
    group_ctrl.set_title("Remote Robot Movement");
    group_ctrl.set_description(Some("Control performer on Pole's interactive field in real time"));
    pref_page.add(&group_ctrl);

    let row_dpad = adw::ActionRow::new();
    row_dpad.set_title("Action Controls");

    let dpad_box = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    dpad_box.set_valign(gtk::Align::Center);

    let btn_up = gtk::Button::from_icon_name("go-up-symbolic");
    btn_up.set_tooltip_text(Some("Step Up"));
    dpad_box.append(&btn_up);

    let btn_down = gtk::Button::from_icon_name("go-down-symbolic");
    btn_down.set_tooltip_text(Some("Step Down"));
    dpad_box.append(&btn_down);

    let btn_left = gtk::Button::from_icon_name("go-previous-symbolic");
    btn_left.set_tooltip_text(Some("Step Left"));
    dpad_box.append(&btn_left);

    let btn_right = gtk::Button::from_icon_name("go-next-symbolic");
    btn_right.set_tooltip_text(Some("Step Right"));
    dpad_box.append(&btn_right);

    let btn_paint = gtk::Button::from_icon_name("color-select-symbolic");
    btn_paint.set_tooltip_text(Some("Paint Cell"));
    dpad_box.append(&btn_paint);

    let btn_reset = gtk::Button::from_icon_name("edit-clear-all-symbolic");
    btn_reset.set_tooltip_text(Some("Reset Field"));
    dpad_box.append(&btn_reset);

    row_dpad.add_suffix(&dpad_box);
    group_ctrl.add(&row_dpad);

    // Group 3: Console
    let group_console = adw::PreferencesGroup::new();
    group_console.set_title("Pole Console Bridge");
    group_console.set_description(Some("Log messages directly to the main Pole output console"));
    pref_page.add(&group_console);

    let row_msg = adw::EntryRow::new();
    row_msg.set_title("Text to output in console:");
    row_msg.set_text("Hello from native Rust plugin!");

    let btn_send_msg = gtk::Button::with_label("Print");
    btn_send_msg.add_css_class("suggested-action");
    btn_send_msg.set_valign(gtk::Align::Center);
    row_msg.add_suffix(&btn_send_msg);
    group_console.add(&row_msg);

    let ui = Rc::new(PluginUi {
        client,
        row_coords,
        row_cell,
        title_widget,
        entry_msg: row_msg,
    });

    {
        let ui_clone = ui.clone();
        btn_refresh.connect_clicked(move |_| ui_clone.update_state());
    }
    {
        let ui_clone = ui.clone();
        btn_up.connect_clicked(move |_| ui_clone.move_robot("up"));
    }
    {
        let ui_clone = ui.clone();
        btn_down.connect_clicked(move |_| ui_clone.move_robot("down"));
    }
    {
        let ui_clone = ui.clone();
        btn_left.connect_clicked(move |_| ui_clone.move_robot("left"));
    }
    {
        let ui_clone = ui.clone();
        btn_right.connect_clicked(move |_| ui_clone.move_robot("right"));
    }
    {
        let ui_clone = ui.clone();
        btn_paint.connect_clicked(move |_| ui_clone.paint_cell());
    }
    {
        let ui_clone = ui.clone();
        btn_reset.connect_clicked(move |_| ui_clone.reset_robot());
    }
    {
        let ui_clone = ui.clone();
        btn_send_msg.connect_clicked(move |_| ui_clone.send_console_msg());
    }

    // Auto update on timer
    {
        let ui_clone = ui.clone();
        glib::timeout_add_local(Duration::from_millis(1000), move || {
            ui_clone.update_state();
            glib::ControlFlow::Continue
        });
    }

    ui.update_state();
    window.present();
}
