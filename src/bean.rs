use std::str::FromStr;

use nom::{
    bytes::complete::{tag, take_while},
    character::complete::{multispace0, space0},
    combinator::opt,
    IResult,
};

use super::annotation::{parse_annotation, AnnotationArg};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Parameter {
    pub class: String,
    pub name: String,
}

impl FromStr for Parameter {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        let (input, class) =
            take_while::<_, _, nom::error::Error<_>>(|c: char| c.is_alphanumeric())(s)
                .map_err(|e| format!("failed to parse parameter {}: {}", s, e))?;
        let (input, _) = multispace0::<_, nom::error::Error<_>>(input)
            .map_err(|e| format!("failed to parse parameter {}: {}", s, e))?;
        let (_, name) =
            take_while::<_, _, nom::error::Error<_>>(|c: char| c.is_alphanumeric())(input)
                .map_err(|e| format!("failed to parse parameter {}: {}", s, e))?;
        Ok(Parameter {
            class: class.to_string(),
            name: name.to_string(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bean {
    name: String,
    class: String,
    parameters: Vec<Parameter>,
}

impl Bean {
    pub fn new(class: String, name: String, parameters: Vec<Parameter>) -> Self {
        Bean {
            class,
            name,
            parameters,
        }
    }
    pub fn name(&self) -> &str {
        self.name.as_ref()
    }

    pub fn class(&self) -> &str {
        self.class.as_ref()
    }

    pub fn parameters(&self) -> &[Parameter] {
        self.parameters.as_ref()
    }
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
    // Get parameters
    let (input, _) = tag("(")(input)?;
    let (input, _) = multispace0(input)?;
    let (input, params) = take_while(|c: char| c != ')')(input)?;
    let params = params.trim_end_matches(')').trim();
    let params: Vec<Parameter> = if !params.is_empty() {
        params
            .trim_end_matches(')')
            .split(',')
            .filter_map(|def| {
                let def = def.trim();
                let (input, class) =
                    take_while::<_, _, nom::error::Error<_>>(|c: char| c.is_alphanumeric())(def)
                        .ok()?;
                let (input, _) = multispace0::<_, nom::error::Error<_>>(input).ok()?;
                let (_, name) =
                    take_while::<_, _, nom::error::Error<_>>(|c: char| c.is_alphanumeric())(input)
                        .ok()?;
                Some(Parameter {
                    name: name.to_string(),
                    class: class.to_string(),
                })
            })
            .collect()
    } else {
        vec![]
    };
    // See if name has been overridden
    let overriden_name = match annotation.value() {
        Some(AnnotationArg::String(name)) => Some(name.clone()),
        _ => None,
    };
    Ok((
        input,
        Bean {
            name: overriden_name.unwrap_or_else(|| name.to_string()),
            class: class.to_string(),
            parameters: params,
        },
    ))
}

#[cfg(test)]
mod tests {
    use crate::bean::{parse_bean, Bean, Parameter};

    #[test]
    pub fn parse_bean_succeeds() {
        assert_eq!(
            Ok((
                ") { ... }",
                Bean {
                    name: "myBean".to_string(),
                    class: "MyBean".to_string(),
                    parameters: vec![]
                }
            )),
            parse_bean("@Bean\n    private MyBean myBean( ) { ... }")
        );
    }

    #[test]
    pub fn parse_bean_succeeds2() {
        assert_eq!(
            Ok((
                ") { ... }",
                Bean {
                    name: "myBean".to_string(),
                    class: "MyBean".to_string(),
                    parameters: vec![Parameter {
                        class: "FooBean".to_string(),
                        name: "fooBean".to_string()
                    }]
                }
            )),
            parse_bean("@Bean\n    private MyBean myBean(FooBean fooBean) { ... }")
        );
    }

    #[test]
    pub fn parse_bean_with_name_succeeds() {
        assert_eq!(
            Ok((
                ") { ... }",
                Bean {
                    name: "newName".to_string(),
                    class: "MyBean".to_string(),
                    parameters: vec![Parameter {
                        class: "FooBean".to_string(),
                        name: "fooBean".to_string()
                    }]
                }
            )),
            parse_bean("@Bean(\"newName\")\n    private MyBean myBean(FooBean fooBean) { ... }")
        );
    }

    #[test]
    pub fn parameter_from_str() {
        assert_eq!(
            Ok(Parameter {
                class: "Foo".to_string(),
                name: "foo".to_string()
            }),
            " Foo foo ".parse()
        );
    }
}
