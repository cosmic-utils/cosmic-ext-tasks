use cosmic::{
    Application,
    cosmic_config::{Config, CosmicConfigEntry},
};
use directories::ProjectDirs;

use crate::{
    Result,
    config::AppConfig,
    shared::store::{Store, migrations},
};

use super::AppModel;

#[derive(Clone, Debug)]
pub struct Flags {
    pub config: AppConfig,
    pub handler: Config,
    pub store: Store,
}

impl Flags {
    pub fn load() -> Result<Self> {
        // Determine the project directories for this application.
        let project = ProjectDirs::from("dev", "edfloreshz", "Tasks")
            .expect("Failed to determine project directories");

        // Run migrations to ensure old data is converted to the new format before starting the app.
        let old_base_dir = project
            .data_dir()
            .parent()
            .expect("Failed to determine previous app directory")
            .join(AppModel::APP_ID);

        let new_base_dir = project.data_dir();

        migrations::migrate(old_base_dir, new_base_dir)?;

        // Store is used for persistent storage of tasks and app state.
        let store = Store::open(project.data_dir())?;

        // Ensure the task state registry exists, seeding the built-in states on first run.
        if let Err(err) = store.states().load_all() {
            tracing::error!("Error loading task states: {err}");
        }

        tracing::info!("Project data directory: {:?}", project.data_dir());

        // Config handler for managing the app's configuration.
        let handler = Config::new(AppModel::APP_ID, AppConfig::VERSION)?;

        // Load the app's configuration, falling back to defaults if there are errors.
        let config = AppConfig::get_entry(&handler).unwrap_or_else(|(errs, config)| {
            tracing::info!("errors loading config: {:?}", errs);
            config
        });

        Ok(Self {
            config,
            handler,
            store,
        })
    }
}
