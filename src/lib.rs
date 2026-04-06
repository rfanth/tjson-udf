// udf_log! is a no-op in test builds, so `msg` variables in map_err closures
// appear unused to the compiler outside of a real MariaDB environment.
#![allow(unused_variables)]
use udf::prelude::*;

// tjson_to_json_str(tjson_str) -> JSON string
struct TjsonToJsonStr;

#[register]
impl BasicUdf for TjsonToJsonStr {
    type Returns<'a> = String;

    fn init(_cfg: &UdfCfg<Init>, args: &ArgList<Init>) -> Result<Self, String> {
        if args.len() != 1 {
            return Err("tjson_to_json_str() requires exactly 1 argument".into());
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
        let input = arg.as_string().ok_or(ProcessError)?;

        let val: serde_json::Value = tjson::from_str(input).map_err(|e| {
            let msg = e.to_string();
            udf_log!("tjson_to_json_str parse error: {msg}");
            ProcessError
        })?;

        serde_json::to_string(&val).map_err(|e| {
            let msg = e.to_string();
            udf_log!("tjson_to_json_str serialize error: {msg}");
            ProcessError
        })
    }
}

// json_to_tjson_str(json_str[, options_json]) -> TJSON string
struct JsonToTjsonStr;

#[register]
impl BasicUdf for JsonToTjsonStr {
    type Returns<'a> = String;

    fn init(_cfg: &UdfCfg<Init>, args: &ArgList<Init>) -> Result<Self, String> {
        if args.is_empty() || args.len() > 2 {
            return Err("json_to_tjson_str() requires 1 or 2 arguments".into());
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
        let input = arg.as_string().ok_or(ProcessError)?;

        let json: serde_json::Value = serde_json::from_str(input).map_err(|e| {
            let msg = e.to_string();
            udf_log!("json_to_tjson_str parse error: {msg}");
            ProcessError
        })?;

        let opts = if let Some(opts_arg) = args.get(1) {
            let opts_val = opts_arg.value();
            if let Some(opts_str) = opts_val.as_string() {
                let cfg: tjson::TjsonConfig = serde_json::from_str(opts_str).map_err(|e| {
                    let msg = e.to_string();
                    udf_log!("json_to_tjson_str options parse error: {msg}");
                    ProcessError
                })?;
                tjson::TjsonOptions::from(cfg)
            } else {
                tjson::TjsonOptions::default()
            }
        } else {
            tjson::TjsonOptions::default()
        };

        tjson::to_string_with(&json, opts).map_err(|e| {
            let msg = e.to_string();
            udf_log!("json_to_tjson_str serialize error: {msg}");
            ProcessError
        })
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
            return Ok(Some("options argument is NULL".into()));
        };
        match serde_json::from_str::<tjson::TjsonConfig>(input) {
            Ok(_) => Ok(None),
            Err(e) => Ok(Some(e.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use udf::mock::{mock_args, MockUdfCfg};

    // --- TjsonToJsonStr tests ---

    #[test]
    fn tjson_to_json_init_ok() {
        let mut cfg = MockUdfCfg::new();
        let mut args = mock_args![("  name: Alice", "s", false)];
        assert!(TjsonToJsonStr::init(cfg.as_init(), args.as_init()).is_ok());
    }

    #[test]
    fn tjson_to_json_init_wrong_arg_count() {
        let mut cfg = MockUdfCfg::new();
        let mut args = mock_args![
            ("  name: Alice", "s", false),
            ("extra", "s2", false)
        ];
        assert!(TjsonToJsonStr::init(cfg.as_init(), args.as_init()).is_err());
    }

    #[test]
    fn tjson_to_json_process_ok() {
        let mut udf = TjsonToJsonStr;
        let mut cfg = MockUdfCfg::new();
        let mut args = mock_args![("  name: Alice\n  age:30", "s", false)];
        let result = TjsonToJsonStr::process(&mut udf, cfg.as_process(), args.as_process(), None);
        let json = result.expect("process failed");
        let val: serde_json::Value = serde_json::from_str(&json).expect("not valid json");
        assert_eq!(val["name"], "Alice");
        assert_eq!(val["age"], 30);
    }

    #[test]
    fn tjson_to_json_process_null_input() {
        let mut udf = TjsonToJsonStr;
        let mut cfg = MockUdfCfg::new();
        let mut args = mock_args![(String None, "s", true)];
        let result = TjsonToJsonStr::process(&mut udf, cfg.as_process(), args.as_process(), None);
        assert!(result.is_err());
    }

    // --- JsonToTjsonStr tests ---

    #[test]
    fn json_to_tjson_init_ok_one_arg() {
        let mut cfg = MockUdfCfg::new();
        let mut args = mock_args![(r#"{"name":"Alice"}"#, "s", false)];
        assert!(JsonToTjsonStr::init(cfg.as_init(), args.as_init()).is_ok());
    }

    #[test]
    fn json_to_tjson_init_ok_two_args() {
        let mut cfg = MockUdfCfg::new();
        let mut args = mock_args![
            (r#"{"name":"Alice"}"#, "s", false),
            (r#"{"canonical":true}"#, "opts", false)
        ];
        assert!(JsonToTjsonStr::init(cfg.as_init(), args.as_init()).is_ok());
    }

    #[test]
    fn json_to_tjson_init_wrong_arg_count() {
        let mut cfg = MockUdfCfg::new();
        let mut args = mock_args![];
        assert!(JsonToTjsonStr::init(cfg.as_init(), args.as_init()).is_err());
    }

    #[test]
    fn json_to_tjson_process_ok() {
        let mut udf = JsonToTjsonStr;
        let mut cfg = MockUdfCfg::new();
        let mut args = mock_args![(r#"{"name":"Alice","age":30}"#, "s", false)];
        let result = JsonToTjsonStr::process(&mut udf, cfg.as_process(), args.as_process(), None);
        assert!(result.is_ok());
    }

    #[test]
    fn json_to_tjson_process_with_canonical() {
        let mut udf = JsonToTjsonStr;
        let mut cfg = MockUdfCfg::new();
        let mut args = mock_args![
            (r#"{"name":"Alice"}"#, "s", false),
            (r#"{"canonical":true}"#, "opts", false)
        ];
        let result = JsonToTjsonStr::process(&mut udf, cfg.as_process(), args.as_process(), None);
        assert!(result.is_ok());
    }

    #[test]
    fn json_to_tjson_process_invalid_json() {
        let mut udf = JsonToTjsonStr;
        let mut cfg = MockUdfCfg::new();
        let mut args = mock_args![("not json {{{", "s", false)];
        let result = JsonToTjsonStr::process(&mut udf, cfg.as_process(), args.as_process(), None);
        assert!(result.is_err());
    }

    #[test]
    fn json_to_tjson_process_invalid_options() {
        let mut udf = JsonToTjsonStr;
        let mut cfg = MockUdfCfg::new();
        let mut args = mock_args![
            (r#"{"name":"Alice"}"#, "s", false),
            ("not valid json", "opts", false)
        ];
        let result = JsonToTjsonStr::process(&mut udf, cfg.as_process(), args.as_process(), None);
        assert!(result.is_err());
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
        assert!(result.unwrap().is_some());
    }

    #[test]
    fn json_to_tjson_process_null_input() {
        let mut udf = JsonToTjsonStr;
        let mut cfg = MockUdfCfg::new();
        let mut args = mock_args![(String None, "s", true)];
        let result = JsonToTjsonStr::process(&mut udf, cfg.as_process(), args.as_process(), None);
        assert!(result.is_err());
    }
}
