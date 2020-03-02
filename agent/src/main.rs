use std::{fs, path::Path};
use ssh_agent::{agent::Agent, proto::{Message, Identity, SignRequest, SignatureBlob}};
use clap::{App, Arg, SubCommand};
use rusoto_core::{region::Region, RusotoError, Client, signature::SignedRequest, request::{HttpResponse, BufferedHttpResponse}};
use url::Url;
use futures::future::Future;
use serde::{Deserialize, Serialize};
use openssh_keys::PublicKey;

#[derive(Debug, Deserialize)]
struct ListIdentities {
	identities: Vec<String>,
}

#[derive(Debug)]
enum ListIdentitiesError {

}

mod service {
	use super::*;

	fn as_base64<S>(vec: &Vec<u8>, serializer: S) -> Result<S::Ok, S::Error>
    where
    	S: serde::Serializer
	{
	    serializer.serialize_str(&base64::encode(&vec[..]))
	}

	#[derive(Debug, Serialize)]
	pub struct SignRequest {
		#[serde(serialize_with = "as_base64")]
		pubkey: Vec<u8>,
		#[serde(serialize_with = "as_base64")]
		data: Vec<u8>,
		flags: u32,
	}

	impl From<super::SignRequest> for SignRequest {
		fn from(req: super::SignRequest) -> Self {
			SignRequest {
				pubkey: req.pubkey_blob,
				data: req.data,
				flags: req.flags,
			}
		}
	}
}

#[derive(Debug, Deserialize)]
struct Signature {

}

#[derive(Debug)]
enum SignError {
	Unknown
}

fn parse_http_list_identities(response: HttpResponse) -> Box<dyn Future<Item = ListIdentities, Error = RusotoError<ListIdentitiesError>> + Send> {
	Box::new(response.buffer().map_err(RusotoError::HttpDispatch).and_then(|buffered: BufferedHttpResponse| {
		match serde_json::from_slice(&buffered.body) {
			Ok(p) => Box::new(futures::future::ok(p)),
			Err(e) => Box::new(futures::future::err(RusotoError::ParseError(format!("{:?}", e)))),
		}
	}))
}

fn parse_http_signature(response: HttpResponse) -> Box<dyn Future<Item = Signature, Error = RusotoError<SignError>> + Send> {
	Box::new(futures::future::err(RusotoError::Service(SignError::Unknown)))
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
				.long("bind-to")
				.short("b")
				.takes_value(true)
				.value_name("PATH")
				.required(true))
		    .help("Run the daemon, bind a UNIX domain socket."))
		.get_matches();

	// Uses an environment variable rather than an argument so that this can be
	// an ECS ValueFrom in an ECS task.
	let ssh_agent_backend_url = Url::parse(&std::env::var("IAM_SSH_AGENT_BACKEND_URL").expect("IAM_SSH_AGENT_BACKEND_URL is required")).expect("IAM_SSH_AGENT_BACKEND_URL is a valid url");
	let agent = AgentBackend::new(ssh_agent_backend_url);

	if let Some(matches) = matches.subcommand_matches("list-keys") {
        eprintln!("{:#?}", agent.identities());
		return;
	}

	if let Some(matches) = matches.subcommand_matches("daemon") {
		let pipe = matches.value_of("bind-to").expect("bind-to is required");
        let pipe = Path::new(pipe);

        if fs::metadata(&pipe).is_ok() {
            if let Ok(_) = fs::remove_file(&pipe){
                println!("Pipe deleted");
            }
        }

        eprintln!("binding to {}", pipe.display());

        let _ = agent.run_unix(&pipe);

        // TODO support exec mode and export SSH_AUTH_SOCK

		return;
	}

	unimplemented!()
}

#[derive(Debug)]
enum AgentBackendError {
	ListIdentities(RusotoError<ListIdentitiesError>),
	Sign(RusotoError<SignError>),
	Unknown(String),
}

struct AgentBackend {
	url: Url,
}

impl AgentBackend {
	fn new(url: Url) -> Self {
		Self {
			url,
		}
	}

	fn fetch_identities(&self) -> Result<ListIdentities, RusotoError<ListIdentitiesError>> {
		let region = Region::default();

		let mut request = SignedRequest::new("GET", "execute-api", &region, &format!("{}/{}", self.url.path(), "identities"));
		request.set_hostname(Some(self.url.host_str().expect("url host").to_owned()));

		Client::shared()
			.sign_and_dispatch(request, parse_http_list_identities)
			.sync()
	}

	fn identities(&self) -> Result<Vec<Identity>, AgentBackendError> {
		let identities = self
			.fetch_identities()
			.map_err(AgentBackendError::ListIdentities)?
			.identities
			.into_iter()
			.filter_map(|identity| {
				PublicKey::parse(&identity).ok().map(|key| {
					Identity {
						pubkey_blob: key.data(),
						comment: key.comment.unwrap_or(String::new()),
					}
				})
			})
			.collect();
		Ok(identities)
	}

	fn fetch_signature(&self, request: &SignRequest) -> Result<Signature, RusotoError<SignError>> {
		let request: service::SignRequest = request.clone().into();

		let bytes = serde_json::to_vec(&request)
			.map_err(|e| RusotoError::ParseError(format!("{:?}", e)))?;

		let region = Region::default();

		let mut request = SignedRequest::new("POST", "execute-api", &region, &format!("{}/{}", self.url.path(), "signature"));
		request.set_hostname(Some(self.url.host_str().expect("url host").to_owned()));
		request.set_payload(Some(bytes));
		request.set_content_type("application/json".to_string());

		Client::shared()
			.sign_and_dispatch(request, parse_http_signature)
			.sync()
	}

	fn sign(&self, request: &SignRequest) -> Result<SignatureBlob, AgentBackendError> {
		let signature = self
			.fetch_signature(request)
			.map_err(AgentBackendError::Sign)?;

		Err(AgentBackendError::Unknown("unimplemented".to_string()))
	}
}

impl Agent for AgentBackend {
	type Error = AgentBackendError;

	fn handle(&self, request: Message) -> Result<Message, Self::Error> {
	    eprintln!("Request: {:#?}", request);

	    let response = match request {
			Message::RequestIdentities => {
				Ok(Message::IdentitiesAnswer(self.identities()?))
			},
			Message::SignRequest(request) => {
				Ok(Message::SignResponse(self.sign(&request)?))
			},
			_ => {
				Err(AgentBackendError::Unknown(format!("received unsupported message: {:?}", request)))
			},
	    };

	    eprintln!("Response {:#?}", response);

	    response
	}
}
