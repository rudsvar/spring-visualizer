use super::{autowired::Autowired, bean::Bean, component_type::ComponentType};
use crate::{
    annotation::{parse_annotation, AnnotationArg},
    bean::parse_bean,
};
use derive_builder::Builder;
use nom::{
    bytes::complete::{is_not, tag, take_while},
    character::complete::{alphanumeric1, char, multispace0},
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
                "ComponentScan" => {
                    let imports = annotation.value();
                    match imports {
                        Some(AnnotationArg::String(path)) => {
                            class_builder.component_scans(vec![path.clone()])
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

    let class = class_builder.build().unwrap();
    log::debug!("Parsed class\n{:#?}", class);

    Ok(("", class))
}

#[cfg(test)]
mod tests {
    use crate::{
        autowired::Autowired,
        bean::Bean,
        class::{parse_class, Class},
        component_type::ComponentType,
    };

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
                    autowires: vec![Autowired::new("Foo".to_string(), "foo".to_string())],
                    bean_defs: vec![Bean::new("MyBean".to_string(), "myBean".to_string())],
                }
            )),
            parse_class(
                r#"
                package a.b.c;

                @Component
                @Import(Bar.class)
                @ComponentScan("a.b.c")
                public class Foo {
                    @Autowired Foo foo;
                    @Bean
                    public MyBean myBean() { ... }
                }
                "#
            )
        );
    }
}
