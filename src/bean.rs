use nom::{
    bytes::complete::{tag, take_while},
    character::complete::{multispace0, space0},
    combinator::opt,
    IResult,
};

use super::annotation::{parse_annotation, AnnotationArg};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bean {
    name: String,
    class: String,
    parameters: Vec<Parameter>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Parameter {
    pub name: String,
    pub class: String,
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
    let params: Vec<Parameter> = params
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
        .collect();
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
                    parameters: vec![Parameter {
                        name: "fooBean".to_string(),
                        class: "FooBean".to_string()
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
                        name: "fooBean".to_string(),
                        class: "FooBean".to_string()
                    }]
                }
            )),
            parse_bean("@Bean(\"newName\")\n    private MyBean myBean(FooBean fooBean) { ... }")
        );
    }
}
