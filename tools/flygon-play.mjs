// Actual game inputs come exclusively from the neural worker. Evidence stays outside Git.
import { chromium } from "playwright";
import fs from "node:fs/promises";
const output = process.env.FLYGON_EVIDENCE_DIR || "/tmp";
const seconds = Number(process.env.FLYGON_PLAY_SECONDS || 300);
if (!Number.isFinite(seconds) || seconds < 20 || seconds > 3600)
  throw Error("FLYGON_PLAY_SECONDS must be 20–3600");
await fs.mkdir(output, { recursive: true });
const browser = await chromium.launch({ headless: true, args: ["--use-gl=angle", "--use-angle=swiftshader", "--enable-unsafe-swiftshader"] });
const page = await browser.newPage({ viewport: { width: 1600, height: 1100 } });
const errors = [];
page.on("pageerror", (e) => errors.push(e.message));
await page.goto(process.env.FLYGON_URL || "http://127.0.0.1:33006/flygon");

await page.waitForFunction(
  () =>
    !document.querySelector("#run").disabled &&
    document.querySelector("#game").contentWindow.__flygonGameBridge,
  null,
  { timeout: 120000 },
);
await page.check("#play");
await page.click("#run");
const samples = [];
for (let i = 0; i < Math.ceil(seconds / 20); i++) {
  await page.waitForTimeout(20000);
  const sample = await page.evaluate(() =>
    Object.fromEntries(
      [
        "metrics",
        "circuits",
        "speed",
        "action",
        "game-state",
        "story-state",
        "phase",
        "events",
      ].map((id) => [id, document.getElementById(id).textContent]),
    ),
  );
  samples.push(sample);
  sample.observation = await page.evaluate(() => document.querySelector("#game").contentWindow.__flygonGameBridge.execute({ kind: "observe" }));
  console.log(JSON.stringify(sample));
  if (sample.phase.includes("ERROR")) break;
}
if ((await page.locator("#run").textContent()) === "Pause")
  await page.click("#run");
await page.waitForTimeout(1000);
await page.screenshot({
  path: `${output}/flygon-story-gameplay.png`,
  fullPage: true,
});
let observation;
try {
  observation = await page.evaluate(() =>
    document
      .querySelector("#game")
      .contentWindow.__flygonGameBridge.execute({ kind: "observe" }),
  );
} catch (e) {
  observation = { error: String(e) };
}
const download = page.waitForEvent("download");
await page.locator(".advanced > details > summary").click();
await page.click("#save-record");
await (await download).saveAs(`${output}/flygon-story-run-record.json`);
const renderedActivity = await page.evaluate(() => {
  const c = document.querySelector("#brain");
  const d = c.getContext("2d").getImageData(0, 0, c.width, c.height).data;
  let activePixels = 0,
    dopaminePixels = 0;
  for (let i = 0; i < d.length; i += 4) {
    if (d[i] === 110 && d[i + 1] === 255 && d[i + 2] === 188) activePixels++;
    if (d[i] === 255 && d[i + 1] === 115 && d[i + 2] === 191) dopaminePixels++;
  }
  return {
    activePixels,
    dopaminePixels,
    palette: "default view.json colors; custom palettes change these counts",
  };
});
await fs.writeFile(
  `${output}/flygon-story-gameplay.json`,
  JSON.stringify({ samples, observation, renderedActivity, errors }, null, 2),
);
console.log(
  "FINAL",
  JSON.stringify({
    map: observation.map_info?.name,
    player: observation.map_info?.player,
    errors,
  }),
);
await browser.close();

if (
  errors.length ||
  observation.error ||
  samples.some((s) => s.phase.includes("ERROR"))
)
  process.exitCode = 1;
