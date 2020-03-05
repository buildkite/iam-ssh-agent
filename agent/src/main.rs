use std::{fs, path::Path};
use ssh_agent::agent::Agent;
use clap::{App, Arg, SubCommand};
use url::Url;

mod agent;

fn main() {
	let _ = env_logger::try_init();

    let matches = App::new("iam-ssh-agent")
		.version("0.1.0")
		.author("Keith Duncan <keith_duncan@me.com>")
		.about("Forward ssh list and sign operations to an iam-ssh-agent backend, receive access to keys based on your IAM identity.")
		.subcommand(SubCommand::with_name("list-keys")
			.about("List all keys for the caller's IAM identity."))
		.subcommand(SubCommand::with_name("daemon")
			.about("Run the daemon, bind a UNIX domain socket to provide ssh list and sign operations.")
			.arg(Arg::with_name("bind-to")
				.long("bind-to")
				.short("b")
				.takes_value(true)
				.value_name("PATH")
				.required(true)))
		.setting(clap::AppSettings::SubcommandRequired)
		.get_matches();

	// Uses an environment variable rather than an argument so that this can be
	// an ECS ValueFrom in an ECS task.
	let ssh_agent_backend_url = Url::parse(&std::env::var("IAM_SSH_AGENT_BACKEND_URL").expect("IAM_SSH_AGENT_BACKEND_URL is required")).expect("IAM_SSH_AGENT_BACKEND_URL is a valid url");
	let agent = agent::Backend::new(ssh_agent_backend_url);

	if let Some(_matches) = matches.subcommand_matches("list-keys") {
        eprintln!("{:#?}", agent.identities());
		return;
	}

	if let Some(matches) = matches.subcommand_matches("daemon") {
		// TODO support exec mode and export SSH_AUTH_SOCK

		let pipe = matches.value_of("bind-to").expect("bind-to is required");
        let pipe = Path::new(pipe);

        if fs::metadata(&pipe).is_ok() {
            if let Ok(_) = fs::remove_file(&pipe){
                println!("pipe deleted");
            }
        }

        eprintln!("binding to {}", pipe.display());

        let _ = agent.run_unix(&pipe);
		return;
	}

	unreachable!()
}
