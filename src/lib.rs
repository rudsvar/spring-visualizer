use nom::{
    bytes::complete::is_not,
    bytes::complete::{tag, take_until, take_while},
    character::complete::char,
    sequence::delimited,
    IResult,
};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    String(String),
    Class(String),
    Array(Vec<Value>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Values {
    Single(Value),
    Multi(HashMap<String, Value>),
}

/// @Annotation
/// @Annotation()
/// @Annotation(Test.class)
/// @Annotation("string")
/// @Annotation(value = "string")
/// @Annotation(value = {"string"})
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Annotation {
    name: String,
    values: Values,
}

pub fn parse_value(input: &str) -> IResult<&str, Value> {
    eprintln!("Parsing value from {}", input);
    let input = input.trim_start();
    let is_string = input.starts_with('"');
    let is_array = input.starts_with('{') && input.ends_with('}');
    let res = if is_string {
        let (input, between) = delimited(char('"'), is_not("\""), char('"'))(input)?;
        (input, Value::String(between.to_string()))
    } else if is_array {
        let (input, between) = delimited(char('{'), is_not("}"), char('}'))(input)?;
        let (_, values) = nom::multi::separated_list0(tag(","), parse_value)(between)?;
        (input, Value::Array(values))
    } else {
        let (input, name) = take_until(".class")(input)?;
        let (input, _) = tag(".class")(input)?;
        (input, Value::Class(name.to_string()))
    };
    Ok(res)
}

pub fn parse_annotation(input: &str) -> IResult<&str, Annotation> {
    let (input, _) = tag("@")(input)?;
    let (input, name) = take_while(|c: char| c.is_alphanumeric())(input)?;
    let (input, between) = delimited(char('('), is_not(")"), char(')'))(input)?;
    let (_, value) = parse_value(between)?;

    Ok((
        input,
        Annotation {
            name: name.to_string(),
            values: Values::Single(value),
        },
    ))
}

#[cfg(test)]
mod tests {
    use crate::{parse_annotation, Annotation, Value, Values};

    #[test]
    pub fn parse_annotation_with_class_value_succeeds() {
        assert_eq!(
            Ok((
                "",
                Annotation {
                    name: "MyAnnotation".to_string(),
                    values: Values::Single(Value::Class("Test".to_string()))
                }
            )),
            parse_annotation("@MyAnnotation(Test.class)")
        );
    }

    #[test]
    pub fn parse_annotation_with_string_value_succeeds() {
        assert_eq!(
            Ok((
                "",
                Annotation {
                    name: "MyAnnotation".to_string(),
                    values: Values::Single(Value::String("com.example".to_string()))
                }
            )),
            parse_annotation("@MyAnnotation(\"com.example\")")
        );
    }

    #[test]
    pub fn parse_annotation_with_array_value_succeeds() {
        assert_eq!(
            Ok((
                "",
                Annotation {
                    name: "MyAnnotation".to_string(),
                    values: Values::Single(Value::Array(vec![
                        Value::String("com.example".to_string()),
                        Value::Class("Foo".to_string())
                    ]))
                }
            )),
            parse_annotation("@MyAnnotation({\"com.example\", Foo.class})")
        );
    }
}
