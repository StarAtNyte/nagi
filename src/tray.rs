use tray_icon::{
    menu::{Menu, MenuEvent, MenuItem},
    TrayIcon, TrayIconBuilder,
};

pub enum TrayAction {
    ShowGui,
    ToggleMute,
    TogglePlayPause,
    Reload,
    Lucky,
    Quit,
}

pub struct TrayHandle {
    #[allow(dead_code)]
    icon: TrayIcon,
    pub item_show: MenuItem,
    pub item_mute: MenuItem,
    pub item_playpause: MenuItem,
    pub item_reload: MenuItem,
    pub item_lucky: MenuItem,
    pub item_quit: MenuItem,
}

impl TrayHandle {
    pub fn new() -> Option<Self> {
        let item_show     = MenuItem::new("Show Nagi",          true, None);
        let item_mute     = MenuItem::new("Toggle Mute",        true, None);
        let item_playpause = MenuItem::new("Toggle Play/Pause", true, None);
        let item_reload   = MenuItem::new("Reload",             true, None);
        let item_lucky    = MenuItem::new("I'm Feeling Lucky",  true, None);
        let item_quit     = MenuItem::new("Quit Nagi",          true, None);

        let menu = Menu::with_items(&[
            &item_show,
            &item_mute,
            &item_playpause,
            &item_reload,
            &item_lucky,
            &item_quit,
        ]).ok()?;

        let icon = TrayIconBuilder::new()
            .with_tooltip("Nagi")
            .with_menu(Box::new(menu))
            .with_icon(
                tray_icon::Icon::from_rgba(vec![80u8; 32 * 32 * 4], 32, 32).ok()?,
            )
            .build()
            .ok()?;

        Some(Self { icon, item_show, item_mute, item_playpause, item_reload, item_lucky, item_quit })
    }

    pub fn poll_action(&self) -> Option<TrayAction> {
        let event = MenuEvent::receiver().try_recv().ok()?;
        if event.id == self.item_show.id()      { return Some(TrayAction::ShowGui); }
        if event.id == self.item_mute.id()      { return Some(TrayAction::ToggleMute); }
        if event.id == self.item_playpause.id() { return Some(TrayAction::TogglePlayPause); }
        if event.id == self.item_reload.id()    { return Some(TrayAction::Reload); }
        if event.id == self.item_lucky.id()     { return Some(TrayAction::Lucky); }
        if event.id == self.item_quit.id()      { return Some(TrayAction::Quit); }
        None
    }
}
