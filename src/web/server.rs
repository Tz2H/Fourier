use std::{
    io::{self, Read},
    time::Duration,
};

use serde_json::json;
use tiny_http::{Header, Method, Request, Response, Server};

use super::document;

const MAX_UPLOAD: usize = 32 * 1024 * 1024;

pub fn run() -> io::Result<()> {
    let port = std::env::var("FOURIER_PORT").unwrap_or_else(|_| "8787".into());
    let port: u16 = port.parse().ok().filter(|port| *port != 0).ok_or_else(|| {
        io::Error::new(io::ErrorKind::InvalidInput, "FOURIER_PORT 必须为 1–65535")
    })?;

    let address = format!("127.0.0.1:{port}");
    let server = Server::http(&address).map_err(io::Error::other)?;
    println!(
        "\n  Fourier Studio\n  打开 http://{address}\n  按 Ctrl+C 停止服务。图片仅在本机内存中处理。\n"
    );

    // 单个计算队列避免多次大图上传同时占用内存。
    loop {
        if let Some(request) = server.recv_timeout(Duration::from_secs(1))? {
            handle(request, &address);
        }
    }
}

fn handle(mut request: Request, address: &str) {
    let url = request.url().to_owned();
    let (path, query) = url.split_once('?').unwrap_or((&url, ""));
    let detail = query
        .split('&')
        .find_map(|part| part.strip_prefix("detail="))
        .unwrap_or("balanced");

    if request.method() == &Method::Get {
        let resource: Option<(&str, &[u8])> = match path {
            "/" => Some((
                "text/html; charset=utf-8",
                include_bytes!("../../web/index.html"),
            )),
            "/style.css" => Some((
                "text/css; charset=utf-8",
                include_bytes!("../../web/style.css"),
            )),
            "/app.js" => Some((
                "text/javascript; charset=utf-8",
                include_bytes!("../../web/app.js"),
            )),
            "/viewport.js" => Some((
                "text/javascript; charset=utf-8",
                include_bytes!("../../web/viewport.js"),
            )),
            "/playback.js" => Some((
                "text/javascript; charset=utf-8",
                include_bytes!("../../web/playback.js"),
            )),
            "/sample.jpeg" => Some(("image/jpeg", include_bytes!("../../images.jpeg"))),
            _ => None,
        };
        if let Some((content_type, bytes)) = resource {
            respond(request, 200, content_type, bytes.to_vec());
            return;
        }
    }

    let result = match (request.method(), path) {
        (&Method::Get, "/api/sample") => {
            document::process(include_bytes!("../../images.jpeg"), detail)
        }
        (&Method::Post, "/api/image") => {
            if let Some(origin) = request.headers().iter().find(|h| h.field.equiv("Origin")) {
                let port = address.rsplit(':').next().unwrap();
                if origin.value.as_str() != format!("http://{address}")
                    && origin.value.as_str() != format!("http://localhost:{port}")
                {
                    respond_json(request, 403, json!({"error": "不允许跨站上传。"}));
                    return;
                }
            }

            if request
                .body_length()
                .is_some_and(|length| length > MAX_UPLOAD)
            {
                respond_json(request, 413, json!({"error": "图片不能超过 32 MB。"}));
                return;
            }

            let mut bytes = Vec::new();
            match request
                .as_reader()
                .take((MAX_UPLOAD + 1) as u64)
                .read_to_end(&mut bytes)
            {
                Ok(_) if bytes.len() <= MAX_UPLOAD => document::process(&bytes, detail),
                Ok(_) => Err("图片不能超过 32 MB。".into()),
                Err(_) => Err("图片上传中断，请重试。".into()),
            }
        }
        _ => {
            respond_json(request, 404, json!({"error": "页面不存在。"}));
            return;
        }
    };

    match result {
        Ok(document) => respond_json(request, 200, document),
        Err(error) => respond_json(request, 422, json!({"error": error})),
    }
}

fn respond_json(request: Request, status: u16, value: serde_json::Value) {
    respond(
        request,
        status,
        "application/json; charset=utf-8",
        value.to_string().into_bytes(),
    );
}

fn respond(request: Request, status: u16, content_type: &str, body: Vec<u8>) {
    let mut response = Response::from_data(body).with_status_code(status);
    for (name, value) in [
        ("Content-Type", content_type),
        ("Cache-Control", "no-store"),
        ("X-Content-Type-Options", "nosniff"),
        (
            "Content-Security-Policy",
            "default-src 'self'; script-src 'self'; style-src 'self'; img-src 'self' blob:; connect-src 'self'; frame-ancestors 'none'",
        ),
    ] {
        response.add_header(Header::from_bytes(name, value).expect("constant HTTP header"));
    }

    if let Err(error) = request.respond(response) {
        eprintln!("响应发送中断：{error}");
    }
}
