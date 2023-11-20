use std::env::VarError;
use std::error::Error;
use std::ffi::OsStr;
use std::fmt::Display;
use std::fs::File;
use std::io::Read;

#[derive(Debug)]
pub struct Config {
	pub host: String,
	pub domain: String,
	pub password: String,
	pub interval: String,
}

impl Config {
	pub fn parse_from_env() -> Config {
		Config {
			host: get_env_var("HOST"),
			domain: get_env_var("DOMAIN"),
			password: get_file_env_var("PASSWORD_FILE"),
			interval: get_env_var("INTERVAL"),
		}
	}
}

fn get_env_var(key: impl AsRef<OsStr>) -> String {
	let err = match std::env::var(&key) {
		Ok(res) => return res,
		Err(err) => err,
	};

	match err {
		VarError::NotPresent => panic!("missing environment var {:?}", key.as_ref()),
		VarError::NotUnicode(s) => panic!("environment var {:?} not valid unicode, was {:?}", key.as_ref(), s),
	}
}

fn get_file_env_var(key: impl AsRef<OsStr>) -> String {
	let mut res = String::new();
	let path = get_env_var(key);
	let mut file = match File::open(&path) {
		Ok(file) => file,
		Err(err) => exit_with_error(format!("could not open file ({path})"), err),
	};
	let _read_bytes = match file.read_to_string(&mut res) {
		Ok(bytes) => bytes,
		Err(err) => exit_with_error(format!("could not read from file ({path})"), err),
	};

	res.trim().to_string()
}

fn exit_with_error(msg: impl Display, e: impl Error) -> ! {
	panic!("{msg}, due to:\n\n{}", format_error(e))
}

fn format_error(e: impl Error) -> String {
	let mut res = format!(" - {}", e.to_string());

	let mut source = e.source();

	while let Some(e) = source {
		res.push_str(&format!(", caused by\n - {}", e.to_string()));

		source = e.source();
	}

	res.trim().to_string()
}

