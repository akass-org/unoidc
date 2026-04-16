use axum::{
    extract::{ConnectInfo, Request},
    middleware::Next,
    response::Response,
};
use std::net::SocketAddr;

/// 在缺少 ConnectInfo 时注入默认地址，避免测试环境 extractor 失败
pub async fn ensure_connect_info_middleware(mut req: Request, next: Next) -> Response {
    if req.extensions().get::<ConnectInfo<SocketAddr>>().is_none() {
        req.extensions_mut()
            .insert(ConnectInfo(SocketAddr::from(([0, 0, 0, 0], 0))));
    }

    next.run(req).await
}
