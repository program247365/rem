use color_eyre::Result;
use crossterm::event::KeyEvent;
use ratatui::prelude::Rect;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tracing::{debug, info};

use crate::{
    action::Action,
    components::{
        Component, fps::FpsCounter, home::Home, lists::ListsComponent,
        permission::PermissionComponent, reminders::RemindersComponent,
    },
    config::Config,
    eventkit::EventKitManager,
    tui::{Event, Tui},
};

pub struct App {
    config: Config,
    tick_rate: f64,
    frame_rate: f64,
    components: Vec<Box<dyn Component>>,
    should_quit: bool,
    should_suspend: bool,
    mode: Mode,
    last_tick_key_events: Vec<KeyEvent>,
    action_tx: mpsc::UnboundedSender<Action>,
    action_rx: mpsc::UnboundedReceiver<Action>,
    eventkit: Option<EventKitManager>,
    permission_component: PermissionComponent,
    lists_component: Option<ListsComponent>,
    reminders_component: Option<RemindersComponent>,
    last_ctrl_c_time: Option<std::time::Instant>,
}

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Mode {
    #[default]
    Home,
    Permission,
    Lists,
    Reminders,
}

impl App {
    pub fn new(tick_rate: f64, frame_rate: f64) -> Result<Self> {
        let (action_tx, action_rx) = mpsc::unbounded_channel();
        let app = Self {
            tick_rate,
            frame_rate,
            components: vec![Box::new(Home::new()), Box::new(FpsCounter::default())],
            should_quit: false,
            should_suspend: false,
            config: Config::new()?,
            mode: Mode::Permission,
            last_tick_key_events: Vec::new(),
            action_tx: action_tx.clone(),
            action_rx,
            eventkit: None,
            permission_component: PermissionComponent::new(),
            lists_component: None,
            reminders_component: None,
            last_ctrl_c_time: None,
        };

        // Start permission check immediately
        action_tx.send(Action::CheckPermissions)?;

        Ok(app)
    }

    pub async fn run(&mut self) -> Result<()> {
        let mut tui = Tui::new()?
            // .mouse(true) // uncomment this line to enable mouse support
            .tick_rate(self.tick_rate)
            .frame_rate(self.frame_rate);
        tui.enter()?;

        for component in self.components.iter_mut() {
            component.register_action_handler(self.action_tx.clone())?;
        }
        for component in self.components.iter_mut() {
            component.register_config_handler(self.config.clone())?;
        }
        for component in self.components.iter_mut() {
            component.init(tui.size()?)?;
        }

        // Register permission component
        self.permission_component
            .register_action_handler(self.action_tx.clone())?;
        self.permission_component
            .register_config_handler(self.config.clone())?;

        let action_tx = self.action_tx.clone();
        loop {
            self.handle_events(&mut tui).await?;
            self.handle_actions(&mut tui)?;
            if self.should_suspend {
                tui.suspend()?;
                action_tx.send(Action::Resume)?;
                action_tx.send(Action::ClearScreen)?;
                // tui.mouse(true);
                tui.enter()?;
            } else if self.should_quit {
                tui.stop()?;
                break;
            }
        }
        tui.exit()?;
        Ok(())
    }

    async fn handle_events(&mut self, tui: &mut Tui) -> Result<()> {
        let Some(event) = tui.next_event().await else {
            return Ok(());
        };
        let action_tx = self.action_tx.clone();
        match event {
            Event::Quit => action_tx.send(Action::Quit)?,
            Event::Tick => action_tx.send(Action::Tick)?,
            Event::Render => action_tx.send(Action::Render)?,
            Event::Resize(x, y) => action_tx.send(Action::Resize(x, y))?,
            Event::Key(key) => self.handle_key_event(key)?,
            _ => {}
        }
        for component in self.components.iter_mut() {
            if let Some(action) = component.handle_events(Some(event.clone()))? {
                action_tx.send(action)?;
            }
        }

        // Handle events for current mode-specific components
        match self.mode {
            Mode::Permission => {
                if let Some(action) = self
                    .permission_component
                    .handle_events(Some(event.clone()))?
                {
                    action_tx.send(action)?;
                }
            }
            Mode::Lists => {
                if let Some(lists_component) = &mut self.lists_component {
                    if let Some(action) = lists_component.handle_events(Some(event.clone()))? {
                        action_tx.send(action)?;
                    }
                }
            }
            Mode::Reminders => {
                if let Some(reminders_component) = &mut self.reminders_component {
                    if let Some(action) = reminders_component.handle_events(Some(event.clone()))? {
                        action_tx.send(action)?;
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_key_event(&mut self, key: KeyEvent) -> Result<()> {
        let action_tx = self.action_tx.clone();
        
        // Handle double Ctrl+C to quit
        if key.code == crossterm::event::KeyCode::Char('c') && key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) {
            let now = std::time::Instant::now();
            if let Some(last_time) = self.last_ctrl_c_time {
                if now.duration_since(last_time) < std::time::Duration::from_millis(500) {
                    // Double Ctrl+C within 500ms - quit immediately
                    self.should_quit = true;
                    return Ok(());
                }
            }
            self.last_ctrl_c_time = Some(now);
        }
        
        let Some(keymap) = self.config.keybindings.get(&self.mode) else {
            return Ok(());
        };
        match keymap.get(&vec![key]) {
            Some(action) => {
                info!("Got action: {action:?}");
                action_tx.send(action.clone())?;
            }
            _ => {
                // If the key was not handled as a single key action,
                // then consider it for multi-key combinations.
                self.last_tick_key_events.push(key);

                // Check for multi-key combinations
                if let Some(action) = keymap.get(&self.last_tick_key_events) {
                    info!("Got action: {action:?}");
                    action_tx.send(action.clone())?;
                }
            }
        }
        Ok(())
    }

    fn handle_actions(&mut self, tui: &mut Tui) -> Result<()> {
        while let Ok(action) = self.action_rx.try_recv() {
            if action != Action::Tick && action != Action::Render {
                debug!("{action:?}");
            }
            match action {
                Action::Tick => {
                    self.last_tick_key_events.drain(..);
                }
                Action::Quit => self.should_quit = true,
                Action::Suspend => self.should_suspend = true,
                Action::Resume => self.should_suspend = false,
                Action::ClearScreen => tui.terminal.clear()?,
                Action::Resize(w, h) => self.handle_resize(tui, w, h)?,
                Action::Render => self.render(tui)?,
                Action::CheckPermissions => {
                    if let Some(action) = self.permission_component.update(action.clone())? {
                        self.action_tx.send(action)?;
                    }
                }
                Action::RequestPermissions => {
                    if let Some(action) = self.permission_component.update(action.clone())? {
                        self.action_tx.send(action)?;
                    }
                }
                Action::LoadLists => {
                    if let Some(eventkit) = self.permission_component.get_eventkit() {
                        self.eventkit = Some(EventKitManager::new()?);
                        let mut lists_component = ListsComponent::new();
                        lists_component.register_action_handler(self.action_tx.clone())?;
                        lists_component.register_config_handler(self.config.clone())?;

                        // Load lists directly into the component
                        if let Err(e) = lists_component.load_lists(eventkit) {
                            eprintln!("Failed to load lists: {}", e);
                        }

                        self.lists_component = Some(lists_component);
                        self.mode = Mode::Lists;
                    }
                }
                Action::SelectList(ref list_id) => {
                    if let Some(_lists_component) = &self.lists_component {
                        // Find the selected list to get its title
                        let list_title = format!("List {list_id}"); // Simplified

                        let mut reminders_component =
                            RemindersComponent::new(list_id.clone(), list_title);
                        reminders_component.register_action_handler(self.action_tx.clone())?;
                        reminders_component.register_config_handler(self.config.clone())?;

                        self.reminders_component = Some(reminders_component);
                        self.mode = Mode::Reminders;

                        // Load reminders
                        self.action_tx
                            .send(Action::LoadReminders(list_id.clone()))?;
                    }
                }
                Action::LoadReminders(ref list_id) => {
                    if let Some(reminders_component) = &mut self.reminders_component {
                        if let Some(eventkit) = &self.eventkit {
                            // Load reminders directly into the component
                            if let Err(e) = reminders_component.load_reminders(eventkit) {
                                eprintln!("Failed to load reminders: {}", e);
                            }
                        }
                    }
                }
                Action::Back => match self.mode {
                    Mode::Reminders => {
                        self.reminders_component = None;
                        self.mode = Mode::Lists;
                    }
                    Mode::Lists => {
                        self.lists_component = None;
                        self.mode = Mode::Permission;
                    }
                    _ => {}
                },
                _ => {}
            }
            for component in self.components.iter_mut() {
                if let Some(action) = component.update(action.clone())? {
                    self.action_tx.send(action)?
                };
            }
        }
        Ok(())
    }

    fn handle_resize(&mut self, tui: &mut Tui, w: u16, h: u16) -> Result<()> {
        tui.resize(Rect::new(0, 0, w, h))?;
        self.render(tui)?;
        Ok(())
    }

    fn render(&mut self, tui: &mut Tui) -> Result<()> {
        tui.draw(|frame| {
            match self.mode {
                Mode::Permission => {
                    if let Err(err) = self.permission_component.draw(frame, frame.area()) {
                        let _ = self.action_tx.send(Action::Error(format!(
                            "Failed to draw permission: {err:?}"
                        )));
                    }
                }
                Mode::Lists => {
                    if let Some(lists_component) = &mut self.lists_component {
                        if let Err(err) = lists_component.draw(frame, frame.area()) {
                            let _ = self
                                .action_tx
                                .send(Action::Error(format!("Failed to draw lists: {err:?}")));
                        }
                    }
                }
                Mode::Reminders => {
                    if let Some(reminders_component) = &mut self.reminders_component {
                        if let Err(err) = reminders_component.draw(frame, frame.area()) {
                            let _ = self.action_tx.send(Action::Error(format!(
                                "Failed to draw reminders: {err:?}"
                            )));
                        }
                    }
                }
                _ => {
                    // Default components for other modes
                    for component in self.components.iter_mut() {
                        if let Err(err) = component.draw(frame, frame.area()) {
                            let _ = self
                                .action_tx
                                .send(Action::Error(format!("Failed to draw: {err:?}")));
                        }
                    }
                }
            }
        })?;
        Ok(())
    }
}
