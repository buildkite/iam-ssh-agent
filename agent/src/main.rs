use std::os::unix::net::{UnixListener};
use ssh_agent::SSHAgentHandler;
use clap::{App, SubCommand};
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
		    .help("Run the daemon, bind a UNIX domain socket."))
		.get_matches();

	// Uses an environment variable rather than an argument so that this can be
	// an ECS ValueFrom in an ECS task.
	let ssh_agent_backend_url = Url::parse(&std::env::var("IAM_SSH_AGENT_BACKEND_URL").expect("IAM_SSH_AGENT_BACKEND_URL is required")).expect("valid url");

	if let Some(matches) = matches.subcommand_matches("list-keys") {
		eprintln!("{:?}", matches);

		let region = Region::default();

		let mut request = SignedRequest::new("GET", "execute-api", &region, &format!("{}/{}", ssh_agent_backend_url.path(), "identities"));
		request.set_hostname(Some(ssh_agent_backend_url.host_str().expect("url host").to_owned()));

		let response = Client::shared()
            .sign_and_dispatch(request, parse_http_list_identities)
            .sync()
            .expect("response");

        eprintln!("{:?}", response);
        
		return;
	}

	if let Some(matches) = matches.subcommand_matches("daemon") {
		// export SSH_AGENT_SOCK
		eprintln!("{:?}", matches);
		return;
	}

	unimplemented!()
}
