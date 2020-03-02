use std::{fs, path::Path, os::unix::net::UnixListener};
use ssh_agent::{SSHAgentHandler, error::HandleResult, Response, Identity};
use clap::{App, Arg, SubCommand};
use rusoto_core::{region::Region, RusotoError, Client, signature::SignedRequest, request::HttpResponse};
use url::Url;
use futures::future::Future;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct ListIdentities {
	identities: Vec<String>,
}

#[derive(Debug)]
enum ListIdentitiesError {

}

fn parse_http_list_identities(response: HttpResponse) -> Box<dyn Future<Item = ListIdentities, Error = RusotoError<ListIdentitiesError>> + Send> {
	let response = response.buffer().wait().expect("body");

	let body: ListIdentities = serde_json::from_slice(&response.body).expect("parse");

	Box::new(futures::future::ok(body))
}

fn main() {
    let matches = App::new("iam-ssh-agent")
		.version("1.0")
		.author("Keith Duncan <keith_duncan@me.com>")
		.about("Use an AWS IAM SSH Agent backend for SSH Authentication")
		.subcommand(SubCommand::with_name("list-keys")
		    .help("List all keys for the caller IAM identity."))
		.subcommand(SubCommand::with_name("daemon")
			.arg(Arg::with_name("bind-to")
				.required(true))
		    .help("Run the daemon, bind a UNIX domain socket."))
		.get_matches();

	// Uses an environment variable rather than an argument so that this can be
	// an ECS ValueFrom in an ECS task.
	let ssh_agent_backend_url = Url::parse(&std::env::var("IAM_SSH_AGENT_BACKEND_URL").expect("IAM_SSH_AGENT_BACKEND_URL is required")).expect("valid url");
	let mut handler = Handler {
		url: ssh_agent_backend_url,
	};

	if let Some(matches) = matches.subcommand_matches("list-keys") {
        eprintln!("{:#?}", handler.list_identities());
		return;
	}

	if let Some(matches) = matches.subcommand_matches("daemon") {
		let pipe = matches.value_of("bind-to").unwrap();
        let pipe = Path::new(pipe);

        if fs::metadata(&pipe).is_ok() {
            if let Ok(_) = fs::remove_file(&pipe){
                println!("Pipe deleted");
            }
        }

        eprintln!("binding to {}", pipe.display());

        let listener = UnixListener::bind(pipe).unwrap();
        ssh_agent::Agent::run(handler, listener);

        // TODO support exec mode and export SSH_AGENT_SOCK

		return;
	}

	unimplemented!()
}

struct Handler {
	url: Url,
}

impl Handler {
	fn list_identities(&mut self) -> ListIdentities {
		let region = Region::default();

		let mut request = SignedRequest::new("GET", "execute-api", &region, &format!("{}/{}", self.url.path(), "identities"));
		request.set_hostname(Some(self.url.host_str().expect("url host").to_owned()));

		Client::shared()
			.sign_and_dispatch(request, parse_http_list_identities)
			.sync()
			.expect("response")
	}
}

impl SSHAgentHandler for Handler {
	fn new() -> Self {
		unimplemented!()
	}

	fn identities(&mut self) -> HandleResult<Response> {
		unimplemented!()
	}

	fn sign_request(&mut self, pubkey: Vec<u8>, data: Vec<u8>, _flags: u32) -> HandleResult<Response> {
		unimplemented!()
	}
}
