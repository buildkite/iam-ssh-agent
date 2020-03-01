use std::os::unix::net::{UnixListener};
use ssh_agent::SSHAgentHandler;

use clap::{App, SubCommand};

fn main() {
    let matches = App::new("iam-ssh-agent")
		.version("1.0")
		.author("Keith Duncan <keith_duncan@me.com>")
		.about("Use an AWS IAM SSH Agent backend for SSH Authentication")
		.subcommand(SubCommand::with_name("list-keys")
		        .long("list-keys")
		        .short("l")
		        .help("List all keys for the caller IAM identity."))
		.arg(SubCommand::with_name("daemon")
		        .long("daemon")
		        .help("Run the daemon, bind a UNIX domain socket."))
		.subcommand_required(true)
		.get_matches();

	println!("{:?}", matches);

	unimplemented!()
}
