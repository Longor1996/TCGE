extern crate toml;
use crate::router;
use std::fs;
use std::io::Read;

pub struct Settings {
	pub table: toml::value::Table,
}

impl Settings {
	pub fn init() -> Settings {
		Settings {
			table: toml::value::Table::new(),
		}
	}
	
	pub fn load(&mut self) -> Result<(),SettingsError> {
		let exe_file_name = ::std::env::current_exe()
			.map_err(|_| SettingsError::LoadError("Failed to find executable.".to_string()))?;
		
		let exe_path = exe_file_name.parent()
			.ok_or(SettingsError::LoadError("Failed to find executable directory.".to_string()))?;
		
		let config_dir = exe_path.join("config");
		let config_file = config_dir.join("engine.toml");
		
		let mut config_file = fs::File::open(config_file.as_path())
			.map_err(|err| {SettingsError::LoadError(err.to_string())})?;
		
		let mut config_str = String::new();
		config_file.read_to_string(&mut config_str)
			.map_err(|err| {SettingsError::LoadError(err.to_string())})?;
		
		let config = config_str.parse::<toml::Value>()
			.map_err(|err| {SettingsError::LoadError(err.to_string())})?;
		
		if let Some(config) = config.as_table() {
			self.table = config.clone();
		} else {
			return Err(SettingsError::LoadError("Top-Level value must be a table.".to_string()));
		}
		
		Ok(())
	}
}



pub enum SettingsError {
	LoadError(String),
}

impl router::comp::Component for Settings {
	fn get_type_name(&self) -> &'static str {
		"Settings"
	}
	
	fn on_attachment(&mut self, _node_id: usize) {}
	fn on_detachment(&mut self, _node_id: usize) {}
	
	fn on_load(&mut self) {}
	fn on_unload(&mut self) {}
	
	fn on_event(&mut self, _event: &mut router::event::Wrapper) {}
}



pub struct SettingsReloadEvent {
	pub settings: &'static Settings
}

impl SettingsReloadEvent {
	pub fn new(settings: &'static Settings) -> SettingsReloadEvent {
		SettingsReloadEvent {settings}
	}
}

impl router::event::Event for SettingsReloadEvent {
	fn is_passive(&self) -> bool {false}
}
