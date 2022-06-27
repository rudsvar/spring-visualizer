use std::{
    error::Error,
    ffi::OsString,
    fs::File,
    io::{BufReader, Read},
    path::Path,
    str::FromStr,
};
use walkdir::{DirEntry, WalkDir};

fn read_file(path: &Path) -> Result<String, Box<dyn Error>> {
    // Read file contents
    let f = File::open(&path)?;
    let mut f = BufReader::new(f);
    let mut buf = String::new();
    f.read_to_string(&mut buf)?;
    Ok(buf)
}

fn javafiles(package: &str) -> impl Iterator<Item = DirEntry> + '_ {
    WalkDir::new(".").into_iter().filter_map(move |entry| {
        let entry = entry.unwrap();
        let path = entry.path();
        if !path.is_file() {
            return None;
        }

        let ext = entry.path().extension();
        let java_ext = OsString::from_str("java").unwrap();
        let is_java = ext == Some(&java_ext);

        let is_right_package = path.to_str().unwrap().contains(package);
        if is_java && is_right_package {
            Some(entry)
        } else {
            None
        }
    })
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = std::env::args();
    let executable = args.next().unwrap();
    let dir = args
        .next()
        .ok_or_else(|| format!("Usage: {} <path>", executable))?;
    println!("digraph Components {{");

    for entry in javafiles(&dir) {
        // Get filename
        let filename = entry
            .file_name()
            .to_str()
            .expect("invalid utf-8 filename")
            .trim_end_matches(".java");
        // Read file contents
        let buf = read_file(entry.path()).unwrap();

        // Parse imports
        let import = parse_import(&buf);
        if let Some(import) = import {
            for e in import.edges() {
                println!("    {}", e);
            }
        }

        // Parse component scans
        let scans = ComponentScan::parse(&buf);
        if let Some(scan) = scans {
            for e in scan.edges(&dir) {
                println!("    {}", e);
            }
        }

        let beans = Bean::parse(&buf);
        for b in beans {
            println!("    {} -> {} [label=defines]", filename, b.class_name);
        }

        let dependencies = Dependency::parse(&buf);
        for d in dependencies {
            println!("    {} -> {} [label=autowires]", filename, d.class_name);
        }
    }

    println!("}}");

    Ok(())
}

struct Annotation {
    class_name: String,
    values: Vec<String>,
}

fn find_annotation(annotation_name: &str, input: &str) -> Option<Annotation> {
    // Find import
    let input = &input[input.find(&format!("@{}", &annotation_name))?..];
    let start_paren = input.find('(')?;
    let end_paren = input.find(')')?;
    let between = &input[start_paren + 1..end_paren];
    let values: Vec<String> = between
        .trim_start_matches('{')
        .trim_end_matches('}')
        .split(',')
        .map(|v| v.trim().to_string())
        .collect();
    let input = &input[end_paren + ')'.len_utf8()..];

    // Find class name
    let class_name = find_class_name(input)?;
    let component = Annotation { class_name, values };
    Some(component)
}

fn find_class_name(input: &str) -> Option<String> {
    const CLASS: &str = "class";
    let start_of_class = input.find(CLASS)?;
    let input = &input[start_of_class + CLASS.len()..];
    let start_of_name = input.find(|c: char| c.is_alphabetic())?;
    let input = &input[start_of_name..];
    let class_name: String = input.chars().take_while(|c| c.is_alphabetic()).collect();
    Some(class_name)
}

#[derive(Debug, PartialEq, Eq)]
struct Import {
    class_name: String,
    classes: Vec<String>,
}

impl Import {
    pub fn edges(&self) -> Vec<String> {
        let mut buf = Vec::new();
        for c in &self.classes {
            buf.push(format!(
                r#""{}" -> "{}" [label="imports"];"#,
                self.class_name, c
            ));
        }
        buf
    }
}

fn parse_import(input: &str) -> Option<Import> {
    let annotation = find_annotation("Import", input)?;
    let children = annotation
        .values
        .into_iter()
        .map(|v| v.trim().trim_end_matches(".class").to_string())
        .collect();
    let import = Import {
        class_name: annotation.class_name,
        classes: children,
    };
    Some(import)
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ComponentScan {
    class_name: String,
    packages: Vec<String>,
}

impl ComponentScan {
    fn parse(input: &str) -> Option<ComponentScan> {
        let annotation = find_annotation("ComponentScan", input)?;
        let paths = annotation
            .values
            .into_iter()
            .map(|v| v.trim_matches('"').to_string())
            .collect();
        let component_scan = ComponentScan {
            class_name: annotation.class_name,
            packages: paths,
        };
        Some(component_scan)
    }
    fn edges(&self, dir: &str) -> Vec<String> {
        let mut edges = Vec::new();
        // Paths that this scans
        for path in &self.packages {
            edges.push(format!(
                r#""{}" -> "{}" [label="scans"];"#,
                self.class_name, path
            ));
        }
        // Files scanned
        for entry in javafiles(dir) {
            // Find out which package this file is in
            let scanned_by = self
                .packages
                .iter()
                .find(|&p| entry.path().to_str().expect("weird filename").contains(p));
            if scanned_by.is_none() {
                continue;
            }
            let scanned_by = scanned_by.expect("just checked");

            // Read file
            let path = entry.path();
            let buf = read_file(path).unwrap();

            // Check if marked as component
            let component_types = ["@Component", "@Repository", "@Service", "@Configuration"];
            let is_component = component_types.iter().any(|ct| buf.contains(ct));
            let file_name = entry
                .file_name()
                .to_str()
                .unwrap()
                .trim_end_matches(".java");

            if is_component {
                edges.push(format!(
                    r#""{}" -> "{}" [label="contains"];"#,
                    scanned_by, file_name
                ));
            }
        }
        edges
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bean {
    class_name: String,
}

impl Bean {
    pub fn new(class_name: String) -> Self {
        Self { class_name }
    }

    pub fn parse(mut input: &str) -> Vec<Bean> {
        let mut beans = Vec::new();
        while let Some(pos) = input.find("@Bean") {
            // Skip @Bean
            input = &input[pos + "@Bean".len()..];
            // Stop at (
            let start_paren = input.find('(').expect("start paren");
            let between = &input[..start_paren];
            // Skip visibility modifier
            let between = between
                .trim_start()
                .trim_start_matches("public")
                .trim_start_matches("private")
                .trim_start();
            let class_name = between.chars().take_while(|c| c.is_alphabetic()).collect();
            beans.push(Bean { class_name })
        }
        beans
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dependency {
    class_name: String,
}

impl Dependency {
    pub fn new(class_name: String) -> Self {
        Self { class_name }
    }

    pub fn parse(mut input: &str) -> Vec<Dependency> {
        let mut dependencies = Vec::new();
        while let Some(pos) = input.find("@Autowire") {
            // Skip @Autowire
            input = input[pos + "@Autowire".len()..].trim_start();
            let class_name = input.chars().take_while(|c| c.is_alphabetic()).collect();
            dependencies.push(Dependency { class_name })
        }
        dependencies
    }
}

#[cfg(test)]
mod tests {
    use crate::{parse_import, Bean, ComponentScan, Dependency, Import};

    #[test]
    fn single_import() {
        let my_str = r#"
        import foo.bar;

        @Import ( Something.class )
        class ClassName { ... }
        "#;
        let c = parse_import(my_str);
        assert_eq!(
            Some(Import {
                class_name: "ClassName".to_string(),
                classes: vec!["Something".to_string()]
            }),
            c
        );
    }

    #[test]
    fn multi_import() {
        let my_str = r#"
        import foo.bar;

        @Import ({ Something.class, Potato.class })
        class ClassName { ... }
        "#;
        let c = parse_import(my_str);
        assert_eq!(
            Some(Import {
                class_name: "ClassName".to_string(),
                classes: vec!["Something".to_string(), "Potato".to_string()]
            }),
            c
        );
    }

    #[test]
    fn component_scan() {
        let my_str = r#"
        import foo.bar;

        @ComponentScan(
            "foo.bar.baz"
        )
        class ClassName { ... }
        "#;
        let c = ComponentScan::parse(my_str);
        assert_eq!(
            Some(ComponentScan {
                class_name: "ClassName".to_string(),
                packages: vec!["foo.bar.baz".to_string()]
            }),
            c
        );
    }

    #[test]
    fn bean_defs() {
        let my_str = r#"
        @Bean
        public A a() { ... }

        @Bean
        private B b() { ... }

        @Bean
        C c() { ... }
        "#;
        let c = Bean::parse(my_str);
        assert_eq!(
            vec![
                Bean::new("A".to_string()),
                Bean::new("B".to_string()),
                Bean::new("C".to_string()),
            ],
            c
        );
    }

    #[test]
    fn dependencies() {
        let my_str = r#"
        class MyClass {
            @Autowire A a;
            @Autowire B b;
        }
        "#;
        let c = Dependency::parse(my_str);
        assert_eq!(
            vec![
                Dependency::new("A".to_string()),
                Dependency::new("B".to_string()),
            ],
            c
        );
    }
}
