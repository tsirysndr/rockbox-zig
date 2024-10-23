use prost_build::{Config, Service, ServiceGenerator};
use serde::{Deserialize, Serialize};
use std::io::Result;
use std::{fs, path::PathBuf, process::Command};

#[derive(Deserialize, Serialize, Debug, Default)]
#[serde(default)]
pub struct BuildConfig {
    /// base path for protobuf files
    pub base_path: Option<PathBuf>,
    /// protobuf include dirs
    pub includes: Vec<String>,
    /// protobuf files
    pub files: Vec<String>,
    /// dir for generated code, defaults to Cargo OUT_DIR, else the current dir
    pub output: Option<String>,
    /// build options for messages
    pub messages: Vec<BuildOption>,
    /// build options for enums
    pub enums: Vec<BuildOption>,
    /// build options for fields
    pub fields: Vec<BuildOption>,
    /// build options for bytes
    pub bytes: Vec<String>,
    /// build options for BTreeMap
    pub btree_maps: Vec<String>,
}

#[derive(Deserialize, Serialize, Debug, Default)]
#[serde(default)]
pub struct BuildOption {
    /// a list of paths you want to add the attribute
    pub paths: Vec<String>,
    /// description of the option
    pub description: String,
    /// extra attributes to put on generated data structure, for example: `derive(Serialize, Deserialize)`
    /// it will be converted to `#[derive(Serialize, Deserialize)]`
    pub attrs: Vec<String>,
}

impl From<BuildConfig> for Builder {
    fn from(config: BuildConfig) -> Self {
        // For the output directory, use the specified one, or fallback to the
        // OUT_DIR env variable provided by Cargo if it exists (it should!), else
        // fallback to the current directory.
        let output_dir: String = match &config.output {
            None => {
                let default_output_dir = std::env::var("OUT_DIR");

                match default_output_dir {
                    Err(_) => String::new(),
                    Ok(cargo_out_dir) => cargo_out_dir,
                }
            }
            Some(specified_output) => specified_output.to_owned(),
        };

        let mut c = Config::new();

        c.btree_map(config.btree_maps);
        c.bytes(config.bytes);

        for opt in config.messages {
            for p in opt.paths {
                c.type_attribute(p, to_attr(&opt.attrs));
            }
        }

        for opt in config.enums {
            for p in opt.paths {
                c.type_attribute(p, to_attr(&opt.attrs));
            }
        }

        for opt in config.fields {
            for p in opt.paths {
                c.field_attribute(p, to_attr(&opt.attrs));
            }
        }

        fs::create_dir_all(&output_dir).unwrap();
        c.out_dir(&output_dir);

        let f = |v: String| match config.base_path {
            Some(ref base_path) => base_path.join(v).to_string_lossy().to_string(),
            None => v,
        };
        Self {
            config: c,
            includes: config.includes.into_iter().map(f).collect(),
            files: config.files.into_iter().map(f).collect(),
        }
    }
}

pub struct Builder {
    /// prost configuration
    config: Config,
    /// protobuf include dirs
    pub includes: Vec<String>,
    /// protobuf files
    pub files: Vec<String>,
}

impl Builder {
    /// Configures the code generator for service generator
    pub fn service_generator(&mut self, service_generator: Box<dyn ServiceGenerator>) -> &mut Self {
        self.config.service_generator(service_generator);
        self
    }

    /// build protobuf with configuration
    pub fn build_protos(&mut self) {
        self.config
            .compile_protos(&self.files, &self.includes)
            .unwrap_or_else(|e| panic!("Failed to compile proto files. Err: {:?}", e));

        Command::new("cargo")
            .args(["fmt"])
            .status()
            .expect("cargo fmt failed");
    }
}

fn to_attr(attrs: &[String]) -> String {
    attrs
        .iter()
        .map(|s| format!("#[{}]", s))
        .collect::<Vec<_>>()
        .join("\n")
}

struct MyServiceGen;

impl ServiceGenerator for MyServiceGen {
    fn generate(&mut self, service: Service, _buf: &mut String) {
        println!("{service:#?}");
    }
}

fn main() -> Result<()> {
    let content = include_str!("./build_config.yml");
    let config: BuildConfig = serde_yaml::from_str(content).unwrap();
    Builder::from(config)
        .service_generator(Box::new(MyServiceGen))
        .build_protos();
    Ok(())
}
