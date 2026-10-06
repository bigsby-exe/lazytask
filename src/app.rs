use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io::{self, Stdout};
use std::time::Duration;

use crate::config::Config;
use crate::handlers::input::{Action, InputHandler};
use crate::handlers::sync::SyncHandler;
use crate::sync_persistence::load_sync_settings;
use crate::taskchampion::TaskChampionIntegration;
use crate::ui::app_ui::AppUI;

pub type AppTerminal = Terminal<CrosstermBackend<Stdout>>;

pub struct App {
    pub config: Config,
    pub terminal: AppTerminal,
    pub ui: AppUI,
    pub input_handler: InputHandler,
    pub taskchampion: TaskChampionIntegration,
    pub sync_handler: SyncHandler,
    pub should_quit: bool,
}

impl App {
    pub async fn new(config_path: Option<&str>, _verbose: bool) -> Result<Self> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;

        let config = Config::load(config_path)?;
        let mut taskchampion = TaskChampionIntegration::new(None).await?;
        if let Some(saved_sync) = load_sync_settings(taskchampion.data_dir())? {
            taskchampion.configure_sync(saved_sync)?;
        }
        let sync_handler = SyncHandler::new();
        let ui = AppUI::new(&config)?;
        let input_handler = InputHandler::new(&config);

        Ok(App {
            config,
            terminal,
            ui,
            input_handler,
            taskchampion,
            sync_handler,
            should_quit: false,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        self.sync_handler.initialize(&self.taskchampion)?;
        self.ui.load_tasks(&mut self.taskchampion).await?;

        let mut needs_redraw = true;

        loop {
            if needs_redraw {
                self.terminal
                    .draw(|f| self.ui.render_with_sync(f, &self.sync_handler))?;
                needs_redraw = false;
            }

            if event::poll(Duration::from_millis(250))? {
                match event::read()? {
                    Event::Key(key) => {
                        let in_form = self.ui.has_active_form();
                        let action = self
                            .input_handler
                            .handle_key_event_with_context(key, in_form);
                        match action {
                            Action::Quit => {
                                self.should_quit = true;
                            }
                            _ => {
                                self.ui
                                    .handle_action(
                                        action,
                                        &mut self.taskchampion,
                                        &mut self.sync_handler,
                                    )
                                    .await?;
                                needs_redraw = true;
                            }
                        }
                    }
                    Event::Resize(_, _) => {
                        needs_redraw = true;
                    }
                    _ => {}
                }
            }

            if self.should_quit {
                break;
            }
        }

        Ok(())
    }
}

impl Drop for App {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(
            self.terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        );
        let _ = self.terminal.show_cursor();
    }
}
