//! Opt-in stdio replay of the actual production-controller Safari naming save.

use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

#[test]
#[ignore = "requires enabled CRYSTAL_SAFARI_TEST_PACK and reached CRYSTAL_SAFARI_NAME_SAVE"]
fn mcp_resumes_safari_naming_and_finishes_pc_capture_without_stderr() {
    let pack = PathBuf::from(
        std::env::var_os("CRYSTAL_SAFARI_TEST_PACK").expect("enabled Safari review pack"),
    );
    let save = PathBuf::from(
        std::env::var_os("CRYSTAL_SAFARI_NAME_SAVE")
            .expect("production-controller pc-name-draft save"),
    );
    let mut child = Command::new(env!("CARGO_BIN_EXE_geothite"))
        .arg("mcp")
        .arg(pack)
        .arg("--load")
        .arg(save)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("production stdio MCP");
    for (id, (name, arguments)) in [
        ("observe", serde_json::json!({})),
        ("press", serde_json::json!({"button":"b"})),
        ("observe", serde_json::json!({})),
        ("press", serde_json::json!({"button":"a"})),
        ("press", serde_json::json!({"button":"start"})),
        ("press", serde_json::json!({"button":"a"})),
        ("observe", serde_json::json!({})),
        ("press", serde_json::json!({"button":"a"})),
        ("status", serde_json::json!({})),
    ]
    .into_iter()
    .enumerate()
    {
        let request = serde_json::json!({"jsonrpc":"2.0","id":id,"method":"tools/call","params":{"name":name,"arguments":arguments}});
        writeln!(child.stdin.as_mut().expect("MCP input"), "{request}").expect("request");
    }
    drop(child.stdin.take());
    let output = child.wait_with_output().expect("MCP completion");
    assert!(output.status.success(), "MCP failed: {output:?}");
    assert!(
        output.stderr.is_empty(),
        "Safari presentation polluted stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let replies = String::from_utf8(output.stdout)
        .expect("UTF8 stdio")
        .lines()
        .map(|line| serde_json::from_str::<serde_json::Value>(line).expect("JSON RPC response"))
        .collect::<Vec<_>>();
    assert_eq!(replies.len(), 9);
    for response in &replies {
        assert!(response.get("error").is_none(), "{response}");
        assert_eq!(response["result"]["isError"], false, "{response}");
    }
    let state = |index: usize| &replies[index]["result"]["structuredContent"];
    assert_eq!(state(0)["viewport_title"], "NICKNAME");
    assert_eq!(state(0)["viewport"][0]["text"], "Ab");
    assert_eq!(
        state(2)["viewport"][0]["text"],
        "A",
        "B edits the real native name draft"
    );
    assert!(state(6)["dialogue"].to_string().contains("Stored in BOX3."));
    assert!(
        state(8)["map"]
            .as_str()
            .expect("native shore map")
            .starts_with("MnBoundarySafari")
    );
    assert!(state(8)["menu"].is_null());
}
