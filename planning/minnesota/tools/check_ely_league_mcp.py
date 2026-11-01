#!/usr/bin/env python3
"""Replay actual controller-earned League saves through the production stdio client."""
import argparse
from pathlib import Path
import subprocess
import tempfile

from check_minneapolis_mcp import Client
from check_north_shore_story_mcp import choose, finish
from check_winona_mcp import travel, walk
from render_reference import load_pack

ROOT = Path(__file__).resolve().parents[3]


def register(client, pack):
    walk(client, pack, (5,4))
    client.press("up")
    client.press("a")
    choose(client, False)
    refusal = finish(client)
    assert "Healing and supplies" in refusal or "clinic is free" in refusal
    client.press("up")
    client.press("a")
    choose(client, True)
    finish(client)
    client.at("MnLeagueRoom1",4,16)


def play(client, pack, mode):
    if mode == "postgame":
        client.at("MnElyPokecenter1F",3,5)
        walk(client, pack, (3,3))
        client.press("up")
        client.press("a")
        finish(client)
        walk(client, pack, (3,6))
        travel(client,"down","MnEly",(11,22))
        walk(client,pack,(25,14))
        travel(client,"up","MnLeagueLobby",(1,4))
    else:
        client.at("MnLeagueRoom1",4,16)
        walk(client,pack,(5,16))
        travel(client,"down","MnLeagueLobby",(1,5))
    register(client,pack)
    walk(client,pack,(5,16))
    travel(client,"down","MnLeagueLobby",(1,5))
    walk(client,pack,(1,4))
    travel(client,"left","MnEly",(25,14))
    walk(client,pack,(24,0))
    travel(client,"up","MnR13C00R02",(34,31))
    client.press("start")
    assert "PACK" in client.observe()["content"][0]["text"]


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--pack",type=Path,default=ROOT/"content-packs/minnesota-northstar-ely-league.crystalpack")
    parser.add_argument("--client",type=Path,default=ROOT/"target/northstar-story/wasm-dev/geothite")
    parser.add_argument("--save",type=Path,required=True)
    parser.add_argument("--mode",choices=["registered","postgame"],required=True)
    parser.add_argument("--boosted-review",action="store_true",help="Label the user-authorized supplied-level continuation.")
    args=parser.parse_args()
    pack=load_pack(args.pack)
    with tempfile.TemporaryFile() as stderr:
        process=subprocess.Popen([str(args.client),"mcp",str(args.pack),"--load",str(args.save)],
                                 stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=stderr,text=True,bufsize=1)
        client=Client(process)
        try:
            client.rpc("initialize",dict(protocolVersion="2025-06-18",capabilities={},clientInfo=dict(name="ely-league-check",version="1")))
            before=client.status()
            play(client,pack,args.mode)
            after=client.status()
            assert (after["money"],after["badges"])==(before["money"],before["badges"])
        finally:
            process.stdin.close()
            assert process.wait(timeout=30)==0
        stderr.seek(0)
        assert stderr.read()==b"","gameplay stderr must remain byte-empty"
    scope="boosted story review" if args.boosted_review else "played"
    print(f"PASS: {scope} {args.mode} Continue, native registration decline/retry, run exit, Ely/R13 walking, Start, unchanged money/badges and byte-empty stderr.")


if __name__=="__main__":
    main()
