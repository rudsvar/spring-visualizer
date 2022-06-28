use nom::{
    bytes::complete::is_not,
    bytes::complete::{tag, take_until, take_while},
    character::complete::{char, multispace0, space0},
    combinator::{map, opt},
    multi::many0,
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
pub enum Args {
    Single(Value),
    Multi(HashMap<String, Value>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Annotation {
    name: String,
    args: Args,
}

impl Annotation {
    pub fn value(&self) -> Option<&Value> {
        match &self.args {
            Args::Single(v) => Some(v),
            Args::Multi(vs) => vs.get("value"),
        }
    }
}

pub fn parse_single_value(input: &str) -> IResult<&str, Value> {
    let is_string = input.starts_with('"');
    let is_array = input.starts_with('{') && input.ends_with('}');
    let res = if is_string {
        let (input, between) = delimited(char('"'), is_not("\""), char('"'))(input)?;
        (input, Value::String(between.to_string()))
    } else if is_array {
        let (input, between) = delimited(char('{'), is_not("}"), char('}'))(input)?;
        let (_, values) = nom::multi::separated_list0(
            |input| {
                let (input, _) = tag(",")(input)?;
                let (input, _) = space0(input)?;
                Ok((input, ()))
            },
            parse_single_value,
        )(between)?;
        (input, Value::Array(values))
    } else {
        let (input, name) = take_until(".class")(input)?;
        let (input, _) = tag(".class")(input)?;
        (input, Value::Class(name.to_string()))
    };
    Ok(res)
}

pub fn parse_multi_value(input: &str) -> IResult<&str, HashMap<String, Value>> {
    eprintln!("Parsing multi value from {}", input);
    let (input, key_value_pairs) = many0(|input| {
        let (input, key) = take_while(|c: char| c.is_alphanumeric())(input)?;
        let (input, _) = space0(input)?;
        let (input, _) = tag("=")(input)?;
        let (input, _) = space0(input)?;
        let (input, value) = parse_single_value(input)?;
        let (input, _) = opt(tag(","))(input)?;
        let (input, _) = space0(input)?;
        Ok((input, (key.to_string(), value)))
    })(input)?;
    dbg!(&key_value_pairs);
    let values = key_value_pairs.into_iter().collect();
    Ok((input, values))
}

pub fn parse_args(input: &str) -> IResult<&str, Args> {
    if input.contains('=') {
        map(parse_multi_value, Args::Multi)(input)
    } else {
        map(parse_single_value, Args::Single)(input)
    }
}

pub fn parse_annotation(input: &str) -> IResult<&str, Annotation> {
    let (input, _) = tag("@")(input)?;
    let (input, name) = take_while(|c: char| c.is_alphanumeric())(input)?;
    let (input, _) = multispace0(input)?;

    // Parse args if there are any
    let (input, args) = if input.starts_with('(') {
        // Parse args between parentheses
        let (input, between) = delimited(char('('), is_not(")"), char(')'))(input)?;
        let (_, args) = parse_args(between)?;
        let (input, _) = multispace0(input)?;
        (input, args)
    } else {
        // No args
        (input, Args::Multi(HashMap::new()))
    };

    Ok((
        input,
        Annotation {
            name: name.to_string(),
            args,
        },
    ))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bean {
    name: String,
    class: String,
}

pub fn parse_bean(input: &str) -> IResult<&str, Bean> {
    let (input, annotation) = parse_annotation(input)?;
    // Skip visibility modifier
    let (input, _) = opt(tag("public"))(input)?;
    let (input, _) = opt(tag("protected"))(input)?;
    let (input, _) = opt(tag("private"))(input)?;
    let (input, _) = space0(input)?;
    // Get return type
    let (input, class) = take_while(|c: char| c.is_alphanumeric())(input)?;
    let (input, _) = multispace0(input)?;
    // Get method name
    let (input, name) = take_while(|c: char| c.is_alphanumeric())(input)?;
    // See if name has been overridden
    let overriden_name = match annotation.value() {
        Some(Value::String(name)) => Some(name.clone()),
        _ => None,
    };
    Ok((
        input,
        Bean {
            name: overriden_name.unwrap_or_else(|| name.to_string()),
            class: class.to_string(),
        },
    ))
}

#[cfg(test)]
mod tests {
    use crate::{parse_annotation, parse_bean, Annotation, Args, Bean, Value};

    #[test]
    pub fn parse_annotation_with_class_value_succeeds() {
        assert_eq!(
            Ok((
                "",
                Annotation {
                    name: "MyAnnotation".to_string(),
                    args: Args::Single(Value::Class("Test".to_string()))
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
                    args: Args::Single(Value::String("com.example".to_string()))
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
                    args: Args::Single(Value::Array(vec![
                        Value::String("com.example".to_string()),
                        Value::Class("Foo".to_string())
                    ]))
                }
            )),
            parse_annotation("@MyAnnotation({\"com.example\", Foo.class})")
        );
    }

    #[test]
    pub fn parse_annotation_with_key_value_pairs_succeeds() {
        assert_eq!(
            Ok((
                "",
                Annotation {
                    name: "MyAnnotation".to_string(),
                    args: Args::Multi(
                        vec![
                            ("foo".to_string(), Value::Class("Foo".to_string())),
                            ("bar".to_string(), Value::String("com.example".to_string())),
                            (
                                "baz".to_string(),
                                Value::Array(vec![
                                    Value::Class("A".to_string()),
                                    Value::Class("B".to_string())
                                ])
                            )
                        ]
                        .into_iter()
                        .collect()
                    )
                }
            )),
            parse_annotation(
                "@MyAnnotation(foo = Foo.class, bar = \"com.example\", baz = {A.class, B.class})"
            )
        );
    }

    #[test]
    pub fn parse_bean_succeeds() {
        assert_eq!(
            Ok((
                "() { ... }",
                Bean {
                    name: "myBean".to_string(),
                    class: "MyBean".to_string(),
                }
            )),
            parse_bean("@Bean\n    private MyBean myBean() { ... }")
        );
    }

    #[test]
    pub fn parse_bean_with_name_succeeds() {
        assert_eq!(
            Ok((
                "() { ... }",
                Bean {
                    name: "newName".to_string(),
                    class: "MyBean".to_string(),
                }
            )),
            parse_bean("@Bean(\"newName\")\n    private MyBean myBean() { ... }")
        );
    }
}
