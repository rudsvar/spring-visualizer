use std::{
    error::Error,
    fs::File,
    io::{BufReader, Read},
};

use walkdir::WalkDir;

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = std::env::args();
    let executable = args.next().unwrap();
    let dir = args
        .next()
        .ok_or_else(|| format!("Usage: {} <path>", executable))?;
    println!("digraph Components {{");

    for entry in WalkDir::new(&dir) {
        let entry = entry?;
        if entry.file_type().is_file() {
            // Read file contents
            let path = entry.path();
            let f = File::open(&path)?;
            let mut f = BufReader::new(f);
            let mut buf = String::new();
            f.read_to_string(&mut buf)?;

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
    let start_paren = input.find("(")?;
    let end_paren = input.find(")")?;
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
    const CLASS: &'static str = "class";
    let start_of_class = input.find(CLASS)?;
    let input = &input[start_of_class + CLASS.len()..];
    let start_of_name = input.find(|c: char| c.is_alphabetic())?;
    let input = &input[start_of_name..];
    let class_name: String = input.chars().take_while(|c| c.is_alphabetic()).collect();
    Some(class_name)
}

#[derive(Debug, PartialEq, Eq)]
struct Import {
    name: String,
    children: Vec<String>,
}

impl Import {
    pub fn edges(&self) -> Vec<String> {
        let mut buf = Vec::new();
        for c in &self.children {
            buf.push(format!("{} -> {} [label=\"imports\"];", self.name, c));
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
        name: annotation.class_name,
        children,
    };
    Some(import)
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ComponentScan {
    class_name: String,
    paths: Vec<String>,
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
            paths,
        };
        Some(component_scan)
    }
    fn edges(&self, dir: &str) -> Vec<String> {
        let mut edges = Vec::new();
        for entry in WalkDir::new(dir) {
            if let Ok(entry) = entry {
                // Is file and is in scan path
                let is_file = entry.file_type().is_file();
                let is_scanned = self
                    .paths
                    .iter()
                    .any(|p| entry.path().to_str().expect("weird filename").contains(p));
                if !is_file || !is_scanned {
                    continue;
                }

                // Read file
                let path = entry.path();
                let f = File::open(&path).expect("failed to open file");
                let mut f = BufReader::new(f);
                let mut buf = String::new();
                f.read_to_string(&mut buf).expect("failed to read file");

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
                        "{} -> {} [label=\"scans\"];",
                        self.class_name, file_name
                    ));
                }
            }
        }
        edges
    }
}

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
            name: "ClassName".to_string(),
            children: vec!["Something".to_string()]
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
            name: "ClassName".to_string(),
            children: vec!["Something".to_string(), "Potato".to_string()]
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
            paths: vec!["foo.bar.baz".to_string()]
        }),
        c
    );
}
