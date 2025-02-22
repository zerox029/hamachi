use clap::Subcommand;

#[derive(Debug, Subcommand)]
pub(crate) enum ConfigSubcommand {
    List,
    Get,
    Set { name: String, value: String },
    Unset,
    RenameSection,
    RemoveSection,
    Edit,
}

pub fn config(subcommand: ConfigSubcommand) {
    match subcommand {
        ConfigSubcommand::Set { name, value } => config_set(name, value),
        _ => todo!(),
    }
}

fn config_set(_name: String, _value: String) {
    todo!()
}
