//! The external town pack must use the same stdio controller path as normal play.
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

#[test]
fn mcp_plays_grand_marais_starter_exit_and_start_menu_without_stderr() {
    let pack = std::env::var_os("CRYSTAL_MINNESOTA_TEST_PACK")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../content-packs/minnesota-grand-marais.crystalpack")
        });
    assert!(pack.is_file(), "build the external Grand Marais pack first");
    let mut child = Command::new(env!("CARGO_BIN_EXE_geothite"))
        .arg("mcp")
        .arg(pack)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("start production MCP client");
    for (id, (name, arguments)) in [
        ("status", serde_json::json!({})),
        ("move", serde_json::json!({"direction":"down"})),
        ("observe", serde_json::json!({"detail":"compact"})),
        ("press", serde_json::json!({"button":"a","times":25})),
        ("move", serde_json::json!({"direction":"down","steps":2})),
        ("press", serde_json::json!({"button":"start"})),
        ("observe", serde_json::json!({"detail":"compact"})),
    ]
    .into_iter()
    .enumerate()
    {
        let request = serde_json::json!({"jsonrpc":"2.0","id":id,"method":"tools/call","params":{"name":name,"arguments":arguments}});
        writeln!(child.stdin.as_mut().expect("MCP stdin"), "{request}").expect("write request");
    }
    drop(child.stdin.take());
    let output = child.wait_with_output().expect("finish MCP client");
    assert!(output.status.success(), "MCP failed: {output:?}");
    assert!(
        output.stderr.is_empty(),
        "gameplay polluted stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let responses = String::from_utf8(output.stdout)
        .expect("MCP UTF-8")
        .lines()
        .map(|line| serde_json::from_str::<serde_json::Value>(line).expect("JSON-RPC response"))
        .collect::<Vec<_>>();
    assert_eq!(responses.len(), 7);
    for response in &responses {
        assert!(response.get("error").is_none(), "RPC error: {response}");
        assert_eq!(response["result"]["isError"], false);
    }
    let state = |index: usize| &responses[index]["result"]["structuredContent"];
    assert_eq!(state(0)["map"], "MnGrandMaraisFieldStation");
    assert_eq!(state(1)["position"]["y"], 6);
    assert_eq!(state(1)["dialogue"], true);
    assert!(state(2)["dialogue"].to_string().contains("TAMARACK"));
    assert_eq!(state(3)["pokedex"]["owned"], 1);
    assert_eq!(state(4)["map"], "MnGrandMarais");
    assert!(state(5)["menu"].is_string());
    assert!(state(6)["menu"].to_string().contains("PACK"));
}
