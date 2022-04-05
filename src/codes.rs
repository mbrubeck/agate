/// The server was unable to parse the client's request, presumably due to a malformed request. (cf HTTP 400)
pub const BAD_REQUEST: u8 = 59;
/// The request was for a resource at a domain not served by the server and the server does not accept proxy requests.
pub const PROXY_REQUEST_REFUSED: u8 = 53;
/// The requested resource could not be found but may be available in the future. (cf HTTP 404)
pub const NOT_FOUND: u8 = 51;
/// The resource requested is no longer available and will not be available again. Search engines and similar tools should remove this resource from their indices. Content aggrefators should stop requesting the resource and convey to their human users that the subscribed resource is gone. (cf HTTP 410)
pub const GONE: u8 = 52;
/// The requested resource should be consistently requested from the new URL provided in the future. Tools loke search engine indexers or content aggregators should update their configurations to avoid requesting the old URL, and end-user clients may automatically update bookmarks, etc. Note that clients that only pay attention to the initial digit of status codes will treat this as a temporary redirect. They will still end up at the right place, they just won't be able to make use of the knowledge that this redirect is permanent, so they'll pay a small performance penality by having to follow the redirect each time.
pub const REDIRECT_PERMANENT: u8 = 31;
/// The request was handled successfully and a response body will follow the response header. The <META> line is a MIME media type which applies to the response body.
pub const SUCCESS: u8 = 20;
