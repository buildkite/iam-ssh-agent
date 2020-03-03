use ssh_agent::{agent::Agent, proto::{Message, Identity, SignRequest, SignatureBlob}};
use rusoto_core::{region::Region, RusotoError, Client, signature::SignedRequest};
use url::Url;
use openssh_keys::PublicKey;

mod service;

#[derive(Debug)]
pub enum BackendError {
	ListIdentities(RusotoError<service::ListIdentitiesError>),
	Sign(RusotoError<service::SignError>),
	Unknown(String),
}

pub struct Backend {
	url: Url,
}

impl Backend {
	pub fn new(url: Url) -> Self {
		Self {
			url,
		}
	}

	pub fn fetch_identities(&self) -> Result<service::ListIdentities, RusotoError<service::ListIdentitiesError>> {
		let region = Region::default();

		let mut request = SignedRequest::new("GET", "execute-api", &region, &format!("{}/{}", self.url.path(), "identities"));
		request.set_hostname(Some(self.url.host_str().expect("url host").to_owned()));

		Client::shared()
			.sign_and_dispatch(request, service::parse_http_list_identities)
			.sync()
	}

	pub fn identities(&self) -> Result<Vec<Identity>, BackendError> {
		let identities = self
			.fetch_identities()
			.map_err(BackendError::ListIdentities)?
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

	pub fn fetch_signature(&self, request: &SignRequest) -> Result<service::Signature, RusotoError<service::SignError>> {
		let request: service::SignRequest = request.clone().into();

		let bytes = serde_json::to_vec(&request)
			.map_err(|e| RusotoError::ParseError(format!("{:?}", e)))?;

		let region = Region::default();

		let mut request = SignedRequest::new("POST", "execute-api", &region, &format!("{}/{}", self.url.path(), "signature"));
		request.set_hostname(Some(self.url.host_str().expect("url host").to_owned()));
		request.set_payload(Some(bytes));
		request.set_content_type("application/json".to_string());

		Client::shared()
			.sign_and_dispatch(request, service::parse_http_signature)
			.sync()
	}

	pub fn sign(&self, request: &SignRequest) -> Result<SignatureBlob, BackendError> {
		let signature = self
			.fetch_signature(request)
			.map_err(BackendError::Sign)?
			.into();
		Ok(signature)
	}
}

impl Agent for Backend {
	type Error = BackendError;

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
				Err(BackendError::Unknown(format!("received unsupported message: {:?}", request)))
			},
	    };

	    eprintln!("Response {:#?}", response);

	    response
	}
}