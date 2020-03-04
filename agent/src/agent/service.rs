use rusoto_core::{RusotoError, request::{HttpResponse, BufferedHttpResponse}};
use ssh_agent::proto::SignatureBlob;
use futures::future::Future;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct ListIdentities {
	pub identities: Vec<String>,
}

#[derive(Debug)]
pub enum ListIdentitiesError {

}

mod base64 {
	use serde::Deserialize;

	pub fn serialize<S>(vec: &Vec<u8>, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer
	{
	    serializer.serialize_str(&base64::encode(&vec[..]))
	}

	pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
	where
		D: serde::Deserializer<'de>
	{
		let s = <&str>::deserialize(deserializer)?;
	    base64::decode(s).map_err(serde::de::Error::custom)
	}
}

#[derive(Debug, Serialize)]
pub struct SignRequest {
	#[serde(with = "base64")]
	pubkey: Vec<u8>,
	#[serde(with = "base64")]
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

#[derive(Debug, Deserialize)]
pub struct Signature {
	#[serde(with = "base64")]
	signature: Vec<u8>,
}

#[derive(Debug)]
pub enum SignError {
	
}

impl Into<SignatureBlob> for Signature {
	fn into(self) -> SignatureBlob {
		self.signature
	}
}

pub fn parse_http_list_identities(response: HttpResponse) -> Box<dyn Future<Item = ListIdentities, Error = RusotoError<ListIdentitiesError>> + Send> {
	Box::new(response.buffer().map_err(RusotoError::HttpDispatch).and_then(|buffered: BufferedHttpResponse| {
		eprintln!("mod=service fn=parse_http_list_identities response={:?}", buffered);

		match serde_json::from_slice(&buffered.body) {
			Ok(p) => Box::new(futures::future::ok(p)),
			Err(e) => Box::new(futures::future::err(RusotoError::ParseError(format!("{:?}", e)))),
		}
	}))
}

pub fn parse_http_signature(response: HttpResponse) -> Box<dyn Future<Item = Signature, Error = RusotoError<SignError>> + Send> {
	Box::new(response.buffer().map_err(RusotoError::HttpDispatch).and_then(|buffered: BufferedHttpResponse| {
		eprintln!("mod=service fn=parse_http_signature response={:?}", buffered);

		match serde_json::from_slice(&buffered.body) {
			Ok(p) => Box::new(futures::future::ok(p)),
			Err(e) => Box::new(futures::future::err(RusotoError::ParseError(format!("{:?}", e)))),
		}
	}))
}