use std::os::unix::net::{UnixListener};
use ssh_agent::SSHAgentHandler;

use clap::{App, SubCommand};

fn main() {
    let matches = App::new("iam-ssh-agent")
		.version("1.0")
		.author("Keith Duncan <keith_duncan@me.com>")
		.about("Use an AWS IAM SSH Agent backend for SSH Authentication")
		.subcommand(SubCommand::with_name("list-keys")
		    .help("List all keys for the caller IAM identity."))
		.subcommand(SubCommand::with_name("daemon")
		    .help("Run the daemon, bind a UNIX domain socket."))
		.get_matches();

	println!("{:?}", matches);

	if let Some(matches) = matches.subcommand_matches("list-keys") {

	}

	if let Some(matches) = matches.subcommand_matches("daemon") {
		
	}

	unimplemented!()
}
