use nom::{
    bytes::complete::{is_not, tag, take_until, take_while},
    character::complete::{char, multispace0, space0},
    combinator::{map, opt},
    multi::many0,
    sequence::delimited,
    IResult,
};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AnnotationArg {
    String(String),
    Class(String),
    Array(Vec<AnnotationArg>),
}

pub fn parse_arg(input: &str) -> IResult<&str, AnnotationArg> {
    let is_string = input.starts_with('"');
    let is_array = input.starts_with('{');
    let res = if is_string {
        let (input, between) = delimited(char('"'), is_not("\""), char('"'))(input)?;
        (input, AnnotationArg::String(between.to_string()))
    } else if is_array {
        let (input, between) = delimited(char('{'), is_not("}"), char('}'))(input)?;
        let between = between.trim_start();
        let (_, values) = nom::multi::separated_list0(
            |input| {
                let (input, _) = tag(",")(input)?;
                let (input, _) = space0(input)?;
                Ok((input, ()))
            },
            parse_arg,
        )(between)?;
        (input, AnnotationArg::Array(values))
    } else {
        let (input, name) = take_until(".class")(input)?;
        let (input, _) = tag(".class")(input)?;
        (input, AnnotationArg::Class(name.to_string()))
    };
    Ok(res)
}

pub fn parse_key_value_pairs(input: &str) -> IResult<&str, HashMap<String, AnnotationArg>> {
    log::debug!("Parsing multi value from {}", input);
    let (input, key_value_pairs) = many0(|input| {
        let (input, key) = take_while(|c: char| c.is_alphanumeric())(input)?;
        let (input, _) = space0(input)?;
        let (input, _) = tag("=")(input)?;
        let (input, _) = space0(input)?;
        let (input, value) = parse_arg(input)?;
        let (input, _) = opt(tag(","))(input)?;
        let (input, _) = space0(input)?;
        Ok((input, (key.to_string(), value)))
    })(input)?;
    let values = key_value_pairs.into_iter().collect();
    Ok((input, values))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AnnotationArgs {
    Single(AnnotationArg),
    Multi(HashMap<String, AnnotationArg>),
}

pub fn parse_args(input: &str) -> IResult<&str, AnnotationArgs> {
    if input.contains('=') {
        map(parse_key_value_pairs, AnnotationArgs::Multi)(input)
    } else {
        map(parse_arg, AnnotationArgs::Single)(input)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Annotation {
    name: String,
    args: AnnotationArgs,
}

impl Annotation {
    pub fn value(&self) -> Option<&AnnotationArg> {
        match &self.args {
            AnnotationArgs::Single(v) => Some(v),
            AnnotationArgs::Multi(vs) => vs.get("value"),
        }
    }

    pub fn name(&self) -> &str {
        self.name.as_ref()
    }

    pub fn args(&self) -> &AnnotationArgs {
        &self.args
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
        (input, AnnotationArgs::Multi(HashMap::new()))
    };

    Ok((
        input,
        Annotation {
            name: name.to_string(),
            args,
        },
    ))
}

pub fn parse_annotations(mut input: &str) -> IResult<&str, Vec<Annotation>> {
    let mut annotations = Vec::new();
    while let Some(pos) = input.find('@') {
        input = &input[pos..];
        let (new_input, annotation) = parse_annotation(input)?;
        annotations.push(annotation);
        input = new_input;
    }
    Ok((input, annotations))
}

#[cfg(test)]
mod tests {
    use crate::annotation::{parse_annotation, Annotation, AnnotationArg, AnnotationArgs};

    #[test]
    pub fn parse_annotation_with_class_value_succeeds() {
        assert_eq!(
            Ok((
                "",
                Annotation {
                    name: "MyAnnotation".to_string(),
                    args: AnnotationArgs::Single(AnnotationArg::Class("Test".to_string()))
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
                    args: AnnotationArgs::Single(AnnotationArg::String("com.example".to_string()))
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
                    args: AnnotationArgs::Single(AnnotationArg::Array(vec![
                        AnnotationArg::String("com.example".to_string()),
                        AnnotationArg::Class("Foo".to_string())
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
                    args: AnnotationArgs::Multi(
                        vec![
                            ("foo".to_string(), AnnotationArg::Class("Foo".to_string())),
                            (
                                "bar".to_string(),
                                AnnotationArg::String("com.example".to_string())
                            ),
                            (
                                "baz".to_string(),
                                AnnotationArg::Array(vec![
                                    AnnotationArg::Class("A".to_string()),
                                    AnnotationArg::Class("B".to_string())
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
}
