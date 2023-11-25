use std::str::FromStr;

use super::{autowired::Autowired, bean::Bean, component_type::ComponentType};
use crate::{
    annotation::{parse_annotation, AnnotationArg},
    bean::{parse_bean, Parameter},
};
use derive_builder::Builder;
use nom::{
    bytes::complete::{is_not, tag, take_while},
    character::complete::{alphanumeric1, char, multispace0},
    combinator::opt,
    error::ErrorKind,
    multi::many0,
    sequence::delimited,
    IResult,
};

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
    parameters: Vec<Parameter>,
    #[builder(default)]
    autowires: Vec<Autowired>,
    #[builder(default)]
    bean_defs: Vec<Bean>,
    #[builder(default)]
    interfaces: Vec<String>,
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

    pub fn parameters(&self) -> &[Parameter] {
        self.parameters.as_ref()
    }

    pub fn autowires(&self) -> &[Autowired] {
        self.autowires.as_ref()
    }

    pub fn bean_defs(&self) -> &[Bean] {
        self.bean_defs.as_ref()
    }

    pub fn interfaces(&self) -> &[String] {
        self.interfaces.as_ref()
    }
}

pub fn parse_constructor(class_name: &str, body: &str) -> Option<Vec<Parameter>> {
    let pos = body.find(&format!("{}(", class_name))?;
    let body = &body[pos + class_name.len()..];

    let body = body.trim_start_matches('(');
    let (_, params) = take_while::<_, _, nom::error::Error<_>>(|c: char| c != ')')(body).ok()?;
    let params = params.trim_end_matches(')').trim();
    let params: Vec<Parameter> = if !params.is_empty() {
        params
            .trim_end_matches(')')
            .split(',')
            .filter_map(|p| Parameter::from_str(p).ok())
            .collect()
    } else {
        Vec::new()
    };

    Some(params)
}

pub fn parse_class(input: &str) -> IResult<&str, Class> {
    let mut class_builder = ClassBuilder::default();

    // Package declaration
    let pos = input
        .find("package")
        .ok_or_else(|| nom::Err::Failure(nom::error::make_error(input, ErrorKind::Fail)))?;
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
            match annotation.name() {
                "Import" => {
                    let imports = annotation.value();
                    match imports {
                        Some(AnnotationArg::Class(import)) => {
                            class_builder.imports(vec![import.clone()])
                        }
                        Some(AnnotationArg::Array(values)) => {
                            let mut classes = Vec::new();
                            for v in values {
                                if let AnnotationArg::Class(class) = v {
                                    classes.push(class.clone());
                                }
                            }
                            class_builder.imports(classes)
                        }
                        _ => &mut class_builder,
                    }
                }
                "SpringBootApplication" => class_builder.component_scans(vec![package.to_string()]),
                "ComponentScan" => {
                    let imports = annotation.value();
                    match imports {
                        Some(AnnotationArg::String(path)) => {
                            class_builder.component_scans(vec![path.clone()])
                        }
                        Some(AnnotationArg::Array(values)) if values.is_empty() => {
                            tracing::info!("Empty component scan");
                            class_builder.component_scans(vec![package.to_string()])
                        }
                        Some(AnnotationArg::Array(values)) => {
                            let mut paths = Vec::new();
                            for v in values {
                                if let AnnotationArg::String(path) = v {
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
            match annotation.name() {
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
    let class_start = input
        .find("class")
        .ok_or_else(|| nom::Err::Failure(nom::error::make_error(input, ErrorKind::Fail)))?;
    let input = &input[class_start + "class".len()..];
    let (input, _) = multispace0(input)?;
    let (input, name) = take_while(|c: char| c.is_alphanumeric())(input)?;
    class_builder.name(name.to_string());

    // Find interfaces this class extends
    let interfaces_start = input.find("implements");
    if let Some(pos) = interfaces_start {
        let input = &input[pos + "implements".len()..];
        let (input, _) = multispace0(input)?;
        let (_, interfaces) = take_while(|c: char| c.is_alphanumeric())(input)?;
        let interfaces = interfaces
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();
        class_builder.interfaces(interfaces);
    }

    // Find constructor
    let parameters = parse_constructor(name, input);
    if let Some(parameters) = parameters {
        class_builder.parameters(parameters);
    }

    // Autowire
    let mut autowire_start = input;
    let mut autowires = Vec::new();
    while let Some(pos) = autowire_start.find("@Autowired") {
        autowire_start = &autowire_start[pos..];
        let (input, (class, name)) = delimited(
            tag("@Autowired"),
            |input| {
                let (input, _) =
                    many0(delimited(multispace0, parse_annotation, multispace0))(input)?;
                let (input, _) = multispace0(input)?;
                let (input, class) = alphanumeric1(input)?;
                let (input, _) = multispace0(input)?;
                let (input, name) = alphanumeric1(input)?;
                Ok((input, (class, name)))
            },
            opt(char(';')),
        )(autowire_start)?;
        autowires.push(Autowired::new(class.to_string(), name.to_string()));
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

    let class = class_builder
        .build()
        .expect("should have been built correctly");
    tracing::trace!("Parsed class\n{:#?}", class);

    Ok(("", class))
}

#[cfg(test)]
mod tests {
    use std::vec;

    use crate::{
        autowired::Autowired,
        bean::{Bean, Parameter},
        class::{parse_class, Class},
        component_type::ComponentType,
    };

    use super::parse_constructor;

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
                    parameters: vec![Parameter {
                        annotations: vec!["@Arg".to_string()],
                        class: "Arg".to_string(),
                        name: "arg".to_string()
                    }],
                    autowires: vec![
                        Autowired::new("Foo".to_string(), "foo".to_string()),
                        Autowired::new("FooBean".to_string(), "fooBean".to_string())
                    ],
                    bean_defs: vec![Bean::new(
                        "MyBean".to_string(),
                        "myBean".to_string(),
                        vec![Parameter {
                            annotations: vec!["@Autowired".to_string(), "@NotNull".to_string()],
                            class: "FooBean".to_string(),
                            name: "fooBean".to_string()
                        }]
                    )],
                    interfaces: vec!["IFoo".to_string()]
                }
            )),
            parse_class(
                r#"
                package a.b.c;

                @Component
                @Import(Bar.class)
                @ComponentScan("a.b.c")
                public class Foo implements IFoo {
                    @Autowired Foo foo;
                    @Bean
                    public MyBean myBean( @Autowired @NotNull FooBean fooBean ) { ... }

                    Foo(@Arg Arg arg) {}
                }
                "#
            )
        );
    }

    #[test]
    fn parse_constructor_works() {
        let body = r#"
            // Docs with Foo?
            public or something Foo(@NotNull @Something Bar bar, Baz baz) {
                // stuff
            }
        "#;
        let class_name = "Foo";
        let params = parse_constructor(class_name, body).unwrap();
        assert_eq!(
            vec![
                Parameter {
                    annotations: vec!["@NotNull".to_string(), "@Something".to_string()],
                    class: "Bar".to_string(),
                    name: "bar".to_string(),
                },
                Parameter {
                    annotations: vec![],
                    class: "Baz".to_string(),
                    name: "baz".to_string(),
                }
            ],
            params
        )
    }
}
