use std::{collections::HashMap, env, fs};

use jiff::tz::TimeZone;

use crate::app_error::AppError;

type EnvHashMap = HashMap<String, String>;

const LOCAL_ENV: &str = ".env";
const DOCKER_ENV: &str = "/app_env/.env";

#[derive(Debug, Clone)]
pub struct AppEnv {
    pub download_time: (i8, i8),
    pub location_backup: String,
    pub log_level: tracing::Level,
    pub timezone: TimeZone,
    pub ws_address: String,
    pub ws_apikey: String,
    pub ws_password: String,
    pub ws_token_address: String,
}

impl AppEnv {
    /// Parse "true" or "false" to bool, else false
    fn parse_boolean(key: &str, map: &EnvHashMap) -> bool {
        map.get(key).is_some_and(|value| value == "true")
    }

    /// Check a given file actually exists on the file system
    fn check_file_exists(filename: String) -> Result<String, AppError> {
        if fs::exists(&filename).unwrap_or_default() {
            Ok(filename)
        } else {
            Err(AppError::FileNotFound(filename))
        }
    }

    fn parse_string(key: &str, map: &EnvHashMap) -> Result<String, AppError> {
        map.get(key)
            .map_or(Err(AppError::MissingEnv(key.into())), |value| {
                Ok(value.into())
            })
    }
    /// Check that a given timezone is valid, else return UTC
    fn parse_timezone(map: &EnvHashMap) -> TimeZone {
        map.get("TIMEZONE").map_or(TimeZone::UTC, |s| {
            jiff::tz::TimeZone::get(s).unwrap_or(TimeZone::UTC)
        })
    }

    fn parse_download_time(map: &EnvHashMap) -> (i8, i8) {
        if let Ok(value) = Self::parse_string("DL_TIME", map) {
            let hour = value[0..2]
                .parse::<u8>()
                .ok()
                .and_then(|v| i8::try_from(v).ok());

            let minute = value[2..]
                .parse::<u8>()
                .ok()
                .and_then(|v| i8::try_from(v).ok());

            if let Some(hour) = hour
                && let Some(minute) = minute
                && hour < 24
                && minute < 59
            {
                return (hour, minute);
            }
        }
        (3, 0)
    }

    /// Parse debug and/or trace into tracing level
    fn parse_log(map: &EnvHashMap) -> tracing::Level {
        if Self::parse_boolean("LOG_TRACE", map) {
            tracing::Level::TRACE
        } else if Self::parse_boolean("LOG_DEBUG", map) {
            tracing::Level::DEBUG
        } else {
            tracing::Level::INFO
        }
    }

    /// Load, and parse .env file, return AppEnv
    fn generate() -> Result<Self, AppError> {
        let env_map = env::vars()
            .map(|i| (i.0, i.1))
            .collect::<HashMap<String, String>>();

        Ok(Self {
            download_time: Self::parse_download_time(&env_map),
            location_backup: Self::check_file_exists(Self::parse_string(
                "LOCATION_BACKUP",
                &env_map,
            )?)?,
            timezone: Self::parse_timezone(&env_map),
            log_level: Self::parse_log(&env_map),
            ws_address: Self::parse_string("WS_ADDRESS", &env_map)?,
            ws_apikey: Self::parse_string("WS_APIKEY", &env_map)?,
            ws_password: Self::parse_string("WS_PASSWORD", &env_map)?,
            ws_token_address: Self::parse_string("WS_TOKEN_ADDRESS", &env_map)?,
        })
    }

    pub fn get() -> Self {
        let env_path = if std::fs::exists(DOCKER_ENV).unwrap_or_default() {
            DOCKER_ENV
        } else if std::fs::exists(LOCAL_ENV).unwrap_or_default() {
            LOCAL_ENV
        } else {
            println!("\n\x1b[31munable to load env file\x1b[0m\n");
            std::process::exit(1);
        };

        dotenvy::from_path(env_path).ok();
        match Self::generate() {
            Ok(s) => s,
            Err(e) => {
                println!("\n\x1b[31m{e}\x1b[0m\n");
                std::process::exit(1);
            }
        }
    }
}

/// Run tests with
///
/// cargo watch -q -c -w src/ -x 'test env_ -- --nocapture'
#[expect(clippy::unwrap_used)]
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
        }

        // random file err
        let result = AppEnv::check_file_exists("./some_random_file.txt".to_owned());
        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::FileNotFound(value) => assert_eq!(value, "./some_random_file.txt"),
            _ => unreachable!(),
        }
    }

    #[test]
    fn env_parse_log_valid() {
        // FIXTURES
        let map = HashMap::from([("RANDOM_STRING".to_owned(), "123".to_owned())]);

        // ACTION
        let result = AppEnv::parse_log(&map);

        // CHECK
        assert_eq!(result, tracing::Level::INFO);

        // FIXTURES
        let map = HashMap::from([("LOG_DEBUG".to_owned(), "false".to_owned())]);

        // ACTION
        let result = AppEnv::parse_log(&map);

        // CHECK
        assert_eq!(result, tracing::Level::INFO);

        // FIXTURES
        let map = HashMap::from([("LOG_TRACE".to_owned(), "false".to_owned())]);

        // ACTION
        let result = AppEnv::parse_log(&map);

        // CHECK
        assert_eq!(result, tracing::Level::INFO);

        // FIXTURES
        let map = HashMap::from([
            ("LOG_DEBUG".to_owned(), "false".to_owned()),
            ("LOG_TRACE".to_owned(), "false".to_owned()),
        ]);

        // ACTION
        let result = AppEnv::parse_log(&map);

        // CHECK
        assert_eq!(result, tracing::Level::INFO);

        // FIXTURES
        let map = HashMap::from([
            ("LOG_DEBUG".to_owned(), "true".to_owned()),
            ("LOG_TRACE".to_owned(), "false".to_owned()),
        ]);

        // ACTION
        let result = AppEnv::parse_log(&map);

        // CHECK
        assert_eq!(result, tracing::Level::DEBUG);

        // FIXTURES
        let map = HashMap::from([
            ("LOG_DEBUG".to_owned(), "true".to_owned()),
            ("LOG_TRACE".to_owned(), "true".to_owned()),
        ]);

        // ACTION
        let result = AppEnv::parse_log(&map);

        // CHECK
        assert_eq!(result, tracing::Level::TRACE);

        // FIXTURES
        let map = HashMap::from([
            ("LOG_DEBUG".to_owned(), "false".to_owned()),
            ("LOG_TRACE".to_owned(), "true".to_owned()),
        ]);

        // ACTION
        let result = AppEnv::parse_log(&map);

        // CHECK
        assert_eq!(result, tracing::Level::TRACE);
    }

    #[test]
    fn env_parse_timezone_ok() {
        // FIXTURES
        let mut map = HashMap::new();
        map.insert("TIMEZONE".to_owned(), "America/New_York".to_owned());

        // ACTION
        let result = AppEnv::parse_timezone(&map);

        // CHECK
        assert_eq!(result.iana_name(), Some("America/New_York"));

        let mut map = HashMap::new();
        map.insert("TIMEZONE".to_owned(), "Europe/Berlin".to_owned());

        // ACTION
        let result = AppEnv::parse_timezone(&map);

        // CHECK
        assert_eq!(result.iana_name(), Some("Europe/Berlin"));

        // FIXTURES
        let map = HashMap::new();

        // ACTION
        let result = AppEnv::parse_timezone(&map);

        // CHECK
        assert_eq!(result.iana_name(), Some("UTC"));
    }

    #[test]
    fn env_parse_timezone_err() {
        // FIXTURES
        let mut map = HashMap::new();
        map.insert("TIMEZONE".to_owned(), "america/New__York".to_owned());

        // ACTION
        let result = AppEnv::parse_timezone(&map);
        // CHECK
        assert_eq!(result.iana_name(), Some("UTC"));

        // No timezone present
        // FIXTURES
        let map = HashMap::new();
        let result = AppEnv::parse_timezone(&map);

        // CHECK
        assert_eq!(result.iana_name(), Some("UTC"));
    }
    #[test]
    fn env_parse_dl_ok() {
        // FIXTURES
        let mut map = HashMap::new();
        map.insert("DL_TIME".to_owned(), "0445".to_owned());

        // ACTION
        let result = AppEnv::parse_download_time(&map);

        assert_eq!(result, (4, 45));

        map.insert("DL_TIME".to_owned(), "2122".to_owned());

        // ACTION
        let result = AppEnv::parse_download_time(&map);

        assert_eq!(result, (21, 22));

        map.insert("DL_TIME".to_owned(), "TIME".to_owned());

        // ACTION
        let result = AppEnv::parse_download_time(&map);

        assert_eq!(result, (3, 0));

        map.insert("DL_TIME".to_owned(), "0565".to_owned());

        // ACTION
        let result = AppEnv::parse_download_time(&map);

        assert_eq!(result, (3, 0));

        map.insert("DL_TIME".to_owned(), "3115".to_owned());

        // ACTION
        let result = AppEnv::parse_download_time(&map);

        assert_eq!(result, (3, 0));

        map.insert("LL_TIME".to_owned(), "3115".to_owned());

        // ACTION
        let result = AppEnv::parse_download_time(&map);

        assert_eq!(result, (3, 0));
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
