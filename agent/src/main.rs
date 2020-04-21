use std::{env, fs, path::Path};
use ssh_agent::agent::Agent;
use clap::{App, Arg, SubCommand};
use url::Url;

mod agent;

fn main() {
    let _ = env_logger::try_init();

    unsafe {
        signal_hook::register(signal_hook::SIGTERM, || std::process::exit(0)).expect("register SIGTERM handler");
        signal_hook::register(signal_hook::SIGINT, || std::process::abort()).expect("register SIGINT handler");
    }

    let about = env!("CARGO_PKG_DESCRIPTION").to_owned() + "\n\nAll commands require the IAM_SSH_AGENT_BACKEND_URL environment variable to be set e.g. https://${ApiId}.execute-api.${Region}.amazonaws.com/${Stage}";
    let about = textwrap::fill(&about, 80);

    let matches = App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(about.as_str())
        .subcommand(SubCommand::with_name("list-keys")
            .about("List all keys for the caller's IAM identity."))
        .subcommand(SubCommand::with_name("daemon")
            .about("Run the daemon, bind a UNIX domain socket to provide ssh list and sign operations. Defaults to SSH_AUTH_SOCK if unspecified.")
            .arg(Arg::with_name("bind-to")
                .long("bind-to")
                .short("b")
                .takes_value(true)
                .value_name("PATH")))
        .setting(clap::AppSettings::SubcommandRequired)
        .get_matches();

    // Uses an environment variable rather than an argument so that this can be
    // an ECS ValueFrom in an ECS task.
    let ssh_agent_backend_url = Url::parse(&env::var("IAM_SSH_AGENT_BACKEND_URL").expect("IAM_SSH_AGENT_BACKEND_URL is required")).expect("IAM_SSH_AGENT_BACKEND_URL is a valid url");
    let agent = agent::Backend::new(ssh_agent_backend_url);

    if let Some(_matches) = matches.subcommand_matches("list-keys") {
        eprintln!("{:#?}", agent.identities());
        return;
    }

    if let Some(matches) = matches.subcommand_matches("daemon") {
        // TODO support exec mode and export SSH_AUTH_SOCK

        // Support command line for testing and an environment variable for
        // systemd units.
        let pipe = matches.value_of("bind-to").expect("bind-to is required");
        let pipe = Path::new(pipe);

        if fs::metadata(&pipe).is_ok() {
            if let Ok(_) = fs::remove_file(&pipe) {
                println!("fn=main pipe={} at=deleted", pipe.display());
            }
        }

        eprintln!("fn=main pipe={} at=bind", pipe.display());

        match agent.run_unix(&pipe) {
            Err(e) => eprintln!("fn=main pipe={} at=bind err={:?}", pipe.display(), e),
            Ok(_) => {},
        }

        eprintln!("fn=main pipe={} at=finished", pipe.display());

        std::process::exit(1);
    }

    unreachable!()
}
