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
}

impl Bean {
    pub fn new(class: String, name: String) -> Self {
        Bean { class, name }
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
        },
    ))
}

#[cfg(test)]
mod tests {
    use crate::bean::{parse_bean, Bean};

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
