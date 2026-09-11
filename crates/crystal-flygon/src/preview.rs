//! Loopback-only release preview and source iteration. Production uses the
//! existing crystal-web-server; this utility has no game or neural computation.
use std::{
    collections::BTreeMap,
    fs,
    io::{self, BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
    path::{Component, Path, PathBuf},
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
struct Preview {
    root: PathBuf,
    data: PathBuf,
    sources: BTreeMap<String, PathBuf>,
    dev: bool,
}
pub fn serve(root: &Path, data: Option<&Path>, workspace: Option<&Path>, port: u16) -> Result<()> {
    let root = root.canonicalize()?;
    let data = data.map_or_else(|| Ok(root.join("flygon-data")), Path::canonicalize)?;
    let mut sources = BTreeMap::new();
    if let Some(repo) = workspace {
        for name in super::UI {
            sources.insert(format!("/{name}"), repo.join("web-client").join(name));
        }
        for name in super::PROFILES
            .iter()
            .copied()
            .chain(["view", "dataset", "manifest"])
        {
            sources.insert(
                format!("/flygon-{name}.json"),
                repo.join(format!("modpacks/flygon/{name}.json")),
            );
        }
    }
    let listener = TcpListener::bind(("127.0.0.1", port))?;
    let state = Arc::new(Preview {
        root,
        data,
        sources,
        dev: workspace.is_some(),
    });
    println!(
        "Flygon http://127.0.0.1:{}/flygon.html · real local UTC clock · {}",
        listener.local_addr()?.port(),
        if state.dev {
            "live source UI"
        } else {
            "packaged release"
        }
    );
    for stream in listener.incoming() {
        let state = state.clone();
        let stream = stream?;
        std::thread::spawn(move || {
            if let Err(e) = handle(stream, &state) {
                eprintln!("Preview request: {e}");
            }
        });
    }
    Ok(())
}
fn reply(
    stream: &mut TcpStream,
    status: &str,
    mime: &str,
    body: &[u8],
    head: bool,
) -> io::Result<()> {
    write!(
        stream,
        "HTTP/1.1 {status}\r\nContent-Type: {mime}\r\nContent-Length: {}\r\nCache-Control: no-store\r\nConnection: close\r\n\r\n",
        body.len()
    )?;
    if !head {
        stream.write_all(body)?;
    }
    Ok(())
}
fn handle(mut stream: TcpStream, state: &Preview) -> io::Result<()> {
    stream.set_read_timeout(Some(Duration::from_secs(10)))?;
    stream.set_write_timeout(Some(Duration::from_secs(60)))?;
    let mut reader = BufReader::new(stream.try_clone()?);
    let mut request = String::new();
    reader.read_line(&mut request)?;
    if request.len() > 8192 {
        return reply(
            &mut stream,
            "400 Bad Request",
            "text/plain",
            b"Request too large",
            false,
        );
    }
    let mut headers = 0;
    loop {
        let mut line = String::new();
        reader.read_line(&mut line)?;
        headers += line.len();
        if headers > 16384 {
            return reply(
                &mut stream,
                "400 Bad Request",
                "text/plain",
                b"Headers too large",
                false,
            );
        }
        if line == "\r\n" || line.is_empty() {
            break;
        }
    }
    let parts: Vec<_> = request.split_whitespace().collect();
    if parts.len() != 3 {
        return reply(
            &mut stream,
            "400 Bad Request",
            "text/plain",
            b"Invalid request",
            false,
        );
    }
    let head = parts[0] == "HEAD";
    if !head && parts[0] != "GET" {
        return reply(
            &mut stream,
            "405 Method Not Allowed",
            "text/plain",
            b"Read only",
            false,
        );
    }
    let url = parts[1].split('?').next().unwrap_or("/");
    if url == "/v1/clock" {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(io::Error::other)?
            .as_millis();
        return reply(
            &mut stream,
            "200 OK",
            "application/json",
            format!(
                "{{\"unixMillis\":{now},\"timeZone\":\"UTC\",\"authority\":\"local-development\"}}"
            )
            .as_bytes(),
            head,
        );
    }
    if url == "/flygon-dev.json" {
        return reply(
            &mut stream,
            "200 OK",
            "application/json",
            format!("{{\"local_clock\":true,\"live_view\":{}}}", state.dev).as_bytes(),
            head,
        );
    }
    if url == "/flygon-dev-events" && state.dev {
        write!(
            stream,
            "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nCache-Control: no-store\r\nConnection: close\r\n\r\n: connected\n\n"
        )?;
        let paths = [
            ("css", state.sources.get("/flygon.css").unwrap()),
            ("view", state.sources.get("/flygon-view.json").unwrap()),
        ];
        let mut stamps = [None, None];
        let mut tick = 0;
        loop {
            std::thread::sleep(Duration::from_millis(500));
            tick += 1;
            for (i, (kind, path)) in paths.iter().enumerate() {
                let stamp = fs::metadata(path).and_then(|m| m.modified()).ok();
                if stamp != stamps[i] {
                    stamps[i] = stamp;
                    write!(
                        stream,
                        "data: {{\"kind\":\"{kind}\",\"revision\":{tick}}}\n\n"
                    )?;
                }
            }
            if tick % 20 == 0 {
                write!(stream, ": alive\n\n")?;
            }
            stream.flush()?;
        }
    }
    let relative = if url == "/" {
        "index.html"
    } else {
        url.trim_start_matches('/')
    };
    if relative.contains('%')
        || relative.contains('\\')
        || Path::new(relative)
            .components()
            .any(|c| !matches!(c, Component::Normal(_)))
    {
        return reply(
            &mut stream,
            "400 Bad Request",
            "text/plain",
            b"Invalid path",
            head,
        );
    }
    let (file, base) = if let Some(file) = state.sources.get(url) {
        (file.clone(), file.parent().unwrap().to_path_buf())
    } else if let Some(name) = relative.strip_prefix("flygon-data/") {
        (state.data.join(name), state.data.clone())
    } else {
        (state.root.join(relative), state.root.clone())
    };
    let file = match file.canonicalize() {
        Ok(p) if p.starts_with(base.canonicalize()?) && p.is_file() => p,
        _ => {
            return reply(
                &mut stream,
                "404 Not Found",
                "text/plain",
                b"Asset not found",
                head,
            );
        }
    };
    let mime = match file.extension().and_then(|s| s.to_str()).unwrap_or("") {
        "html" => "text/html; charset=utf-8",
        "js" => "text/javascript",
        "css" => "text/css",
        "json" => "application/json",
        "wasm" => "application/wasm",
        "svg" => "image/svg+xml",
        _ => "application/octet-stream",
    };
    if relative == "index.html" && state.dev {
        let mut html = fs::read_to_string(&file)?;
        let anchor = "const bridge = createGameBridge(wasm);";
        if !html.contains("__flygonGameBridge") {
            html=html.replace(anchor,&format!("{anchor}\nif(new URLSearchParams(location.search).get('flygon')==='1')window.__flygonGameBridge=bridge;"));
        }
        if !html.contains("__flygonActivateAudio") {
            html=html.replace(anchor,&format!("{anchor}\nif(new URLSearchParams(location.search).get('flygon')==='1')window.__flygonActivateAudio=resumeAudio;"));
        }
        return reply(&mut stream, "200 OK", mime, html.as_bytes(), head);
    }
    let mut file = fs::File::open(file)?;
    let size = file.metadata()?.len();
    write!(
        stream,
        "HTTP/1.1 200 OK\r\nContent-Type: {mime}\r\nContent-Length: {size}\r\nCache-Control: no-cache\r\nConnection: close\r\n\r\n"
    )?;
    if !head {
        io::copy(&mut file, &mut stream)?;
    }
    Ok(())
}
