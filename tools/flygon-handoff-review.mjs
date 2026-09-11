// Browser validation with real manual takeover and real neural game inputs.
import { chromium } from "playwright";
import fs from "node:fs/promises";
const output =
  process.env.FLYGON_EVIDENCE_DIR || "target/flygon-evidence/story-release";
await fs.mkdir(output, { recursive: true });
const browser = await chromium.launch({ headless: false });
try {
  const page = await browser.newPage({
    viewport: { width: 1520, height: 1040 },
  });
  const errors = [];
  page.on("pageerror", (e) => errors.push(e.message));
  await page.goto(process.env.FLYGON_URL || "http://127.0.0.1:33100/flygon");
  await page.waitForFunction(
    () =>
      document.querySelector("#brain").dataset.frames &&
      document.querySelector("#game").contentWindow.__flygonGameBridge,
    null,
    { timeout: 120000 },
  );
  const observe = () =>
    page.evaluate(() =>
      document
        .querySelector("#game")
        .contentWindow.__flygonGameBridge.execute({ kind: "observe" }),
    );
  const initial = await observe();
  if (initial.reward_state?.version !== 1)
    throw Error("Missing story telemetry");
  await page
    .frameLocator("#game")
    .locator("canvas")
    .click({ position: { x: 40, y: 40 } });
  await page.waitForFunction(() =>
    document.querySelector("#phase").textContent.includes("HUMAN CONTROL"),
  );
  if (await page.locator("#play").isChecked())
    throw Error("Human takeover did not release game");
  await page.click("#run");
  if (!(await page.locator("#play").isChecked()))
    throw Error("Run failed to restore game submissions");
  await page.waitForTimeout(12000);
  await page.click("#run");
  await page.waitForTimeout(3000);
  await page.click(".advanced > details > summary");
  const download = page.waitForEvent("download");
  await page.click("#save-record");
  const d = await download;
  const path = `${output}/handoff-record.json`;
  await d.saveAs(path);
  const record = JSON.parse(await fs.readFile(path, "utf8"));
  if (record.transport.neuralButtonSubmissions < 1)
    throw Error("No resumed neural submissions");
  const after = await observe();
  console.log(
    JSON.stringify({
      initialScreen: initial.status.screen,
      finalScreen: after.status.screen,
      telemetryVersion: after.reward_state.version,
      transport: record.transport,
      errors,
    }),
  );
  if (errors.length) throw Error(errors.join(";"));
  await page.screenshot({ path: `${output}/handoff.png` });
} finally {
  await browser.close();
}
