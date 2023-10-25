use lsp_server::*;

use super::context::*;
use crate::fmt::FormatConfig;
use lsp_types::*;

/// Handles go-to-def request of the language server.
pub fn on_fmt_request(context: &Context, request: &Request) {
    let parameters = serde_json::from_value::<DocumentFormattingParams>(request.params.clone())
        .expect("could not deserialize go-to-def request");
    let fpath = parameters.text_document.uri.to_file_path().unwrap();
    let send_err = |context: &Context, msg: String| {
        let r = Response::new_err(request.id.clone(), ErrorCode::UnknownErrorCode as i32, msg);
        context
            .connection
            .sender
            .send(Message::Response(r))
            .unwrap();
    };
    let content = std::fs::read_to_string(fpath.as_path()).unwrap();
    let fmt = match super::fmt::format(content.as_str(), FormatConfig { indent_size: 4 }) {
        Ok(x) => x,
        Err(_err) => {
            // TODO handle _err.
            send_err(context, "format not ok,maybe parse error".to_string());
            return;
        }
    };
    use std::fs::write;
    write(fpath.as_path(), fmt.as_bytes()).unwrap();
    let r = Response::new_ok(
        request.id.clone(),
        serde_json::to_value(None as Option<Vec<TextEdit>>).unwrap(),
    );
    context
        .connection
        .sender
        .send(Message::Response(r))
        .unwrap();
}
