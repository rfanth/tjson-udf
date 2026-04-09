// udf_log! is a no-op in test builds, so `msg` variables in map_err closures
// appear unused to the compiler outside of a real MariaDB environment.
#![allow(unused_variables)]
use udf::prelude::*;

#[cfg(feature = "error-log")]
fn udf_log_prefixed_lines(prefix: &str, msg: impl std::fmt::Display) {
    let msg = msg.to_string();
    let mut lines = msg.lines();
    if let Some(first_line) = lines.next() {
        let line = format!("{prefix}{first_line}");
        udf_log!(line);
        for rest in lines {
            let line = format!("{prefix}{rest}");
            udf_log!(line);
        }
    } else {
        let line = prefix.to_string();
        udf_log!(line);
    }
}

#[cfg(not(feature = "error-log"))]
fn udf_log_prefixed_lines(_prefix: &str, _msg: impl std::fmt::Display) {}

// tjson_to_json(tjson_str) -> JSON string, or NULL on error
struct TjsonToJson;

#[register]
impl BasicUdf for TjsonToJson {
    type Returns<'a> = Option<String>;

    fn init(_cfg: &UdfCfg<Init>, args: &ArgList<Init>) -> Result<Self, String> {
        if args.len() != 1 {
            return Err("tjson_to_json() requires exactly 1 argument".into());
        }
        args.get(0).unwrap().set_type_coercion(SqlType::String);
        Ok(Self)
    }

    fn process<'a>(
        &'a mut self,
        _cfg: &UdfCfg<Process>,
        args: &ArgList<Process>,
        _error: Option<NonZeroU8>,
    ) -> Result<Self::Returns<'a>, ProcessError> {
        let arg = args.get(0).unwrap().value();
        let input = match arg.as_string() {
            Some(s) => s,
            None => return Ok(None),
        };

        let val: serde_json::Value = tjson::from_str(input).map_err(|e| {
            udf_log_prefixed_lines("tjson_to_json parse error: ", e);
            ProcessError
        })?;

        serde_json::to_string(&val)
            .map(Some)
            .map_err(|e| {
                udf_log_prefixed_lines("tjson_to_json serialize error: ", e);
                ProcessError
            })
    }
}

// tjson_to_json_err(tjson_str) -> NULL on success, error string on failure
struct TjsonToJsonErr;

#[register]
impl BasicUdf for TjsonToJsonErr {
    type Returns<'a> = Option<String>;

    fn init(_cfg: &UdfCfg<Init>, args: &ArgList<Init>) -> Result<Self, String> {
        if args.len() != 1 {
            return Err("tjson_to_json_err() requires exactly 1 argument".into());
        }
        args.get(0).unwrap().set_type_coercion(SqlType::String);
        Ok(Self)
    }

    fn process<'a>(
        &'a mut self,
        _cfg: &UdfCfg<Process>,
        args: &ArgList<Process>,
        _error: Option<NonZeroU8>,
    ) -> Result<Self::Returns<'a>, ProcessError> {
        let arg = args.get(0).unwrap().value();
        let input = match arg.as_string() {
            Some(s) => s,
            None => return Ok(None),
        };

        match tjson::from_str::<serde_json::Value>(input) {
            Ok(_) => Ok(None),
            Err(e) => Ok(Some(e.to_string())),
        }
    }
}

// json_to_tjson(json_str[, options_json]) -> TJSON string, or NULL on error
struct JsonToTjson;

#[register]
impl BasicUdf for JsonToTjson {
    type Returns<'a> = Option<String>;

    fn init(_cfg: &UdfCfg<Init>, args: &ArgList<Init>) -> Result<Self, String> {
        if args.is_empty() || args.len() > 2 {
            return Err("json_to_tjson() requires 1 or 2 arguments".into());
        }
        args.get(0).unwrap().set_type_coercion(SqlType::String);
        if let Some(mut a) = args.get(1) {
            a.set_type_coercion(SqlType::String);
        }
        Ok(Self)
    }

    fn process<'a>(
        &'a mut self,
        _cfg: &UdfCfg<Process>,
        args: &ArgList<Process>,
        _error: Option<NonZeroU8>,
    ) -> Result<Self::Returns<'a>, ProcessError> {
        let arg = args.get(0).unwrap().value();
        let input = match arg.as_string() {
            Some(s) => s,
            None => return Ok(None),
        };

        let json: serde_json::Value = serde_json::from_str(input).map_err(|e| {
            udf_log_prefixed_lines("json_to_tjson parse error: ", e);
            ProcessError
        })?;

        let opts = parse_tjson_opts(&args, "json_to_tjson")?;

        tjson::to_string_with(&json, opts)
            .map(Some)
            .map_err(|e| {
                udf_log_prefixed_lines("json_to_tjson serialize error: ", e);
                ProcessError
            })
    }
}

// json_to_tjson_err(json_str[, options_json]) -> NULL on success, error string on failure
struct JsonToTjsonErr;

#[register]
impl BasicUdf for JsonToTjsonErr {
    type Returns<'a> = Option<String>;

    fn init(_cfg: &UdfCfg<Init>, args: &ArgList<Init>) -> Result<Self, String> {
        if args.is_empty() || args.len() > 2 {
            return Err("json_to_tjson_err() requires 1 or 2 arguments".into());
        }
        args.get(0).unwrap().set_type_coercion(SqlType::String);
        if let Some(mut a) = args.get(1) {
            a.set_type_coercion(SqlType::String);
        }
        Ok(Self)
    }

    fn process<'a>(
        &'a mut self,
        _cfg: &UdfCfg<Process>,
        args: &ArgList<Process>,
        _error: Option<NonZeroU8>,
    ) -> Result<Self::Returns<'a>, ProcessError> {
        let arg = args.get(0).unwrap().value();
        let input = match arg.as_string() {
            Some(s) => s,
            None => return Ok(None),
        };

        let json: serde_json::Value = match serde_json::from_str(input) {
            Ok(v) => v,
            Err(e) => return Ok(Some(e.to_string())),
        };

        let opts = match parse_tjson_opts(&args, "json_to_tjson_err") {
            Ok(o) => o,
            Err(_) => return Ok(Some("failed to parse options".into())),
        };

        match tjson::to_string_with(&json, opts) {
            Ok(_) => Ok(None),
            Err(e) => Ok(Some(e.to_string())),
        }
    }
}

// tjson_options_check(opts_json) -> NULL if valid, error string if invalid
struct TjsonOptionsCheck;

#[register]
impl BasicUdf for TjsonOptionsCheck {
    type Returns<'a> = Option<String>;

    fn init(_cfg: &UdfCfg<Init>, args: &ArgList<Init>) -> Result<Self, String> {
        if args.len() != 1 {
            return Err("tjson_options_check() requires exactly 1 argument".into());
        }
        args.get(0).unwrap().set_type_coercion(SqlType::String);
        Ok(Self)
    }

    fn process<'a>(
        &'a mut self,
        _cfg: &UdfCfg<Process>,
        args: &ArgList<Process>,
        _error: Option<NonZeroU8>,
    ) -> Result<Self::Returns<'a>, ProcessError> {
        let arg = args.get(0).unwrap().value();
        let Some(input) = arg.as_string() else {
            return Ok(None);
        };
        match serde_json::from_str::<tjson::TjsonConfig>(input) {
            Ok(_) => Ok(None),
            Err(e) => Ok(Some(e.to_string())),
        }
    }
}

fn parse_tjson_opts(args: &ArgList<Process>, fn_name: &str) -> Result<tjson::TjsonOptions, ProcessError> {
    if let Some(opts_arg) = args.get(1) {
        let opts_val = opts_arg.value();
        if let Some(opts_str) = opts_val.as_string() {
            let cfg: tjson::TjsonConfig = serde_json::from_str(opts_str).map_err(|e| {
                let prefix = format!("{fn_name} options parse error: ");
                udf_log_prefixed_lines(&prefix, e);
                ProcessError
            })?;
            return Ok(tjson::TjsonOptions::from(cfg));
        }
    }
    Ok(tjson::TjsonOptions::default())
}

#[cfg(test)]
mod tests {
    use super::*;
    use udf::mock::{mock_args, MockUdfCfg};

    // --- TjsonToJson tests ---

    #[test]
    fn tjson_to_json_init_ok() {
        let mut cfg = MockUdfCfg::new();
        let mut args = mock_args![("  name: Alice", "s", false)];
        assert!(TjsonToJson::init(cfg.as_init(), args.as_init()).is_ok());
    }

    #[test]
    fn tjson_to_json_init_wrong_arg_count() {
        let mut cfg = MockUdfCfg::new();
        let mut args = mock_args![
            ("  name: Alice", "s", false),
            ("extra", "s2", false)
        ];
        assert!(TjsonToJson::init(cfg.as_init(), args.as_init()).is_err());
    }

    #[test]
    fn tjson_to_json_process_ok() {
        let mut udf = TjsonToJson;
        let mut cfg = MockUdfCfg::new();
        let mut args = mock_args![("  name: Alice\n  age:30", "s", false)];
        let result = TjsonToJson::process(&mut udf, cfg.as_process(), args.as_process(), None);
        let json = result.expect("process failed").expect("got NULL");
        let val: serde_json::Value = serde_json::from_str(&json).expect("not valid json");
        assert_eq!(val["name"], "Alice");
        assert_eq!(val["age"], 30);
    }

    #[test]
    fn tjson_to_json_process_null_input() {
        let mut udf = TjsonToJson;
        let mut cfg = MockUdfCfg::new();
        let mut args = mock_args![(String None, "s", true)];
        let result = TjsonToJson::process(&mut udf, cfg.as_process(), args.as_process(), None);
        assert_eq!(result.unwrap(), None);
    }

    // --- TjsonToJsonErr tests ---

    #[test]
    fn tjson_to_json_err_valid_input_returns_null() {
        let mut udf = TjsonToJsonErr;
        let mut cfg = MockUdfCfg::new();
        let mut args = mock_args![("  name: Alice", "s", false)];
        let result = TjsonToJsonErr::process(&mut udf, cfg.as_process(), args.as_process(), None);
        assert_eq!(result.unwrap(), None);
    }

    #[test]
    fn tjson_to_json_err_invalid_input_returns_message() {
        let mut udf = TjsonToJsonErr;
        let mut cfg = MockUdfCfg::new();
        let mut args = mock_args![("{{{{not tjson", "s", false)];
        let result = TjsonToJsonErr::process(&mut udf, cfg.as_process(), args.as_process(), None);
        assert!(result.unwrap().is_some());
    }

    #[test]
    fn tjson_to_json_err_null_input_returns_null() {
        let mut udf = TjsonToJsonErr;
        let mut cfg = MockUdfCfg::new();
        let mut args = mock_args![(String None, "s", true)];
        let result = TjsonToJsonErr::process(&mut udf, cfg.as_process(), args.as_process(), None);
        assert_eq!(result.unwrap(), None);
    }

    // --- JsonToTjson tests ---

    #[test]
    fn json_to_tjson_init_ok_one_arg() {
        let mut cfg = MockUdfCfg::new();
        let mut args = mock_args![(r#"{"name":"Alice"}"#, "s", false)];
        assert!(JsonToTjson::init(cfg.as_init(), args.as_init()).is_ok());
    }

    #[test]
    fn json_to_tjson_init_ok_two_args() {
        let mut cfg = MockUdfCfg::new();
        let mut args = mock_args![
            (r#"{"name":"Alice"}"#, "s", false),
            (r#"{"canonical":true}"#, "opts", false)
        ];
        assert!(JsonToTjson::init(cfg.as_init(), args.as_init()).is_ok());
    }

    #[test]
    fn json_to_tjson_init_wrong_arg_count() {
        let mut cfg = MockUdfCfg::new();
        let mut args = mock_args![];
        assert!(JsonToTjson::init(cfg.as_init(), args.as_init()).is_err());
    }

    #[test]
    fn json_to_tjson_process_ok() {
        let mut udf = JsonToTjson;
        let mut cfg = MockUdfCfg::new();
        let mut args = mock_args![(r#"{"name":"Alice","age":30}"#, "s", false)];
        let result = JsonToTjson::process(&mut udf, cfg.as_process(), args.as_process(), None);
        assert!(result.unwrap().is_some());
    }

    #[test]
    fn json_to_tjson_process_with_canonical() {
        let mut udf = JsonToTjson;
        let mut cfg = MockUdfCfg::new();
        let mut args = mock_args![
            (r#"{"name":"Alice"}"#, "s", false),
            (r#"{"canonical":true}"#, "opts", false)
        ];
        let result = JsonToTjson::process(&mut udf, cfg.as_process(), args.as_process(), None);
        assert!(result.unwrap().is_some());
    }

    #[test]
    fn json_to_tjson_process_invalid_json() {
        let mut udf = JsonToTjson;
        let mut cfg = MockUdfCfg::new();
        let mut args = mock_args![("not json {{{", "s", false)];
        let result = JsonToTjson::process(&mut udf, cfg.as_process(), args.as_process(), None);
        assert!(result.is_err());
    }

    #[test]
    fn json_to_tjson_process_invalid_options() {
        let mut udf = JsonToTjson;
        let mut cfg = MockUdfCfg::new();
        let mut args = mock_args![
            (r#"{"name":"Alice"}"#, "s", false),
            ("not valid json", "opts", false)
        ];
        let result = JsonToTjson::process(&mut udf, cfg.as_process(), args.as_process(), None);
        assert!(result.is_err());
    }

    #[test]
    fn json_to_tjson_process_null_input() {
        let mut udf = JsonToTjson;
        let mut cfg = MockUdfCfg::new();
        let mut args = mock_args![(String None, "s", true)];
        let result = JsonToTjson::process(&mut udf, cfg.as_process(), args.as_process(), None);
        assert_eq!(result.unwrap(), None);
    }

    // --- JsonToTjsonErr tests ---

    #[test]
    fn json_to_tjson_err_valid_input_returns_null() {
        let mut udf = JsonToTjsonErr;
        let mut cfg = MockUdfCfg::new();
        let mut args = mock_args![(r#"{"name":"Alice"}"#, "s", false)];
        let result = JsonToTjsonErr::process(&mut udf, cfg.as_process(), args.as_process(), None);
        assert_eq!(result.unwrap(), None);
    }

    #[test]
    fn json_to_tjson_err_invalid_input_returns_message() {
        let mut udf = JsonToTjsonErr;
        let mut cfg = MockUdfCfg::new();
        let mut args = mock_args![("not json {{{", "s", false)];
        let result = JsonToTjsonErr::process(&mut udf, cfg.as_process(), args.as_process(), None);
        assert!(result.unwrap().is_some());
    }

    #[test]
    fn json_to_tjson_err_null_input_returns_null() {
        let mut udf = JsonToTjsonErr;
        let mut cfg = MockUdfCfg::new();
        let mut args = mock_args![(String None, "s", true)];
        let result = JsonToTjsonErr::process(&mut udf, cfg.as_process(), args.as_process(), None);
        assert_eq!(result.unwrap(), None);
    }

    // --- TjsonOptionsCheck tests ---

    #[test]
    fn options_check_valid() {
        let mut udf = TjsonOptionsCheck;
        let mut cfg = MockUdfCfg::new();
        let mut args = mock_args![(r#"{"canonical":true}"#, "opts", false)];
        let result = TjsonOptionsCheck::process(&mut udf, cfg.as_process(), args.as_process(), None);
        assert_eq!(result.unwrap(), None);
    }

    #[test]
    fn options_check_empty_object() {
        let mut udf = TjsonOptionsCheck;
        let mut cfg = MockUdfCfg::new();
        let mut args = mock_args![("{}", "opts", false)];
        let result = TjsonOptionsCheck::process(&mut udf, cfg.as_process(), args.as_process(), None);
        assert_eq!(result.unwrap(), None);
    }

    #[test]
    fn options_check_invalid_returns_message() {
        let mut udf = TjsonOptionsCheck;
        let mut cfg = MockUdfCfg::new();
        let mut args = mock_args![(r#"{"canonical":"true"}"#, "opts", false)];
        let result = TjsonOptionsCheck::process(&mut udf, cfg.as_process(), args.as_process(), None);
        assert!(result.unwrap().is_some());
    }

    #[test]
    fn options_check_null_input() {
        let mut udf = TjsonOptionsCheck;
        let mut cfg = MockUdfCfg::new();
        let mut args = mock_args![(String None, "opts", true)];
        let result = TjsonOptionsCheck::process(&mut udf, cfg.as_process(), args.as_process(), None);
        assert_eq!(result.unwrap(), None);
    }
}
