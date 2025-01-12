use nu_protocol::ast::Call;
use nu_protocol::engine::{Command, EngineState, Stack};
use nu_protocol::{
    Category, Example, IntoPipelineData, PipelineData, ShellError, Signature, Span, Type, Value,
};

#[derive(Clone)]
pub struct ToUrl;

impl Command for ToUrl {
    fn name(&self) -> &str {
        "to url"
    }

    fn signature(&self) -> Signature {
        Signature::build("to url")
            .input_output_types(vec![
                (Type::Record(vec![]), Type::String),
                (Type::Table(vec![]), Type::String),
            ])
            .category(Category::Formats)
    }

    fn usage(&self) -> &str {
        "Convert record or table into URL-encoded text"
    }

    fn examples(&self) -> Vec<Example> {
        vec![
            Example {
                description: "Outputs a URL string representing the contents of this record",
                example: r#"{ mode:normal userid:31415 } | to url"#,
                result: Some(Value::test_string("mode=normal&userid=31415")),
            },
            Example {
                description: "Outputs a URL string representing the contents of this 1-row table",
                example: r#"[[foo bar]; ["1" "2"]] | to url"#,
                result: Some(Value::test_string("foo=1&bar=2")),
            },
        ]
    }

    fn run(
        &self,
        _engine_state: &EngineState,
        _stack: &mut Stack,
        call: &Call,
        input: PipelineData,
    ) -> Result<nu_protocol::PipelineData, ShellError> {
        let head = call.head;
        to_url(input, head)
    }
}

fn to_url(input: PipelineData, head: Span) -> Result<PipelineData, ShellError> {
    let output: Result<String, ShellError> = input
        .into_iter()
        .map(move |value| match value {
            Value::Record {
                ref cols,
                ref vals,
                span,
            } => {
                let mut row_vec = vec![];
                for (k, v) in cols.iter().zip(vals.iter()) {
                    match v.as_string() {
                        Ok(s) => {
                            row_vec.push((k.clone(), s.to_string()));
                        }
                        _ => {
                            return Err(ShellError::UnsupportedInput(
                                "Expected a record with string values".to_string(),
                                "value originates from here".into(),
                                head,
                                span,
                            ));
                        }
                    }
                }

                match serde_urlencoded::to_string(row_vec) {
                    Ok(s) => Ok(s),
                    _ => Err(ShellError::CantConvert(
                        "URL".into(),
                        value.get_type().to_string(),
                        head,
                        None,
                    )),
                }
            }
            // Propagate existing errors
            Value::Error { error } => Err(error),
            other => Err(ShellError::UnsupportedInput(
                "Expected a table from pipeline".to_string(),
                "value originates from here".into(),
                head,
                other.expect_span(),
            )),
        })
        .collect();

    Ok(Value::string(output?, head).into_pipeline_data())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_examples() {
        use crate::test_examples;

        test_examples(ToUrl {})
    }
}
