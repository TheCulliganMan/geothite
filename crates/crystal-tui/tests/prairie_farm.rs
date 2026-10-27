//! Stdio uses the same controller with an explicitly exported first-badge journey fixture.
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

#[test]
#[ignore = "requires the external Prairie Farm pack and exported controller-fixture save"]
fn mcp_continues_prairie_farm_moves_and_opens_production_start_without_stderr() {
    let pack =
        PathBuf::from(std::env::var_os("CRYSTAL_PRAIRIE_FARM_TEST_PACK").expect("external pack"));
    let save = PathBuf::from(
        std::env::var_os("CRYSTAL_PRAIRIE_FARM_JOURNEY_SAVE")
            .expect("export the explicit first-badge controller journey"),
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
        .expect("production MCP");
    for (id, (name, arguments)) in [
        ("status", serde_json::json!({})),
        ("move", serde_json::json!({"direction":"down","steps":1})),
        ("press", serde_json::json!({"button":"start"})),
        ("observe", serde_json::json!({"detail":"compact"})),
        ("press", serde_json::json!({"button":"b"})),
    ]
    .into_iter()
    .enumerate()
    {
        let request = serde_json::json!({"jsonrpc":"2.0","id":id,"method":"tools/call","params":{"name":name,"arguments":arguments}});
        writeln!(child.stdin.as_mut().expect("stdin"), "{request}").expect("request");
    }
    drop(child.stdin.take());
    let output = child.wait_with_output().expect("MCP completion");
    assert!(output.status.success(), "MCP failed: {output:?}");
    assert!(
        output.stderr.is_empty(),
        "stderr must remain empty: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let replies = String::from_utf8(output.stdout)
        .expect("UTF-8")
        .lines()
        .map(|line| serde_json::from_str::<serde_json::Value>(line).expect("JSON-RPC"))
        .collect::<Vec<_>>();
    assert_eq!(replies.len(), 5);
    for response in &replies {
        assert!(response.get("error").is_none());
        assert_eq!(response["result"]["isError"], false, "{response}");
    }
    let state = |index: usize| &replies[index]["result"]["structuredContent"];
    assert_eq!(state(0)["map"], "MnR02C00R00");
    assert_eq!(state(0)["position"]["x"], 54);
    assert_eq!(state(0)["position"]["y"], 14);
    assert_eq!(state(1)["position"]["x"], 54);
    assert_eq!(state(1)["position"]["y"], 15);
    assert!(state(3)["menu"].to_string().contains("PACK"));
    assert!(state(4)["menu"].is_null());
}
