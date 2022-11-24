use std::{collections::HashMap, env, fs, time::SystemTime};
use time::UtcOffset;
use time_tz::{timezones, Offset, TimeZone};

use crate::app_error::AppError;

type EnvHashMap = HashMap<String, String>;

const LOCAL_ENV: &str = ".env";
const DOCKER_ENV: &str = "/app_env/.env";

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct EnvTimeZone(String);

impl EnvTimeZone {
    pub fn new(x: impl Into<String>) -> Self {
        let x = x.into();
        if timezones::get_by_name(&x).is_some() {
            Self(x)
        } else {
            Self("Etc/UTC".into())
        }
    }

    pub fn get_offset(&self) -> UtcOffset {
        timezones::get_by_name(&self.0).map_or(UtcOffset::UTC, |tz| {
            tz.get_offset_utc(&time::OffsetDateTime::now_utc()).to_utc()
        })
    }
}

#[derive(Debug, Clone)]
pub struct AppEnv {
    pub debug: bool,
    pub location_backup: String,
    pub start_time: SystemTime,
    pub timezone: EnvTimeZone,
    pub trace: bool,
    pub ws_address: String,
    pub ws_apikey: String,
    pub ws_password: String,
    pub ws_token_address: String,
}

impl AppEnv {
    /// Parse "true" or "false" to bool, else false
    fn parse_boolean(key: &str, map: &EnvHashMap) -> bool {
        map.get(key).map_or(false, |value| value == "true")
    }

    /// Check a given file actually exists on the file system
    fn check_file_exists(filename: String) -> Result<String, AppError> {
        match fs::metadata(&filename) {
            Ok(_) => Ok(filename),
            Err(_) => Err(AppError::FileNotFound(filename)),
        }
    }

    fn parse_string(key: &str, map: &EnvHashMap) -> Result<String, AppError> {
        map.get(key)
            .map_or(Err(AppError::MissingEnv(key.into())), |value| {
                Ok(value.into())
            })
    }
    /// Check that a given timezone is valid, else return UTC
    fn parse_timezone(map: &EnvHashMap) -> EnvTimeZone {
        EnvTimeZone::new(
            map.get("TIMEZONE")
                .map_or_else(String::new, std::borrow::ToOwned::to_owned),
        )
    }

    /// Load, and parse .env file, return AppEnv
    fn generate() -> Result<Self, AppError> {
        let env_map = env::vars()
            .into_iter()
            .map(|i| (i.0, i.1))
            .collect::<HashMap<String, String>>();

        Ok(Self {
            // check location exists
            debug: Self::parse_boolean("DEBUG", &env_map),
            location_backup: Self::check_file_exists(Self::parse_string(
                "LOCATION_BACKUP",
                &env_map,
            )?)?,
            start_time: SystemTime::now(),
            timezone: Self::parse_timezone(&env_map),
            trace: Self::parse_boolean("TRACE", &env_map),
            // utc_offset: Self::parse_offset(&env_map)?,
            ws_address: Self::parse_string("WS_ADDRESS", &env_map)?,
            ws_apikey: Self::parse_string("WS_APIKEY", &env_map)?,
            ws_password: Self::parse_string("WS_PASSWORD", &env_map)?,
            ws_token_address: Self::parse_string("WS_TOKEN_ADDRESS", &env_map)?,
        })
    }

    pub fn get() -> Self {
        let env_path = if std::fs::metadata(DOCKER_ENV).is_ok() {
            DOCKER_ENV
        } else if std::fs::metadata(LOCAL_ENV).is_ok() {
            LOCAL_ENV
        } else {
            println!("\n\x1b[31munable to load env file\x1b[0m\n");
            std::process::exit(1);
        };

        dotenvy::from_path(env_path).ok();
        match Self::generate() {
            Ok(s) => s,
            Err(e) => {
                println!("\n\x1b[31m{}\x1b[0m\n", e);
                std::process::exit(1);
            }
        }
    }
}

/// Run tests with
///
/// cargo watch -q -c -w src/ -x 'test env_ -- --nocapture'
#[allow(clippy::unwrap_used)]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn env_missing_env() {
        let mut map = HashMap::new();
        map.insert("not_fish".to_owned(), "not_fish".to_owned());
        // ACTION
        let result = AppEnv::parse_string("fish", &map);

        // CHECK
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "missing env: 'fish'");
    }

    #[test]
    fn env_parse_string_valid() {
        // FIXTURES
        let mut map = HashMap::new();
        map.insert("LOCATION_SQLITE".to_owned(), "/alarms.db".to_owned());

        // ACTION
        let result = AppEnv::parse_string("LOCATION_SQLITE", &map).unwrap();

        // CHECK
        assert_eq!(result, "/alarms.db");
    }

    #[test]
    fn env_parse_boolean_ok() {
        // FIXTURES
        let mut map = HashMap::new();
        map.insert("valid_true".to_owned(), "true".to_owned());
        map.insert("valid_false".to_owned(), "false".to_owned());
        map.insert("invalid_but_false".to_owned(), "as".to_owned());

        // ACTION
        let result01 = AppEnv::parse_boolean("valid_true", &map);
        let result02 = AppEnv::parse_boolean("valid_false", &map);
        let result03 = AppEnv::parse_boolean("invalid_but_false", &map);
        let result04 = AppEnv::parse_boolean("missing", &map);

        // CHECK
        assert!(result01);
        assert!(!result02);
        assert!(!result03);
        assert!(!result04);
    }

    #[test]
    fn env_check_file_exists_ok() {
        // check folder exists ok
        let result = AppEnv::check_file_exists("./src".to_owned());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "./src");

        // check file exists ok
        let result = AppEnv::check_file_exists("./Cargo.toml".to_owned());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "./Cargo.toml");
    }

    #[test]
    fn env_check_file_exists_err() {
        // random folder error
        let result = AppEnv::check_file_exists("./some_random_folder".to_owned());
        assert!(result.is_err());

        match result.unwrap_err() {
            AppError::FileNotFound(value) => assert_eq!(value, "./some_random_folder"),
            _ => unreachable!(),
        };

        // random file err
        let result = AppEnv::check_file_exists("./some_random_file.txt".to_owned());
        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::FileNotFound(value) => assert_eq!(value, "./some_random_file.txt"),
            _ => unreachable!(),
        };
    }

    #[test]
    fn env_parse_timezone_ok() {
        // FIXTURES
        let mut map = HashMap::new();
        map.insert("TIMEZONE".to_owned(), "America/New_York".to_owned());

        // ACTION
        let result = AppEnv::parse_timezone(&map);

        // CHECK
        assert_eq!(result.0, "America/New_York");

        let mut map = HashMap::new();
        map.insert("TIMEZONE".to_owned(), "Europe/Berlin".to_owned());

        // ACTION
        let result = AppEnv::parse_timezone(&map);

        // CHECK
        assert_eq!(result.0, "Europe/Berlin");

        // FIXTURES
        let map = HashMap::new();

        // ACTION
        let result = AppEnv::parse_timezone(&map);

        // CHECK
        assert_eq!(result.0, "Etc/UTC");
    }

    #[test]
    fn env_parse_timezone_err() {
        // FIXTURES
        let mut map = HashMap::new();
        map.insert("TIMEZONE".to_owned(), "america/New_York".to_owned());

        // ACTION
        let result = AppEnv::parse_timezone(&map);
        // CHECK
        assert_eq!(result.0, "Etc/UTC");

        // No timezone present
        // FIXTURES
        let map = HashMap::new();
        let result = AppEnv::parse_timezone(&map);

        // CHECK
        assert_eq!(result.0, "Etc/UTC");
    }
    #[test]
    fn env_panic_appenv() {
        // ACTION
        let result = AppEnv::generate();

        assert!(result.is_err());
    }

    #[test]
    fn env_return_appenv() {
        // FIXTURES
        dotenvy::dotenv().unwrap();

        // ACTION
        let result = AppEnv::generate();

        assert!(result.is_ok());
    }
}
