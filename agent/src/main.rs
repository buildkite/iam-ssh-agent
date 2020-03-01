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

	eprintln!("{:?}", matches);

	// Uses an environment variable rather than an argument so that this can be
	// an ECS ValueFrom in an ECS task.
	let ssh_agent_backend_url = std::env::var("IAM_SSH_AGENT_BACKEND_URL").expect("IAM_SSH_AGENT_BACKEND_URL is required");

	if let Some(matches) = matches.subcommand_matches("list-keys") {
		eprintln!("{:?}", matches);
		return;
	}

	if let Some(matches) = matches.subcommand_matches("daemon") {
		// export SSH_AGENT_SOCK
		eprintln!("{:?}", matches);
		return;
	}

	unimplemented!()
}
