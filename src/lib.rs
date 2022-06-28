use derive_builder::Builder;
use nom::{
    bytes::complete::is_not,
    bytes::complete::{tag, take_until, take_while},
    character::complete::{alphanumeric1, char, multispace0, space0},
    combinator::{map, opt},
    multi::many0,
    sequence::delimited,
    IResult,
};
use std::collections::HashMap;
use strum::EnumIter;

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

pub fn parse_value(input: &str) -> IResult<&str, Value> {
    let is_string = input.starts_with('"');
    let is_array = input.starts_with('{');
    let res = if is_string {
        let (input, between) = delimited(char('"'), is_not("\""), char('"'))(input)?;
        (input, Value::String(between.to_string()))
    } else if is_array {
        let (input, between) = delimited(char('{'), is_not("}"), char('}'))(input)?;
        let between = between.trim_start();
        let (_, values) = nom::multi::separated_list0(
            |input| {
                let (input, _) = tag(",")(input)?;
                let (input, _) = space0(input)?;
                Ok((input, ()))
            },
            parse_value,
        )(between)?;
        (input, Value::Array(values))
    } else {
        let (input, name) = take_until(".class")(input)?;
        let (input, _) = tag(".class")(input)?;
        (input, Value::Class(name.to_string()))
    };
    Ok(res)
}

pub fn parse_key_value_pairs(input: &str) -> IResult<&str, HashMap<String, Value>> {
    log::debug!("Parsing multi value from {}", input);
    let (input, key_value_pairs) = many0(|input| {
        let (input, key) = take_while(|c: char| c.is_alphanumeric())(input)?;
        let (input, _) = space0(input)?;
        let (input, _) = tag("=")(input)?;
        let (input, _) = space0(input)?;
        let (input, value) = parse_value(input)?;
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
        map(parse_key_value_pairs, Args::Multi)(input)
    } else {
        map(parse_value, Args::Single)(input)
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bean {
    name: String,
    class: String,
}

impl Bean {
    pub fn name(&self) -> &str {
        self.name.as_ref()
    }

    pub fn class(&self) -> &str {
        self.class.as_ref()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Autowired {
    name: String,
    class: String,
}

impl Autowired {
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

#[derive(Debug, Clone, PartialEq, Eq, EnumIter)]
pub enum ComponentType {
    SpringBootApplication,
    Configuration,
    Controller,
    Service,
    Repository,
    Component,
}

impl ComponentType {
    pub fn color_code(&self) -> &'static str {
        match self {
            ComponentType::SpringBootApplication => "#2c9162",
            ComponentType::Configuration => "#28a9e0",
            ComponentType::Controller => "#7050bf",
            ComponentType::Service => "#a81347",
            ComponentType::Repository => "#e06907",
            ComponentType::Component => "#ffc400",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Builder)]
pub struct Class {
    package: String,
    #[builder(default)]
    component_type: Option<ComponentType>,
    #[builder(default)]
    imports: Vec<String>,
    #[builder(default)]
    component_scans: Vec<String>,
    name: String,
    #[builder(default)]
    autowires: Vec<Autowired>,
    #[builder(default)]
    bean_defs: Vec<Bean>,
}

impl Class {
    pub fn package(&self) -> &str {
        self.package.as_ref()
    }

    pub fn component_type(&self) -> Option<&ComponentType> {
        self.component_type.as_ref()
    }

    pub fn imports(&self) -> &[String] {
        self.imports.as_ref()
    }

    pub fn component_scans(&self) -> &[String] {
        self.component_scans.as_ref()
    }

    pub fn name(&self) -> &str {
        self.name.as_ref()
    }

    pub fn autowires(&self) -> &[Autowired] {
        self.autowires.as_ref()
    }

    pub fn bean_defs(&self) -> &[Bean] {
        self.bean_defs.as_ref()
    }
}

pub fn parse_class(input: &str) -> IResult<&str, Class> {
    let mut class_builder = ClassBuilder::default();

    // Package declaration
    let pos = input.find("package").expect("no package declaration");
    let input = &input[pos..];
    let (mut input, between) = delimited(tag("package"), is_not(";"), char(';'))(input)?;
    let package = between.trim();
    class_builder.package(package.to_string());

    // Class level annotations
    if let Some(pos) = input.find('@') {
        let tmp_input = &input[pos..];
        let (new_input, annotations) = many0(parse_annotation)(tmp_input)?;
        input = new_input;
        for annotation in annotations {
            match annotation.name.as_str() {
                "Import" => {
                    let imports = annotation.value();
                    match imports {
                        Some(Value::Class(import)) => class_builder.imports(vec![import.clone()]),
                        Some(Value::Array(values)) => {
                            let mut classes = Vec::new();
                            for v in values {
                                if let Value::Class(class) = v {
                                    classes.push(class.clone());
                                }
                            }
                            class_builder.imports(classes)
                        }
                        _ => &mut class_builder,
                    }
                }
                "ComponentScan" => {
                    let imports = annotation.value();
                    match imports {
                        Some(Value::String(path)) => {
                            class_builder.component_scans(vec![path.clone()])
                        }
                        Some(Value::Array(values)) => {
                            let mut paths = Vec::new();
                            for v in values {
                                if let Value::String(path) = v {
                                    paths.push(path.clone());
                                }
                            }
                            class_builder.component_scans(paths)
                        }
                        _ => &mut class_builder,
                    }
                }
                _ => &mut class_builder,
            };
            // Set component type
            match annotation.name.as_str() {
                "SpringBootApplication" => {
                    class_builder.component_type(Some(ComponentType::SpringBootApplication))
                }
                "Configuration" => class_builder.component_type(Some(ComponentType::Configuration)),
                "Controller" => class_builder.component_type(Some(ComponentType::Controller)),
                "RestController" => class_builder.component_type(Some(ComponentType::Controller)),
                "Service" => class_builder.component_type(Some(ComponentType::Service)),
                "Repository" => class_builder.component_type(Some(ComponentType::Repository)),
                "Component" => class_builder.component_type(Some(ComponentType::Component)),
                _ => &mut class_builder,
            };
        }
    }

    // Class name
    let class_start = input.find("class").expect("no class name");
    let input = &input[class_start + "class".len()..];
    let (input, _) = multispace0(input)?;
    let (input, name) = take_while(|c: char| c.is_alphanumeric())(input)?;
    class_builder.name(name.to_string());

    // Autowire
    let mut autowire_start = input;
    let mut autowires = Vec::new();
    while let Some(pos) = autowire_start.find("@Autowired") {
        autowire_start = &autowire_start[pos..];
        let (input, (class, name)) = delimited(
            tag("@Autowired"),
            |input| {
                let (input, _) = multispace0(input)?;
                let (input, class) = alphanumeric1(input)?;
                let (input, _) = multispace0(input)?;
                let (input, name) = alphanumeric1(input)?;
                Ok((input, (class, name)))
            },
            char(';'),
        )(autowire_start)?;
        autowires.push(Autowired {
            class: class.to_string(),
            name: name.to_string(),
        });
        autowire_start = input;
    }
    class_builder.autowires(autowires);

    // Beans
    let mut beans_start = input;
    let mut beans = Vec::new();
    while let Some(pos) = beans_start.find("@Bean") {
        beans_start = &beans_start[pos..];
        let (input, bean) = parse_bean(beans_start)?;
        beans_start = input;
        beans.push(bean);
    }
    class_builder.bean_defs(beans);

    let class = class_builder.build().unwrap();
    log::debug!("Parsed class\n{:#?}", class);

    Ok(("", class))
}

#[cfg(test)]
mod tests {
    use crate::{
        parse_annotation, parse_bean, parse_class, Annotation, Args, Autowired, Bean, Class,
        ComponentType, Value,
    };

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

    #[test]
    pub fn parse_class_test() {
        assert_eq!(
            Ok((
                "",
                Class {
                    package: "a.b.c".to_string(),
                    component_type: Some(ComponentType::Component),
                    imports: vec!["Bar".to_string()],
                    component_scans: vec!["a.b.c".to_string()],
                    name: "Foo".to_string(),
                    autowires: vec![Autowired {
                        class: "Foo".to_string(),
                        name: "foo".to_string()
                    }],
                    bean_defs: vec![Bean {
                        name: "myBean".to_string(),
                        class: "MyBean".to_string()
                    }],
                }
            )),
            parse_class(
                r#"
                package a.b.c;

                @Component
                @Import(Bar.class)
                @ComponentScan("a.b.c")
                public class Foo {
                    @Autowire Foo foo;
                    @Bean
                    public MyBean myBean() { ... }
                }
                "#
            )
        );
    }
}
