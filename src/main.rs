use spring_visualizer::{Class, ComponentType};
use std::{
    error::Error,
    ffi::OsString,
    fs::File,
    io::{BufReader, Read},
    path::Path,
    str::FromStr,
};
use strum::IntoEnumIterator;
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
    env_logger::init();

    let mut args = std::env::args();
    let executable = args.next().unwrap();
    let dir = args
        .next()
        .ok_or_else(|| format!("Usage: {} <path>", executable))?;
    println!("digraph Components {{");

    println!("    # Legend");
    for component_type in ComponentType::iter() {
        println!(
            "    \"@{:?}\" [fillcolor=\"{}\",style=filled];",
            component_type,
            component_type.color_code()
        );
    }

    println!();

    println!("    # Align legend");
    for (cur, next) in ComponentType::iter().zip(ComponentType::iter().skip(1)) {
        println!(r#"    "@{:?}" -> "@{:?}" [style=invis];"#, cur, next);
    }
    println!();

    let classes: Vec<Class> = javafiles(&dir)
        .map(|entry| {
            let content = read_file(entry.path()).expect("failed to read file");
            let (_, class) =
                spring_visualizer::parse_class(&content).expect("failed to parse class");
            class
        })
        .collect();

    for class in &classes {
        // Node itself
        if let Some(component_type) = class.component_type() {
            println!(
                "    {} [fillcolor=\"{}\"style=filled];",
                class.name(),
                component_type.color_code()
            )
        }

        // Imports
        log::debug!("{}: Imports {:?}", class.name(), class.imports());
        for import in class.imports() {
            log::debug!("Import here");
            println!("    {} -> {} [label=imports];", class.name(), import);
        }

        // Component scans
        for package in class.component_scans() {
            println!("    {} [fillcolor=\"#97de50\",style=filled];", package);
            println!("    {} -> {} [label=scans];", class.name(), package);
            let scanned = classes
                .iter()
                .filter(|c| c.package().contains(package) && c.component_type().is_some());
            for c in scanned {
                println!("    {} -> {} [label=contains];", package, c.name());
            }
        }

        // Autowires
        for autowire in class.autowires() {
            println!(
                "    {} -> {} [label=autowires];",
                class.name(),
                autowire.class()
            );
        }

        // Beans
        for bean in class.bean_defs() {
            println!(
                "    {} -> {} [label=\"defines bean\"];",
                class.name(),
                bean.class()
            );
        }
    }

    println!("}}");

    Ok(())
}
