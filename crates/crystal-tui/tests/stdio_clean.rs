use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

#[test]
fn normal_mcp_gameplay_keeps_stderr_clean() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repository root");
    let pack = repo_root.join("content-packs/text-tui.crystalpack");
    if !pack.exists() {
        eprintln!(
            "skipping external-pack stderr regression; build {} first",
            pack.display()
        );
        return;
    }

    let mut child = Command::new(env!("CARGO_BIN_EXE_geothite"))
        .args(["mcp", pack.to_str().expect("UTF-8 pack path"), "--name", "CHRIS"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("start geothite MCP server");
    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/call",
        "params": {
            "name": "execute_macro",
            "arguments": {
                "actions": [
                    { "type": "move", "value": "right", "times": 4 },
                    { "type": "move", "value": "up", "times": 3 },
                    { "type": "move", "value": "down", "times": 4 }
                ]
            }
        }
    });
    writeln!(
        child.stdin.as_mut().expect("MCP stdin"),
        "{}",
        serde_json::to_string(&request).expect("serialize MCP request")
    )
    .expect("write MCP request");
    drop(child.stdin.take());

    let output = child.wait_with_output().expect("finish MCP server");
    assert!(output.status.success(), "MCP process failed: {output:?}");
    assert!(
        output.stderr.is_empty(),
        "normal gameplay polluted the terminal through stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("UTF-8 MCP output");
    assert!(stdout.contains("PlayersHouse1F"), "unexpected MCP output: {stdout}");
    assert!(stdout.contains("\"mode\":\"text\""), "Mom dialogue did not open: {stdout}");
}
