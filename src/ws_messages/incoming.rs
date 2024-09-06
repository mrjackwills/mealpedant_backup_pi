use serde::{Deserialize, Serialize};
use tracing::debug;

#[derive(Debug)]
pub enum MessageValues {
    Valid(ParsedMessage),
    Invalid(ErrorData),
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case", tag = "name", content = "body")]
pub enum ParsedMessage {
    BackupData(BackupData),
}

#[derive(Deserialize, Debug, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BackupData {
    pub file_name: String,
    pub file_as_b64: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
struct StructuredMessage {
    data: Option<ParsedMessage>,
    error: Option<ErrorData>,
}

// TODO
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case", tag = "error", content = "message")]
pub enum ErrorData {
    Something(String),
}

pub fn to_struct(input: &str) -> Option<MessageValues> {
    if let Ok(data) = serde_json::from_str::<StructuredMessage>(input) {
        if let Some(data) = data.error {
            return Some(MessageValues::Invalid(data));
        }
        if let Some(data) = data.data {
            return Some(MessageValues::Valid(data));
        }
        None
    } else if let Ok(data) = serde_json::from_str::<ErrorData>(input) {
        debug!("Matched error_serialized data");
        Some(MessageValues::Invalid(data))
    } else {
        debug!(input);
        debug!("not a known input message");
        None
    }
}

/// message_incoming
///
/// cargo watch -q -c -w src/ -x 'test message_incoming -- --test-threads=1 --nocapture'
#[expect(clippy::unwrap_used, clippy::pedantic)]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn message_incoming_parse_invalid() {
        let data = r#""#;
        let result = to_struct(data);
        assert!(result.is_none());

        let data = r#"{}"#;
        let result = to_struct(data);
        assert!(result.is_none());
    }

    #[test]
    fn message_incoming_parse_backup_data_ok() {
        let data = r#"
            {
            	"data": {
            		"name" : "backup_data",
            		"body": {
            			"file_name":"some_file_name",
						"file_as_b64": "aGVsbG8gd29ybGQ="
            		}
            	},
				"unique": true
            }"#;
        let result = to_struct(data);
        assert!(result.is_some());
        let result = result.unwrap();
        match result {
            MessageValues::Valid(ParsedMessage::BackupData(data)) => {
                assert_eq!(data.file_name, "some_file_name");
                assert_eq!(data.file_as_b64, "aGVsbG8gd29ybGQ=");
            }
            _ => unreachable!("Shouldn't have matched this"),
        };
    }

    #[test]
    fn message_incoming_invalid_parse_backup_data() {
        // No body
        let data = r#"
			 {
				 "data": {
					 "name" : "backup_data",
				 },
				 "unique": true
			 }"#;
        let result = to_struct(data);
        assert!(result.is_none());

        // invalid message name
        let data = r#"
			 {
				 "data": {
					 "name" : "invalid_name",
					 "body": {
            			"file_name":"some_file_name",
						"file_as_b64": "aGVsbG8gd29ybGQ="
            		}
				 },
				 "unique": true
			 }"#;
        let result = to_struct(data);
        assert!(result.is_none());

        // Empty body
        let data = r#"
    		{
    			"data": {
    				"name" : "backup_data",
    				"body: "",
    			},
				"unique": true
    		}"#;
        let result = to_struct(data);
        assert!(result.is_none());

        // Empty body object
        let data = r#"
    		{
    			"data": {
    				"name" : "backup_data",
    				"body: {},
    			},
				"unique": true
    		}"#;
        let result = to_struct(data);
        assert!(result.is_none());

        // No file_name
        let data = r#"
    		  {
    			  "data": {
    				  "name" : "backup_data",
    				  "body": {
						"file_as_b64": "aGVsbG8gd29ybGQ="
            		}
    			  },
				  "unique": true
    		  }"#;
        let result = to_struct(data);
        assert!(result.is_none());

        // no file_as_b64
        let data = r#"
		{
			"data": {
				"name" : "backup_data",
				"body": {
					"file_name": "file_name",
			  }
			},
			"unique": true
		}"#;
        let result = to_struct(data);
        assert!(result.is_none());

        // additional data field
        let data = r#"
		{
			"data": {
				"name" : "backup_data",
				"body": {
					"file_name": "file_name",
					"file_as_b64": "d",
					"additional":"field"
			  },
			},
			"unique": true
		}"#;
        let result = to_struct(data);
        assert!(result.is_none());

        // missing unique
        let data = r#"
		  {
			  "data": {
				  "name" : "backup_data",
				  "body": {
					  "file_name": "file_name",
					  "file_as_b64": "d",
				}
			  }
		  }"#;
        let result = to_struct(data);
        assert!(result.is_none());

        // unique string
        let data = r#"
				{
					"data": {
						"name" : "backup_data",
						"body": {
							"file_name": "file_name",
							"file_as_b64": "d",
						  }
					},
					"unique": true
				}"#;
        let result = to_struct(data);
        assert!(result.is_none());
    }
}
