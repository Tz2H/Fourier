//! 仅监听本机的网页入口，静态资源编译进二进制，无需前端构建工具。

mod document;
mod playback;
mod server;

pub use server::run;
