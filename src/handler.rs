//! Handler module of rustnao.  The handler for the SauceNAO API calls.

mod error;
pub use error::{Error, Result, ErrType};

mod constants;

mod sauce;
pub use sauce::Sauce;

mod deserialize;
use deserialize::SauceResult;

use url::Url;
use std::cell::Cell;

/// A handler struct to make SauceNAO API calls.
/// ## Example
/// ```
/// use rustnao::Handler;
/// let handle = Handler::new("your_saucenao_api_key", Some(0), None, None, Some(999), Some(999));
/// ``` 
#[derive(Debug, Clone)]
pub struct Handler {
	api_key : String,
	output_type : i32,
	testmode : Option<i32>,
	num_results : Option<i32>,
	db_mask : Option<Vec<u32>>,
	db_mask_i : Option<Vec<u32>>,
	db : Option<u32>,
	short_limit : Cell<u32>,
	long_limit: Cell<u32>,
	short_left : Cell<u32>,
	long_left : Cell<u32>,
	min_similarity : Cell<f64>,
	filter_empty : Cell<bool>,
}

impl Handler {
	/// Associated index for H-Magazines.
	pub const H_MAGAZINES : u32 = constants::H_MAGAZINES.index;
	/// Associated index for H-Game CG.
	pub const H_GAME_CG : u32 = constants::H_GAME_CG.index;
	/// Associated index for DoujinshiDB.
	pub const DOUJINSHI_DB : u32 = constants::DOUJINSHI_DB.index;
	/// Associated index for Pixiv.
	pub const PIXIV : u32 = constants::PIXIV.index;
	/// Associated index for Nico Nico Seiga.
	pub const NICO_NICO_SEIGA : u32 = constants::NICO_NICO_SEIGA.index;
	/// Associated index for Danbooru.
	pub const DANBOORU : u32 = constants::DANBOORU.index;
	/// Associated index for drawr Images.
	pub const DRAWR : u32 = constants::DRAWR.index;
	/// Associated index for Nijie Images.
	pub const NIJIE : u32 = constants::NIJIE.index;
	/// Associated index for Yand.ere.
	pub const YANDE_RE : u32 = constants::YANDE_RE.index;
	/// Associated index for Shutterstock.
	pub const SHUTTERSTOCK : u32 = constants::SHUTTERSTOCK.index;
	/// Associated index for Fakku.
	pub const FAKKU : u32 = constants::FAKKU.index;
	/// Associated index for H-Misc.
	pub const H_MISC : u32 = constants::H_MISC.index;
	/// Associated index for 2D-Market.
	pub const TWO_D_MARKET : u32 = constants::TWO_D_MARKET.index;
	/// Associated index for MediBang.
	pub const MEDIBANG : u32 = constants::MEDIBANG.index;
	/// Associated index for Anime.
	pub const ANIME : u32 = constants::ANIME.index;
	/// Associated index for H-Anime.
	pub const H_ANIME : u32 = constants::H_ANIME.index;
	/// Associated index for Movies.
	pub const MOVIES : u32 = constants::MOVIES.index;
	/// Associated index for Shows.
	pub const SHOWS : u32 = constants::SHOWS.index;
	/// Associated index for Gelbooru.
	pub const GELBOORU : u32 = constants::GELBOORU.index;
	/// Associated index for Konachan.
	pub const KONACHAN : u32 = constants::KONACHAN.index;
	/// Associated index for Sankaku Channel.
	pub const SANKAKU_CHANNEL : u32 = constants::SANKAKU_CHANNEL.index;
	/// Associated index for Anime-Pictures.net.
	pub const ANIME_PICTURES_NET : u32 = constants::ANIME_PICTURES_NET.index;
	/// Associated index for e621.net.
	pub const E621_NET : u32 = constants::E621_NET.index;
	/// Associated index for Idol Complex.
	pub const IDOL_COMPLEX : u32 = constants::IDOL_COMPLEX.index;
	/// Associated index for bcy.net Illust.
	pub const BCY_NET_ILLUST : u32 = constants::BCY_NET_ILLUST.index;
	/// Associated index for bcy.net Cosplay.
	pub const BCY_NET_COSPLAY : u32 = constants::BCY_NET_COSPLAY.index;
	/// Associated index for PortalGraphics.net.
	pub const PORTALGRAPHICS_NET : u32 = constants::PORTALGRAPHICS_NET.index;
	/// Associated index for deviantArt.
	pub const DEVIANTART : u32 = constants::DEVIANTART.index;
	/// Associated index for Pawoo.net.
	pub const PAWOO_NET : u32 = constants::PAWOO_NET.index;
	/// Associated index for Madokami.
	pub const MADOKAMI : u32 = constants::MADOKAMI.index;
	/// Associated index for Mangadex.
	pub const MANGADEX : u32 = constants::MANGADEX.index;

	/// Grabs the appropriate Source data given an index
	fn get_source(&self, index : u32) -> Option<constants::Source<'_>> {
		let mut result : Option<constants::Source<'_>> = None;
		for src in constants::LIST_OF_SOURCES.iter() {
			if src.index == index {
				result = Some(src.clone());
			}
		}
		result
	}

	/// Generates a bitmask from a given vector.
	fn generate_bitmask(&self, mask : Vec<u32>) -> u32 {
		let mut res : u32 = 0;
		for m in mask {
			let mut offset = 0;
			if m >= 18 {
				offset = 1;	// This seems to be some required fix...
			}
			res ^= u32::pow(2, m - offset);
		}
		res
	}

	/// Generates a url from the given image url
	fn generate_url(&self, image_path : &str, num_results : Option<u32>) -> Result<String> {
		let mut request_url = Url::parse(constants::API_URL)?;
		request_url.query_pairs_mut().append_pair("api_key", self.api_key.as_str());
		request_url.query_pairs_mut().append_pair("output_type", self.output_type.to_string().as_str());

		match self.db {
			Some(val) => {
				request_url.query_pairs_mut().append_pair("db", val.to_string().as_str());
			}
			None => (),
		}

		match &self.db_mask {
			Some(val) => {
				if val.len() > 0 {
					request_url.query_pairs_mut().append_pair("dbmask", self.generate_bitmask(val.clone()).to_string().as_str());
				}
				else if self.db.is_none() {
					// Set to 999.
					request_url.query_pairs_mut().append_pair("db", "999");
				}
			}
			None => {
				if self.db.is_none() {
					// Set to 999.
					request_url.query_pairs_mut().append_pair("db", "999");
				}
			},
		}
			
		match &self.db_mask_i {
			Some(val) => {
				if val.len() > 0 {
					request_url.query_pairs_mut().append_pair("dbmaski", self.generate_bitmask(val.clone()).to_string().as_str());
				}
			}
			None => (),
		}

		match self.testmode {
			Some(val) => {
				request_url.query_pairs_mut().append_pair("testmode", val.to_string().as_str());
			}
			None => {
				request_url.query_pairs_mut().append_pair("testmode", "0");
			}
		}

		match num_results {
			Some(results) => {
				request_url.query_pairs_mut().append_pair("numres", results.to_string().as_str());
			}
			None => {
				match self.num_results {
					Some(val) => {
						request_url.query_pairs_mut().append_pair("numres", val.to_string().as_str());
					}
					None => {
						request_url.query_pairs_mut().append_pair("numres", "999");
					}
				}
			}
		}
			
		if image_path.starts_with("https://") || image_path.starts_with("http://") {
			// Link
			request_url.query_pairs_mut().append_pair("url", image_path);
		}

		Ok(request_url.into_string())
	}

	/// Creates a new Handler object.  By default, SauceNAO sets the short limit is set to 30 seconds, and the long limit is set to 24 hours.
	/// Furthermore, by default on all ``get_sauce`` searches, the minimum simliarity is 0.0, and empty URL searches are not filtered out.
	/// ## Arguments
	/// * `api_key` - A string slice holding your api key.
	/// * `testmode` - An Option for a i32, either 0 or 1.  Causes each index which has to output at most 1 for testing.  If this is None, this is by default 0.
	/// * `db_mask` - A Option for a vector of i32 values representing a mask for which database indices you wish to have enabled.
	/// * `db_mask_i` - A Option for a vector of i32 values representing a mask for which database indices you wish to have disabled.
	/// * `db` - An Option for a u32 value to search for a specific index.  Set to 999 for all.  If this and ``db_mask`` are both empty/None, by default it searches all before ``dbmaski`` is applied.
	/// * `num_results` - An Option for a i32 representing the default number of results you wish returned (you can change this number per-search if you want).  If this is None, this is by default 999.
	/// 
	/// ## Example
	/// ```
	/// use rustnao::Handler;
	/// let handle = Handler::new("your_saucenao_api_key", Some(0), None, None, Some(999), Some(999));
	/// ```
	pub fn new(api_key : &str, testmode : Option<i32>, db_mask : Option<Vec<u32>>, db_mask_i : Option<Vec<u32>>, db : Option<u32>, num_results : Option<i32>) -> Handler {
		Handler {
			api_key : api_key.to_string(),
			output_type : 2,	// This is set to 2 by default, as we need a JSON reply
			testmode : testmode,
			num_results : num_results,
			db_mask : db_mask,
			db_mask_i : db_mask_i,
			db : db,
			short_limit : Cell::new(12),
			long_limit: Cell::new(200),
			short_left : Cell::new(12),
			long_left: Cell::new(200),
			min_similarity : Cell::new(0.0),
			filter_empty : Cell::new(false),
		}
	}

	/// Sets the minimum similarity threshold for ``get_sauce``.  By default this is 0.0. 
	/// ## Arguments
	/// * `similarity` - Represents the minimum similarity threshold (in percent) you wish to set.  It can be any value that can convert to a f64.  This includes f32s, i16s, i32s, and i8s.
	/// 
	/// ## Example
	/// ```
	/// use rustnao::Handler;
	/// let handle = Handler::new("your_saucenao_api_key", Some(0), None, None, Some(999), Some(999));
	/// handle.set_min_similarity(50);
	/// ```
	pub fn set_min_similarity<T : Into<f64>>(&self, similarity : T) {
		self.min_similarity.set(similarity.into());
	}

	/// Sets the whether empty URL results should be automatically filtered for ``get_sauce``.  
	/// ## Arguments
	/// * `enabled` - Represents whether filter should be enabled or not.  By default, this is disabled.
	/// 
	/// ## Example
	/// ```
	/// use rustnao::Handler;
	/// let handle = Handler::new("your_saucenao_api_key", Some(0), None, None, Some(999), Some(999));
	/// handle.set_empty_filter(true);
	/// ```
	pub fn set_empty_filter(&self, enabled : bool) {
		self.filter_empty.set(enabled);
	}

	/// Gets the current short limit as an i32.  By default this is 12.
	/// 
	/// ## Example
	/// ```
	/// use rustnao::Handler;
	/// let handle = Handler::new("your_saucenao_api_key", Some(0), None, None, Some(999), Some(999));
	/// println!("{}", handle.get_short_limit());
	/// ``` 
	pub fn get_short_limit(&self) -> u32 {
		self.short_limit.get()
	}

	/// Gets the current long limit as an i32.  By default this is 200.
	/// 
	/// ## Example
	/// ```
	/// use rustnao::Handler;
	/// let handle = Handler::new("your_saucenao_api_key", Some(0), None, None, Some(999), Some(999));
	/// println!("{}", handle.get_long_limit());
	/// ``` 
	pub fn get_long_limit(&self) -> u32 {
		self.long_limit.get()
	}

	/// Gets the current remaining short limit as an i32.
	/// 
	/// ## Example
	/// ```
	/// use rustnao::Handler;
	/// let handle = Handler::new("your_saucenao_api_key", Some(0), None, None, Some(999), Some(999));
	/// println!("{}", handle.get_current_short_limit());
	/// ``` 
	pub fn get_current_short_limit(&self) -> u32 {
		self.short_left.get()
	}

	/// Gets the current remaining long limit as an i32.
	/// 
	/// ## Example
	/// ```
	/// use rustnao::Handler;
	/// let handle = Handler::new("your_saucenao_api_key", Some(0), None, None, Some(999), Some(999));
	/// println!("{}", handle.get_current_long_limit());
	/// ``` 
	pub fn get_current_long_limit(&self) -> u32 {
		self.long_left.get()
	}

	/// Returns a Result of either a vector of Sauce objects, which contain potential sources for the input ``file``, or a SauceError.
	/// ## Arguments
	/// * ``image_path`` - A string slice that contains the url of the image you wish to look up.
	/// * ``num_results`` - An Option containing a u32 to specify the number of results you wish to get for this specific search.  If this is None, it will default to whatever was originally set in the Handler when it was initalized.  This can be at most 999.
	/// * ``min_similarity`` - An Option containing a f64 to specify the minimum similarity you wish to meet for a result to show up for this specific search.  If this is None, it will default to whatever was originally set in the Handler when it was initalized.
	/// 
	/// ## Example
	/// ```
	/// use rustnao::Handler;
	/// let handle = Handler::new("your_saucenao_api_key", Some(0), None, None, Some(999), Some(999));
	/// handle.get_sauce("./tests/test.jpg", None, None);
	/// ```
	/// 
	/// ## Errors
	/// If there was a problem forming a URL, reading a file, making a request, or parsing the returned JSON, an error will be returned.
	/// Furthermore, if you pass a link in which SauceNAO returns an error code, an error containing the code and message will be returned.
	pub fn get_sauce(&self, image_path : &str, num_results : Option<u32>, min_similarity : Option<f64>) -> Result<Vec<Sauce>> {

		// Check passed in values.
		match num_results {
			Some(num_results) => {
				if num_results > 999 {
					return Err(Error::invalid_parameter("num_results must be less than 999.".to_string()));
				}
			}
			None => (),
		}
		match min_similarity {
			Some(min_similarity) => {
				if min_similarity > 100.0 || min_similarity < 0.0 {
					return Err(Error::invalid_parameter("min_similarity must be less 100.0 and greater than 0.0.".to_string()));
				}
			}
			None => (),
		}

		let url_string = self.generate_url(image_path, num_results)?;
		let mut form_param = reqwest::multipart::Form::new();
		if !(image_path.starts_with("https://") || image_path.starts_with("http://")) {
			form_param = reqwest::multipart::Form::new().file("file", image_path)?;
		}

		let client = reqwest::Client::new();
		let returned_sauce: SauceResult = client.post(&url_string).multipart(form_param).send()?.json()?;
		let mut ret_sauce : Vec<Sauce> = Vec::new();
		if returned_sauce.header.status >= 0 {
			// Update non-sauce fields
			self.short_left.set(returned_sauce.header.short_remaining);
			self.long_left.set(returned_sauce.header.long_remaining);
			self.short_limit.set(returned_sauce.header.short_limit.parse().unwrap());
			self.long_limit.set(returned_sauce.header.long_limit.parse().unwrap());

			// Actual "returned" value:
			match returned_sauce.results {
				Some(res) => {
					let actual_min_sim : f64;
					match min_similarity {
						Some(min_sim) => actual_min_sim = min_sim,
						None => actual_min_sim = self.min_similarity.get(),
					}
					for sauce in res {
						let sauce_min_sim : f64 = sauce.header.similarity.parse().unwrap();
						if (sauce_min_sim >= actual_min_sim) && ((self.filter_empty.get() && sauce.data.ext_urls.len() > 0) || !self.filter_empty.get()){
							let actual_index : u32 = sauce.header.index_name.split(":").collect::<Vec<&str>>()[0].to_string().split("#").collect::<Vec<&str>>()[1].to_string().parse::<u32>().unwrap();
							let source : Option<constants::Source> = self.get_source(actual_index);
							
							match source {								
								Some(src) => {
									ret_sauce.push(sauce::new_sauce(
										sauce.data.ext_urls,
										sauce.data.title,
										src.name.to_string(),
										actual_index,
										sauce.header.index_id,
										sauce.header.similarity.parse().unwrap(),
										sauce.header.thumbnail.to_string(),
										match serde_json::to_value(&sauce.data.additional_fields){
											Ok(x) => Some(x),
											Err(_x) => None,
										},
									));
								}
								None => {
									ret_sauce.push(sauce::new_sauce(
										sauce.data.ext_urls,
										sauce.data.title,
										sauce.header.index_name,
										actual_index,
										sauce.header.index_id,
										sauce.header.similarity.parse().unwrap(),
										sauce.header.thumbnail.to_string(),
										None,
									));
								}
							}
						}
					}
				}
				None => (),
			}
			Ok(ret_sauce)
		}
		else {
			Err(Error::invalid_code(returned_sauce.header.status, returned_sauce.header.message))
		}
		
	}

	/// Returns a string representing a vector of Sauce objects as a serialized JSON, or an error.
	/// ## Arguments
	/// * ``image_path`` - A string slice that contains the url of the image you wish to look up.
	/// * ``num_results`` - An Option containing a u32 to specify the number of results you wish to get for this specific search.  If this is None, it will default to whatever was originally set in the Handler when it was initalized.
	/// * ``min_similarity`` - An Option containing a f64 to specify the minimum similarity you wish to meet for a result to show up for this specific search.  If this is None, it will default to whatever was originally set in the Handler when it was initalized.
	/// 
	/// ## Example
	/// ```
	/// use rustnao::Handler;
	/// let handle = Handler::new("your_saucenao_api_key", Some(0), None, None, Some(999), Some(999));
	/// handle.get_sauce_as_pretty_json("https://i.imgur.com/W42kkKS.jpg", None, None);
	/// ```
	/// 
	/// ## Errors
	/// If there was a problem forming a URL, reading a file, making a request, or parsing the returned JSON, an error will be returned.
	/// Furthermore, if you pass a link in which SauceNAO returns an error code, an error containing the code and message will be returned.
	pub fn get_sauce_as_pretty_json(&self, image_path : &str, num_results : Option<u32>, min_similarity : Option<f64>) -> Result<String> {
		let ret_sauce = self.get_sauce(image_path, num_results, min_similarity)?;
		Ok(serde_json::to_string_pretty(&ret_sauce)?)
	}

	/// Returns a string representing a vector of Sauce objects as a serialized JSON, or an error.
	/// ## Arguments
	/// * ``image_path`` - A string slice that contains the url of the image you wish to look up.
	/// * ``num_results`` - An Option containing a u32 to specify the number of results you wish to get for this specific search.  If this is None, it will default to whatever was originally set in the Handler when it was initalized.
	/// * ``min_similarity`` - An Option containing a f64 to specify the minimum similarity you wish to meet for a result to show up for this specific search.  If this is None, it will default to whatever was originally set in the Handler when it was initalized.
	/// 
	/// ## Example
	/// ```
	/// use rustnao::Handler;
	/// let mut handle = Handler::new("your_saucenao_api_key", Some(0), None, None, Some(999), Some(5));
	/// handle.get_sauce_as_json("https://i.imgur.com/W42kkKS.jpg", None, None);
	/// ```
	/// 
	/// ## Errors
	/// If there was a problem forming a URL, reading a file, making a request, or parsing the returned JSON, an error will be returned.
	/// Furthermore, if you pass a link in which SauceNAO returns an error code, an error containing the code and message will be returned.
	pub fn get_sauce_as_json(&self, image_path : &str, num_results : Option<u32>, min_similarity : Option<f64>) -> Result<String> {
		let ret_sauce = self.get_sauce(image_path, num_results, min_similarity)?;
		Ok(serde_json::to_string(&ret_sauce)?)
	}

	/* TODO: Async
	async fn get_sauce_async(&self, url : &str) -> Result<Sauce, SauceError> {

	}

	async fn get_sauce_as_json_async(&self, url : &str) -> Result<String, SauceError> {

	}*/
}

/// A trait to convert to JSON and pretty JSON strings.
pub trait ToJSON {
	/// Converts to a Result containing a JSON string.
	/// ### Errors
	/// There may be a problem converting the object to a JSON string.
	fn to_json(&self) -> Result<String>;

	/// Converts to a Result containing a pretty JSON string.
	/// ### Errors
	/// There may be a problem converting the object to a JSON string.
	fn to_json_pretty(&self) -> Result<String>;
}

impl ToJSON for Vec<Sauce> {
	/// Converts a Sauce vector into a pretty JSON string.
	/// 
	/// ```
	/// use rustnao::{Handler, ToJSON};
	/// let handle = Handler::new("your_saucenao_api_key", Some(0), None, None, Some(999), Some(999));
	/// let result = handle.get_sauce("./tests/test.jpg", None, None);
	/// if result.is_ok() {
	/// 	result.unwrap().to_json_pretty();
	/// }
	/// ```
	fn to_json_pretty(&self) -> Result<String> {
		Ok(serde_json::to_string_pretty(self)?)
	}

	/// Converts a Sauce vector into a JSON string.
	/// 
	/// ```
	/// use rustnao::{Handler, ToJSON};
	/// let handle = Handler::new("your_saucenao_api_key", Some(0), None, None, Some(999), Some(999));
	/// let result = handle.get_sauce("./tests/test.jpg", None, None);
	/// if result.is_ok() {
	/// 	result.unwrap().to_json();
	/// }
	/// ```
	fn to_json(&self) -> Result<String> {
		Ok(serde_json::to_string(self)?)
	}
}