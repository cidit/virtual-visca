
#[derive(clap::Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[arg(long, value_name = "ADDRESS:PORT", default_value = "127.0.0.1:1259")]
    pub visca_bind: String,

    #[arg(long, value_name = "ADDRESS:PORT")]
    pub video_bind: Option<String>,

    #[arg(short, long, action = clap::ArgAction::Count)]
    pub debug: u8,
}
