use anyhow::Result;
use core::fmt;
use maplit::btreemap;
use nom::branch::alt;
use nom::bytes::streaming::{tag, take_until};
use nom::character::complete::not_line_ending;
use nom::character::streaming::{crlf, space1};
use nom::combinator::map;
use nom::multi::separated_list0;
use nom::sequence::separated_pair;
use nom::IResult;
use rustls::pki_types::ServerName;
use std::collections::BTreeMap;
use std::io;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt, ErrorKind};
use tokio::net::TcpStream;
use tokio_rustls::client::TlsStream;
use tokio_rustls::TlsConnector;
use url::Url;

#[derive(Copy, Clone, Debug)]
pub enum Methods {
    Options,
    Describe,
    Setup,
    Play,
    Teardown,
}

impl fmt::Display for Methods {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Methods::Options => f.write_str("OPTIONS"),
            Methods::Describe => f.write_str("DESCRIBE"),
            Methods::Setup => f.write_str("SETUP"),
            Methods::Play => f.write_str("PLAY"),
            Methods::Teardown => f.write_str("TEARDOWN"),
        }
    }
}

// Don't need this: the methods aren't returned as part of the standard header. We may require this
// if we start parsing the OPTIONS response.
// impl Methods {
//     fn parse(i: &[u8]) -> IResult<&[u8], Self> {
//         alt((
//             map(tag(b"OPTIONS"), |_| Self::Options),
//             map(tag(b"DESCRIBE"), |_| Self::Describe),
//             map(tag(b"SETUP"), |_| Self::Setup),
//             map(tag(b"PLAY"), |_| Self::Play),
//             map(tag(b"TEARDOWN"), |_| Self::Teardown),
//         ))(i)
//     }
// }

pub struct Rtsps {
    cseq: u32,
    tcp_addr: SocketAddr,
    pub stream: TlsStream<TcpStream>,
    auth: Option<String>,
    pub frame_buffer: Vec<u8>,
    temp_buffer: Vec<u8>,
}

impl Rtsps {
    pub async fn new(addr: &str) -> Result<Self> {
        let url = Url::parse(addr);

        let socket_addr = match url.clone() {
            Ok(parsed_addr) => parsed_addr.socket_addrs(|| None)?,
            Err(e) => panic!("Trying to parse {addr} resulted in {e}"),
        };

        let mut config = rustls::ClientConfig::builder()
            .dangerous()
            .with_custom_certificate_verifier(Arc::new(crate::no_auth::NoAuth::new()))
            .with_no_client_auth();

        config.key_log = Arc::new(rustls::KeyLogFile::new());

        let connector = TlsConnector::from(Arc::new(config));

        let tcp_stream = TcpStream::connect(socket_addr[0]).await?;

        let domain = ServerName::try_from(url.unwrap().domain().expect("No domain"))
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid dnsname"))?
            .to_owned();

        let tcp_stream = connector.connect(domain, tcp_stream).await?;

        log::debug!("Connecting to server at: {}", socket_addr[0]);

        Ok(Self {
            tcp_addr: socket_addr[0],
            stream: tcp_stream,
            auth: None,
            cseq: 0,
            frame_buffer: Vec::new(),
            temp_buffer: vec![0u8; 4096],
        })
    }

    pub async fn open_stream(&mut self, path: &str) -> Result<()> {
        log::info!("Open stream to {}", path);

        // Send OPTIONS
        self.request_response(Methods::Options, BTreeMap::new(), &path).await?;

        // Send DESCRIBE
        let _headers = self
            .authenticated_request_response(Methods::Describe, BTreeMap::new(), &path)
            .await?;

        // Hard code stream location
        // TODO: Extract stream path from `Content-Base: rtsps://192.168.0.96/streaming/live/1/`
        // header and `a=control:track1` in body
        let track_path = format!("{}/track1", path);

        // Send SETUP with `Transport`, etc headers
        let headers = self
            .authenticated_request_response(
                Methods::Setup,
                btreemap! {
                    // Value hard coded from `ffplay` Wireshark capture
                    "Transport" => "RTP/AVP/TCP;unicast;interleaved=0-1".to_string()
                },
                &track_path,
            )
            .await?;

        // Extract session token from SETUP response
        let session_token = headers
            .headers
            .get("Session")
            // Extract e.g. "061546E4" from "061546E4;timeout=10"
            .and_then(|v| v.split_once(';'))
            .map(|(session_token, _rest)| session_token.to_string())
            .ok_or_else(|| anyhow::anyhow!("No session token found in response"))?;

        // Hard code range to beginning of stream
        let range = "npt=0.000-".to_string();

        self.frame_buffer.clear();

        // Send PLAY with session token and range header
        let _headers = self
            .authenticated_request_response(
                Methods::Play,
                btreemap! {
                    "Session" => session_token,
                    "Range" => range
                },
                &path,
            )
            .await?;

        Ok(())
    }

    async fn authenticated_request_response(
        &mut self,
        method: Methods,
        extra_headers: BTreeMap<&str, String>,
        track: &str,
    ) -> Result<RtspResponse> {
        // Attempt request
        let headers = self.request_response(method, extra_headers.clone(), track).await?;

        // If unauthorised, authenticate, then try original request again
        let headers = if headers.status == RtpStatusCode::Unauthorized {
            log::debug!("--> Request is unauthorised");

            let auth_header = headers
                .headers
                .get("WWW-Authenticate")
                .expect("Needs digest auth, but no digest header present");

            log::debug!("----> Digest header: {}", auth_header);

            // https://github.com/scottlamb/http-auth/blob/main/examples/reqwest.rs

            let mut pw_client = http_auth::PasswordClient::try_from(auth_header.as_str()).expect("Password client");

            log::debug!("----> Password client {:?}", pw_client);

            let username = "bblp";
            // TODO: Pass in password from config
            let password = "192190e7";

            let authorization = pw_client
                .respond(&http_auth::PasswordParams {
                    username,
                    password,
                    uri: &format!("rtsps://{}{}", self.tcp_addr.ip(), track),
                    method: &method.to_string(),
                    body: Some(&[]),
                })
                .expect("Respond");

            // Cache auth for next time
            self.auth = Some(authorization.clone());

            log::debug!("----> Auth reply {:?}", authorization);

            // Make request again, this time with an additional authorization header.
            self.request_response(method, extra_headers, track).await?
        } else {
            headers
        };

        if headers.status == RtpStatusCode::Unauthorized {
            Err(anyhow::anyhow!("Failed to authorise"))
        } else {
            Ok(headers)
        }
    }

    /// Send a request and return its response headers and body
    async fn request_response(
        &mut self,
        method: Methods,
        mut extra_headers: BTreeMap<&str, String>,
        track: &str,
    ) -> Result<RtspResponse> {
        self.cseq += 1;

        let method_str = method.to_string();

        log::debug!("Send {} request", method_str);

        let mut headers = btreemap! {
            "CSeq" => self.cseq.to_string(),
            "User-Agent" => "Machine-Api".to_string(),
        };

        if let Some(auth) = self.auth.as_ref() {
            headers.insert("Authorization", auth.clone());
        }

        headers.append(&mut extra_headers);

        let headers = headers
            .into_iter()
            .map(|(k, v)| format!("{}: {}", k, v))
            .collect::<Vec<_>>()
            .join("\r\n");

        // NOTE: Double newline (\r\n\r\n) delimits header and request body (which is always empty
        // for RTP control messages)
        let request = format!(
            "{} rtsps://{}{} RTSP/1.0\r\n{}\r\n\r\n",
            method_str, self.tcp_addr, track, headers
        );

        log::debug!("--> Request\n\n{}", request);

        self.stream.get_mut().0.writable().await?;

        self.stream.write_all(request.as_bytes()).await?;

        let mut buf_size = 0;
        let mut arr = [0u8; 4096];
        let mut buf = arr.as_mut_slice();

        let headers = loop {
            match self.stream.read(&mut buf).await {
                Ok(n) => {
                    buf_size += n;

                    match RtspResponse::parse(&buf[0..buf_size]) {
                        Ok((_rest, response)) => {
                            break response;
                        }
                        Err(e) => match e {
                            nom::Err::Incomplete(more) => {
                                log::debug!("Waiting for more {:?}", more)
                            }
                            nom::Err::Error(e) => {
                                log::debug!("Error {:?}", e.code);
                            }
                            nom::Err::Failure(e) => {
                                log::error!("Parse failure: {:?}", e.code);

                                return Err(anyhow::anyhow!("Response parse error: {:?}", e.code));
                            }
                        },
                    }
                }
                Err(e) if e.kind() == ErrorKind::WouldBlock => {
                    continue;
                }
                Err(e) => {
                    return Err(e.into());
                }
            }
        };

        log::debug!("--> Response headers\n\n{}", headers);

        if headers.status == RtpStatusCode::NotFound {
            return Err(anyhow::anyhow!("Path {} not found", track));
        }

        Ok(headers)
    }

    pub async fn read_more(&mut self) -> Result<&[u8]> {
        let mut buf = self.temp_buffer.as_mut_slice();

        let n = self.stream.read(&mut buf).await?;

        let buf = &buf[0..n];

        // self.frame_buffer.extend(&buf[..]);

        // dbg!(&self.frame_buffer[(self.frame_buffer.len() - 10)..]);
        // dbg!(&buf[0..buf.len().min(10)]);

        Ok(buf)
    }
}

// RTSP header begins with this tag
const HEADER_START_TOKEN: &[u8] = b"RTSP/1.0";
// Header and body are separated by two newlines
const HEADER_END_TOKEN: &[u8] = b"\r\n\r\n";

// TODO: More status codes from https://github.com/FFmpeg/FFmpeg/blob/deee00e2eb58710f21c1c8775702930bc4d9f86b/libavformat/rtspdec.c#L47-L58
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u16)]
enum RtpStatusCode {
    Ok = 200u16,
    Unauthorized = 401,
    NotFound = 404,
    Other(u16),
}

impl RtpStatusCode {
    fn parse(i: &[u8]) -> IResult<&[u8], Self> {
        alt((
            map(tag(b"200 OK"), |_| Self::Ok),
            map(tag(b"401 Unauthorized"), |_| Self::Unauthorized),
            map(tag(b"404 Stream Not Found"), |_| Self::NotFound),
            map(
                separated_pair(nom::character::streaming::u16, space1, not_line_ending),
                |(n, _text_status)| Self::Other(n),
            ),
        ))(i)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RtspResponse {
    status: RtpStatusCode,
    headers: BTreeMap<String, String>,
}

impl RtspResponse {
    fn parse(i: &[u8]) -> IResult<&[u8], Self> {
        let (i, _discard_preamble) = take_until(HEADER_START_TOKEN)(i)?;

        let (i, (_start_token, status)) = separated_pair(tag(HEADER_START_TOKEN), space1, RtpStatusCode::parse)(i)?;

        let (i, _) = crlf(i)?;

        let (i, headers) = Self::parse_pairs(i)?;

        Ok((i, Self { status, headers }))
    }

    fn parse_pairs(i: &[u8]) -> IResult<&[u8], BTreeMap<String, String>> {
        let (i, headers) = separated_list0(
            crlf,
            map(
                separated_pair(
                    nom::bytes::complete::take_until(": "),
                    nom::bytes::complete::tag(": "),
                    not_line_ending,
                ),
                |(key, value)| {
                    // Headers are ASCII (or the bits we care about are anyway), so we don't mind
                    // losing some fancy characters here
                    (
                        String::from_utf8_lossy(key).trim().to_string(),
                        String::from_utf8_lossy(value).trim().to_string(),
                    )
                },
            ),
        )(i)?;

        let headers = headers.into_iter().collect::<BTreeMap<_, _>>();

        let (i, _header_body_separator) = tag(HEADER_END_TOKEN)(i)?;

        Ok((i, headers))
    }
}

impl fmt::Display for RtspResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let headers = self
            .headers
            .iter()
            .map(|(k, v)| format!("{}: {}", k, v))
            .collect::<Vec<_>>()
            .join("\n");

        write!(f, "{:?}\n{}", self.status, headers)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use maplit::btreemap;

    // Tests that we can parse a header out of packets interleaved with video frames
    #[test]
    fn parse_header_inside_rtp_stream() {
        let preamble_garbage = vec![36u8, 1, 0, 48];

        let header = b"RTSP/1.0 401 Unauthorized\r\nCSeq: 2\r\nDate: Fri, Aug 09 2024 14:00:40 GMT\r\nWWW-Authenticate: Digest realm=\"LIVE555 Streaming Media\", nonce=\"3b8d6b98cb67fb38af1cd3ae50ec393d\"\r\n\r\n".to_vec();

        let postamble_garbage = vec![
            36u8, 1, 0, 48, 128, 200, 0, 6, 116, 243, 37, 127, 234, 96, 159, 202, 2, 208, 111, 239, 158, 100, 214, 132,
            0, 0, 0, 0, 0, 0, 0, 0, 129, 202, 0, 4, 116, 243, 37, 127, 1, 7,
        ];

        let all = vec![preamble_garbage.clone(), header.clone(), postamble_garbage.clone()]
            .iter()
            .flatten()
            .copied()
            .collect::<Vec<u8>>();

        let parsed = RtspResponse::parse(&all);

        let expected = (
            &postamble_garbage[..],
            RtspResponse {
                status: RtpStatusCode::Unauthorized,
                headers: btreemap! {
                    "CSeq".to_string() => "2".to_string(),
                    "Date".to_string() => "Fri, Aug 09 2024 14:00:40 GMT".to_string(),
                    "WWW-Authenticate".to_string() => "Digest realm=\"LIVE555 Streaming Media\", nonce=\"3b8d6b98cb67fb38af1cd3ae50ec393d\"".to_string()
                },
            },
        );

        assert_eq!(parsed, Ok(expected));
    }

    #[test]
    fn find_header_in_garbage() {
        let input = &b"$0*mh`fb5.c*mhBL-P001RTSP/1.0 200 OK\r\nCSeq: 6\r\nDate: Fri, Aug 09 2024 15:27:28 GMT\r\nRange: npt=0.000-\r\nSession: A6A00543\r\nRTP-Info: url=rtsps://192.168.0.96/streaming/live/1/track1;seq=61748;rtptime=2162568838\r\n\r\n"[..];

        let parsed = RtspResponse::parse(input);

        let expected = (
            &[][..],
            RtspResponse {
                status: RtpStatusCode::Ok,
                headers: btreemap! {
                    "CSeq".to_string() => "6".to_string(),
                    "Date".to_string() => "Fri, Aug 09 2024 15:27:28 GMT".to_string(),
                    "Range".to_string() => "npt=0.000-".to_string(),
                    "Session".to_string() => "A6A00543".to_string(),
                    "RTP-Info".to_string() => "url=rtsps://192.168.0.96/streaming/live/1/track1;seq=61748;rtptime=2162568838".to_string()
                },
            },
        );

        assert_eq!(parsed, Ok(expected));
    }

    #[test]
    fn unknown_status() {
        assert_eq!(
            RtpStatusCode::parse(b"418 Teapot"),
            Ok((&[][..], RtpStatusCode::Other(418)))
        );
    }

    #[test]
    fn parse_key_values() {
        let input = &b"CSeq: 1\r\nDate: Fri, Aug 09 2024 14:46:40 GMT\r\nPublic: OPTIONS, DESCRIBE, SETUP, TEARDOWN, PLAY, PAUSE, GET_PARAMETER, SET_PARAMETER\r\n\r\n"[..];

        assert_eq!(
            RtspResponse::parse_pairs(input),
            Ok((
                &[][..],
                btreemap! {
                    "CSeq".to_string() => "1".to_string(),
                    "Date".to_string() => "Fri, Aug 09 2024 14:46:40 GMT".to_string(),
                    "Public".to_string() => "OPTIONS, DESCRIBE, SETUP, TEARDOWN, PLAY, PAUSE, GET_PARAMETER, SET_PARAMETER".to_string()
                }
            ))
        );
    }

    #[test]

    fn parse_header() {
        let header = &b"RTSP/1.0 401 Unauthorized\r\nCSeq: 2\r\nDate: Fri, Aug 09 2024 14:00:40 GMT\r\nWWW-Authenticate: Digest realm=\"LIVE555 Streaming Media\", nonce=\"3b8d6b98cb67fb38af1cd3ae50ec393d\"\r\n\r\n"[..];

        let parsed = RtspResponse::parse(header);

        let expected = (
            &[][..],
            RtspResponse {
                status: RtpStatusCode::Unauthorized,
                headers: btreemap! {
                    "CSeq".to_string() => "2".to_string(),
                    "Date".to_string() => "Fri, Aug 09 2024 14:00:40 GMT".to_string(),
                    "WWW-Authenticate".to_string() => "Digest realm=\"LIVE555 Streaming Media\", nonce=\"3b8d6b98cb67fb38af1cd3ae50ec393d\"".to_string()
                },
            },
        );

        assert_eq!(parsed, Ok(expected));
    }
}
