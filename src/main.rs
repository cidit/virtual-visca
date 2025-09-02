use clap;
use std::net::SocketAddr;
use bevy::prelude as bv;


#[derive(clap::Parser)]
#[command(version, about, long_about = None)]
struct Cli {

    #[arg(short, long, value_name = "ADDRESS:PORT")]
    visca_bind: SocketAddr,


    #[arg(short, long, value_name = "ADDRESS:PORT")]
    video_bind: SocketAddr,

    /// Turn debugging information on
    #[arg(short, long, action = clap::ArgAction::Count)]
    debug: u8,
}


fn main() {
    let cli = Cli::parse();
    if cli.debug > 0 {
        println!("Hello, virtual-visca!");
    }

    bv::App::new().run();
}
