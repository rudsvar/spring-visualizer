use clap::Parser;
use ignore::{DirEntry, Walk};
use itertools::Itertools;
use spring_visualizer::{
    class::{parse_class, Class},
    component_type::ComponentType,
};
use std::{
    error::Error,
    ffi::OsString,
    fmt::Display,
    fs::File,
    io::{BufReader, Read},
    path::Path,
    str::FromStr,
};
use strum::{EnumIter, IntoEnumIterator};

fn read_file(path: &Path) -> Result<String, Box<dyn Error>> {
    // Read file contents
    let f = File::open(&path)?;
    let mut f = BufReader::new(f);
    let mut buf = String::new();
    f.read_to_string(&mut buf)?;
    Ok(buf)
}

fn javafiles(package: &str) -> impl Iterator<Item = DirEntry> + '_ {
    Walk::new("./")
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(move |entry| {
            let path = entry.path();
            if !path.is_file() {
                return false;
            }

            let ext = entry.path().extension();
            let java_ext = OsString::from_str("java").unwrap();
            let is_java = ext == Some(&java_ext);

            let is_right_package = path.to_str().unwrap().contains(package);

            is_java && is_right_package
        })
}

fn print_legend() {
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
}

#[derive(Debug, Clone, PartialEq, Eq, Parser, EnumIter)]
pub enum Feature {
    Import,
    ComponentScan,
    Autowired,
    Bean,
}

impl Display for Feature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let debug = format!("{:?}", self);
        write!(f, "{}", debug.to_lowercase())
    }
}

impl FromStr for Feature {
    type Err = String;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        for feature in Feature::iter() {
            if input == feature.to_string() {
                return Ok(feature);
            }
        }
        return Err(format!("unknown feature {}", input));
    }
}

#[derive(Debug, Clone, Parser)]
pub struct Features {
    features: Vec<Feature>,
}

impl Features {
    pub fn contains(&self, feature: &Feature) -> bool {
        self.features.contains(feature)
    }
}

impl Default for Features {
    fn default() -> Self {
        Self {
            features: Feature::iter().collect(),
        }
    }
}

impl Display for Features {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let comma_separated_features = self.features.iter().map(|f| f.to_string()).join(",");
        write!(f, "{}", comma_separated_features)
    }
}

impl FromStr for Features {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let features = s
            .split(',')
            .map(|s| s.trim())
            .map(FromStr::from_str)
            .collect::<Result<Vec<Feature>, _>>()?;
        Ok(Self { features })
    }
}

#[derive(Debug, Clone, Parser)]
pub struct Args {
    /// Directories to scan for files.
    path: String,
    /// Kinds of relations to include.
    #[clap(short, long, default_value_t)]
    features: Features,
}

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let args = Args::parse();

    println!("digraph Components {{");

    print_legend();

    let classes: Vec<Class> = javafiles(&args.path)
        .map(|entry| {
            let content = read_file(entry.path()).expect("failed to read file");
            let (_, class) = parse_class(&content).expect("failed to parse class");
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
        if args.features.contains(&Feature::Import) {
            log::debug!("{}: Imports {:?}", class.name(), class.imports());
            for import in class.imports() {
                log::debug!("Import here");
                println!("    {} -> {} [label=\"@Import\"];", class.name(), import);
            }
        }

        // Component scans
        if args.features.contains(&Feature::ComponentScan) {
            for package in class.component_scans() {
                println!("    \"{}\" [style=filled];", package);
                println!(
                    "    \"{}\" -> \"{}\" [label=\"@ComponentScan\"];",
                    class.name(),
                    package
                );
                let scanned = classes
                    .iter()
                    .filter(|c| c.package().contains(package) && c.component_type().is_some());
                for c in scanned {
                    println!("    \"{}\" -> {} [label=contains];", package, c.name());
                }
            }
        }

        // Autowires
        if args.features.contains(&Feature::Autowired) {
            for autowire in class.autowires() {
                println!(
                    "    {} -> {} [label=\"@Autowired\"];",
                    class.name(),
                    autowire.class()
                );
            }
        }

        // Beans
        if args.features.contains(&Feature::Bean) {
            for bean in class.bean_defs() {
                println!("    {} [fillcolor=\"#6b1d1d\",style=filled];", bean.class());
                println!(
                    "    {} -> {} [label=\"@Bean\"];",
                    class.name(),
                    bean.class()
                );
            }
        }
    }

    println!("}}");

    Ok(())
}
