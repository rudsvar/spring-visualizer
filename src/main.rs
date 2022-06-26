use std::{
    error::Error,
    fs::File,
    io::{BufReader, Read},
};

use walkdir::WalkDir;

fn main() -> Result<(), Box<dyn Error>> {
    let dir = std::env::args().nth(1).ok_or("give a directory")?;
    println!("digraph Components {{");

    for entry in WalkDir::new(dir) {
        let entry = entry?;
        if entry.file_type().is_file() {
            let path = entry.path();
            let f = File::open(&path)?;
            let mut f = BufReader::new(f);
            let mut buf = String::new();
            f.read_to_string(&mut buf)?;
            let import = parse_import(&buf);
            if let Some(import) = import {
                for e in import.edges() {
                    println!("    {}", e);
                }
            }
        }
    }

    println!("}}");

    Ok(())
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
    // Find import
    let input = &input[input.find("@Import")?..];
    let start_paren = input.find("(")?;
    let end_paren = input.find(")")?;
    let between = &input[start_paren + 1..end_paren];
    let classes: Vec<String> = between
        .trim_start_matches('{')
        .trim_end_matches('}')
        .split(',')
        .map(|s| s.trim().trim_end_matches(".class").to_string())
        .collect();
    let input = &input[end_paren + ')'.len_utf8()..];

    // Find class name
    const CLASS: &'static str = "class";
    let start_of_class = input.find(CLASS)?;
    let input = &input[start_of_class + CLASS.len()..];
    let start_of_name = input.find(|c: char| c.is_alphabetic())?;
    let input = &input[start_of_name..];
    let name: String = input.chars().take_while(|c| c.is_alphabetic()).collect();
    let component = Import {
        name,
        children: classes,
    };
    Some(component)
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
