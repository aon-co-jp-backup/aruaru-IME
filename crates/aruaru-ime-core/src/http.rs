//! ureq 3 の薄い互換層。ureq 2 の `AgentBuilder`/`Request::set`/`into_json` 等の書き味を保ち、
//! 呼び出し側(約60箇所)を最小の変更で ureq 3 に載せるためのもの。
//! 本文の読み込み上限は ureq 2 と同じく呼び出し側の `take()` に任せる(JSON/文字列は 256 MiB まで)。
use std::io::Read;
use std::time::Duration;

pub use ureq::Error;

const BODY_LIMIT: u64 = 256 * 1024 * 1024;

#[derive(Default)]
pub struct AgentBuilder {
    timeout: Option<Duration>,
    connect: Option<Duration>,
    ua: Option<String>,
}

impl AgentBuilder {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn timeout(mut self, d: Duration) -> Self {
        self.timeout = Some(d);
        self
    }
    pub fn timeout_connect(mut self, d: Duration) -> Self {
        self.connect = Some(d);
        self
    }
    pub fn user_agent(mut self, ua: &str) -> Self {
        self.ua = Some(ua.to_string());
        self
    }
    pub fn build(self) -> Agent {
        let mut b = ureq::Agent::config_builder().timeout_global(self.timeout).timeout_connect(self.connect);
        if let Some(ua) = self.ua {
            b = b.user_agent(ua);
        }
        Agent(ureq::Agent::new_with_config(b.build()))
    }
}

#[derive(Clone)]
pub struct Agent(ureq::Agent);

impl Agent {
    pub fn get(&self, url: &str) -> Request {
        Request::new(&self.0, false, url)
    }
    pub fn post(&self, url: &str) -> Request {
        Request::new(&self.0, true, url)
    }
    pub fn put(&self, url: &str) -> Request {
        let mut r = Request::new(&self.0, true, url);
        r.put = true;
        r
    }
}

pub struct Request {
    agent: ureq::Agent,
    post: bool,
    put: bool,
    url: String,
    headers: Vec<(String, String)>,
    queries: Vec<(String, String)>,
    timeout: Option<Duration>,
}

impl Request {
    fn new(agent: &ureq::Agent, post: bool, url: &str) -> Self {
        Request { agent: agent.clone(), post, put: false, url: url.to_string(), headers: vec![], queries: vec![], timeout: None }
    }
    pub fn query(mut self, k: &str, v: &str) -> Self {
        self.queries.push((k.to_string(), v.to_string()));
        self
    }
    pub fn set(mut self, k: &str, v: &str) -> Self {
        self.headers.push((k.to_string(), v.to_string()));
        self
    }
    pub fn timeout(mut self, d: Duration) -> Self {
        self.timeout = Some(d);
        self
    }
    pub fn call(self) -> Result<Response, Error> {
        if self.post {
            return self.send_bytes(&[]);
        }
        let mut r = self.agent.get(&self.url);
        for (k, v) in &self.headers {
            r = r.header(k, v);
        }
        for (k, v) in &self.queries {
            r = r.query(k, v);
        }
        if let Some(t) = self.timeout {
            r = r.config().timeout_global(Some(t)).build();
        }
        r.call().map(Response)
    }
    fn post_builder(&self) -> ureq::RequestBuilder<ureq::typestate::WithBody> {
        let mut r = if self.put { self.agent.put(&self.url) } else { self.agent.post(&self.url) };
        for (k, v) in &self.headers {
            r = r.header(k, v);
        }
        for (k, v) in &self.queries {
            r = r.query(k, v);
        }
        if let Some(t) = self.timeout {
            r = r.config().timeout_global(Some(t)).build();
        }
        r
    }
    fn send_bytes(self, b: &[u8]) -> Result<Response, Error> {
        self.post_builder().send(b).map(Response)
    }
    pub fn send_json(self, body: impl serde::Serialize) -> Result<Response, Error> {
        self.post_builder().send_json(body).map(Response)
    }
    pub fn send_string(self, body: &str) -> Result<Response, Error> {
        self.post_builder().send(body).map(Response)
    }
    pub fn send_form(self, form: &[(&str, &str)]) -> Result<Response, Error> {
        self.post_builder().send_form(form.iter().copied()).map(Response)
    }
}

pub struct Response(ureq::http::Response<ureq::Body>);

impl Response {
    pub fn status(&self) -> u16 {
        self.0.status().as_u16()
    }
    pub fn header(&self, name: &str) -> Option<&str> {
        self.0.headers().get(name).and_then(|v| v.to_str().ok())
    }
    pub fn into_json<T: serde::de::DeserializeOwned>(mut self) -> std::io::Result<T> {
        self.0.body_mut().with_config().limit(BODY_LIMIT).read_json().map_err(std::io::Error::other)
    }
    pub fn into_string(mut self) -> std::io::Result<String> {
        self.0.body_mut().with_config().limit(BODY_LIMIT).read_to_string().map_err(std::io::Error::other)
    }
    pub fn into_reader(self) -> impl Read + Send {
        self.0.into_body().into_reader()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::net::TcpListener;

    fn serve(resp: &'static str) -> String {
        let l = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = l.local_addr().unwrap();
        std::thread::spawn(move || {
            if let Ok((mut s, _)) = l.accept() {
                let mut buf = [0u8; 4096];
                let _ = s.read(&mut buf);
                let _ = s.write_all(resp.as_bytes());
            }
        });
        format!("http://{addr}/x")
    }

    #[test]
    fn get_json_and_header() {
        let url = serve("HTTP/1.1 200 OK\r\nETag: abc\r\nContent-Type: application/json\r\nContent-Length: 8\r\nConnection: close\r\n\r\n{\"a\":42}");
        let r = AgentBuilder::new().timeout(Duration::from_secs(5)).user_agent("t").build().get(&url).call().unwrap();
        assert_eq!(r.status(), 200);
        assert_eq!(r.header("ETag"), Some("abc"));
        let v: serde_json::Value = r.into_json().unwrap();
        assert_eq!(v["a"], 42);
    }

    #[test]
    fn status_error_maps_to_statuscode() {
        let url = serve("HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n");
        let e = AgentBuilder::new().timeout(Duration::from_secs(5)).build().get(&url).call().err().unwrap();
        assert!(matches!(e, Error::StatusCode(404)));
    }
}
