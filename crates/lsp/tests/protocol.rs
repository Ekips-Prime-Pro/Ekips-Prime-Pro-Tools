//! Talks to the server like an editor would, over an in-memory connection.

use std::time::Duration;

use lsp_server::{Connection, Message, Notification, Request, RequestId};
use serde_json::{Value, json};

struct Client {
    conn: Connection,
    next_id: i32,
    server: Option<std::thread::JoinHandle<()>>,
}

impl Client {
    fn start(init_options: Value) -> Client {
        let (server_conn, client_conn) = Connection::memory();
        let server = std::thread::spawn(move || {
            ekips_lsp::run(&server_conn, ekips_lsp::Config::default()).expect("server failed");
        });
        let mut client = Client {
            conn: client_conn,
            next_id: 0,
            server: Some(server),
        };
        let result = client.request(
            "initialize",
            json!({
                "capabilities": {
                    "textDocument": { "completion": { "completionItem": { "snippetSupport": true } } }
                },
                "initializationOptions": init_options,
            }),
        );
        assert_eq!(result["serverInfo"]["name"], "ekips-lsp");
        assert_eq!(result["capabilities"]["positionEncoding"], "utf-16");
        client.notify("initialized", json!({}));
        client
    }

    fn notify(&self, method: &str, params: Value) {
        let note = Notification::new(method.to_string(), params);
        self.conn.sender.send(note.into()).unwrap();
    }

    fn recv(&self) -> Message {
        self.conn
            .receiver
            .recv_timeout(Duration::from_secs(10))
            .expect("no message from server")
    }

    fn request(&mut self, method: &str, params: Value) -> Value {
        self.next_id += 1;
        let id = RequestId::from(self.next_id);
        let req = Request::new(id.clone(), method.to_string(), params);
        self.conn.sender.send(req.into()).unwrap();
        loop {
            if let Message::Response(resp) = self.recv() {
                assert_eq!(resp.id, id);
                assert!(resp.error.is_none(), "{:?}", resp.error);
                return resp.result.unwrap_or(Value::Null);
            }
        }
    }

    fn diagnostics(&self) -> Value {
        loop {
            if let Message::Notification(n) = self.recv() {
                if n.method == "textDocument/publishDiagnostics" {
                    return n.params;
                }
            }
        }
    }

    fn shutdown(mut self) {
        self.request("shutdown", Value::Null);
        self.notify("exit", Value::Null);
        self.server.take().unwrap().join().unwrap();
    }
}

const URI: &str = "file:///mission.ekips";

#[test]
fn diagnostics_follow_every_change() {
    let mut client = Client::start(json!({ "lang": "de" }));
    client.notify(
        "textDocument/didOpen",
        json!({ "textDocument": {
            "uri": URI, "languageId": "ekips", "version": 1,
            "text": "cosnt x = 1;\nasync function main() {}\n"
        }}),
    );
    let diags = client.diagnostics();
    assert_eq!(diags["version"], 1);
    let list = diags["diagnostics"].as_array().unwrap();
    assert_eq!(list.len(), 1);
    assert!(list[0]["message"].as_str().unwrap().contains("Meintest du"));

    client.notify(
        "textDocument/didChange",
        json!({
            "textDocument": { "uri": URI, "version": 2 },
            "contentChanges": [{ "text": "const x = 1;\nasync function main() {}\n" }]
        }),
    );
    let diags = client.diagnostics();
    assert_eq!(diags["version"], 2);
    assert!(diags["diagnostics"].as_array().unwrap().is_empty());

    // Hover and completion on the current text.
    let hover = client.request(
        "textDocument/hover",
        json!({ "textDocument": { "uri": URI }, "position": { "line": 0, "character": 1 } }),
    );
    assert!(
        hover["contents"]["value"]
            .as_str()
            .unwrap()
            .contains("sich nie ändert"),
        "{hover}"
    );
    let completion = client.request(
        "textDocument/completion",
        json!({ "textDocument": { "uri": URI }, "position": { "line": 1, "character": 23 } }),
    );
    let labels: Vec<&str> = completion
        .as_array()
        .unwrap()
        .iter()
        .map(|i| i["label"].as_str().unwrap())
        .collect();
    assert!(labels.contains(&"while"));
    assert!(labels.contains(&"drivebase"));

    client.notify(
        "textDocument/didClose",
        json!({ "textDocument": { "uri": URI } }),
    );
    assert!(
        client.diagnostics()["diagnostics"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    client.shutdown();
}

#[test]
fn hover_on_unknown_document_is_null() {
    let mut client = Client::start(json!({ "lang": "en" }));
    let hover = client.request(
        "textDocument/hover",
        json!({ "textDocument": { "uri": "file:///nope.ekips" }, "position": { "line": 0, "character": 0 } }),
    );
    assert!(hover.is_null());
    client.shutdown();
}
