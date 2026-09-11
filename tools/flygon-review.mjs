// Reproducible render review of the assembled distribution. No compilation,
// synthetic neurons, direct game inputs, or neural computation in this harness.
import * as playwright from "playwright";
import fs from "node:fs/promises";
const engine = process.env.FLYGON_BROWSER || "chromium";
if (!["chromium", "firefox", "webkit"].includes(engine))
  throw Error("Unknown browser");
const output =
  process.env.FLYGON_EVIDENCE_DIR || "target/flygon-evidence/release";
await fs.mkdir(output, { recursive: true });
const browser = await playwright[engine].launch({
  headless: process.env.FLYGON_HEADLESS === "1",
});
try {
  const context = await browser.newContext({
    viewport: { width: 1520, height: 1040 },
    hasTouch: true,
  });
  const page = await context.newPage(),
    errors = [],
    failures = [];
  page.on("pageerror", (e) => {
    errors.push(e.message);
    console.error(e.message);
  });
  page.on("response", (r) => {
    if (r.status() >= 400) failures.push({ url: r.url(), status: r.status() });
  });
  page.setDefaultTimeout(30000);
  await page.goto(
    process.env.FLYGON_URL || "http://127.0.0.1:33008/flygon",
  );
  if ((await page.title()) !== "Flygon")
    throw Error("Unexpected release page or obsolete tagline");

  await page.waitForFunction(
    () =>
      document.getElementById("brain").dataset.frames ||
      document.getElementById("status-detail").textContent,
    null,
    { timeout: 60000 },
  );
  const loadError = await page.locator("#status-detail").textContent();
  if (loadError) throw Error(loadError);
  await page.waitForFunction(
    () => document.getElementById("game").contentWindow.__flygonGameBridge,
    null,
    { timeout: 120000 },
  );
  console.log("Brain and game ready");
  await page.uncheck("#play");
  await page.click("#step");
  await page.waitForFunction(() =>
    document.getElementById("action").textContent.startsWith("Neural output: "),
  );
  await page.waitForTimeout(500);
  await page.click(".advanced > details > summary");
  await page.fill("#cell-query", "MBON01");
  await page.click("#find");
  await page.waitForFunction(() =>
    document.getElementById("inspection").textContent.trim().startsWith("["),
  );
  console.log("Neural decision rendered");
  const source = await page.evaluate(
    () => JSON.parse(document.getElementById("inspection").textContent)[0],
  );
  await page.fill("#neuron", String(source.index));
  await page.click("#inspect");
  await page.waitForFunction(
    () => !document.getElementById("selection").hidden,
  );
  const neighborhood = await page.evaluate(() =>
    JSON.parse(document.getElementById("inspection").textContent),
  );
  if (
    !neighborhood.edges.length ||
    neighborhood.edges.some(
      (e) => e.source !== source.index && e.target !== source.index,
    )
  )
    throw Error("Unexpected connection neighborhood");
  await page.click(".advanced > details > summary");
  await page.locator("#brain").scrollIntoViewIfNeeded();
  const snapshot = () =>
    page.evaluate(() => {
      const c = document.getElementById("brain"),
        pixels = c.getContext("2d").getImageData(0, 0, c.width, c.height).data;
      let hash = 2166136261;
      for (let i = 0; i < pixels.length; i += 16)
        hash = Math.imul(hash ^ pixels[i], 16777619);
      return {
        hash: hash >>> 0,
        ...c.dataset,
        metrics: document.getElementById("metrics").textContent,
      };
    });
  console.log("Connections selected");
  const before = await snapshot();
  const box = await page.locator("#brain").boundingBox();
  await page.mouse.move(box.x + box.width * 0.6, box.y + box.height * 0.55);
  await page.mouse.down();
  await page.mouse.move(box.x + box.width * 0.8, box.y + box.height * 0.64, {
    steps: 24,
  });
  await page.mouse.up();
  await page.waitForFunction(
    (n) => Number(document.getElementById("brain").dataset.frames) > Number(n),
    before.frames,
  );
  await page.locator("#brain").focus();
  await page.keyboard.press("+");
  await page.keyboard.press("ArrowRight");
  await page.waitForTimeout(200);
  const after = await snapshot();
  if (before.hash === after.hash || before.metrics !== after.metrics)
    throw Error("View failed to rotate independently of neural state");
  await page.screenshot({
    path: `${output}/${engine}-desktop.png`,
    fullPage: true,
  });
  await page.setViewportSize({ width: 390, height: 844 });
  await page.locator("#brain").scrollIntoViewIfNeeded();
  await page.waitForTimeout(200);
  const mobile = await page.evaluate(() => ({
    width: innerWidth,
    scrollWidth: document.documentElement.scrollWidth,
  }));
  if (mobile.scrollWidth > mobile.width)
    throw Error("Mobile horizontal overflow");
  await page.screenshot({
    path: `${output}/${engine}-mobile.png`,
    fullPage: true,
  });
  console.log("Desktop and mobile rendered");
  if (engine === "chromium") {
    const cdp = await context.newCDPSession(page),
      bounds = await page.locator("#brain").boundingBox();
    const x = bounds.x + bounds.width / 2,
      y = bounds.y + bounds.height / 2;
    const oldYaw = await page.locator("#brain").getAttribute("data-yaw");
    await cdp.send("Input.dispatchTouchEvent", {
      type: "touchStart",
      touchPoints: [{ x, y }],
    });
    await cdp.send("Input.dispatchTouchEvent", {
      type: "touchMove",
      touchPoints: [{ x: x + 45, y: y + 20 }],
    });
    await cdp.send("Input.dispatchTouchEvent", {
      type: "touchEnd",
      touchPoints: [],
    });
    await page.waitForFunction(
      (yaw) => document.getElementById("brain").dataset.yaw !== yaw,
      oldYaw,
    );
    const oldZoom = await page.locator("#view-scale").textContent();
    for (const [type, spread] of [
      ["touchStart", 20],
      ["touchMove", 30],
      ["touchMove", 55],
    ]) {
      await cdp.send("Input.dispatchTouchEvent", {
        type,
        touchPoints: [
          { x: x - spread, y },
          { x: x + spread, y },
        ],
      });
    }
    await cdp.send("Input.dispatchTouchEvent", {
      type: "touchEnd",
      touchPoints: [],
    });
    await page.waitForFunction(
      (z) => document.getElementById("view-scale").textContent !== z,
      oldZoom,
    );
  }
  const result = {
    engine,
    source,
    connectionCount: neighborhood.edges.length,
    before,
    after,
    mobile,
    errors,
    failures,
  };
  await fs.writeFile(
    `${output}/${engine}-review.json`,
    JSON.stringify(result, null, 2),
  );
  console.log(JSON.stringify(result));
  if (errors.length || failures.length)
    throw Error("Browser errors or failed asset requests");
} finally {
  await browser.close();
}
