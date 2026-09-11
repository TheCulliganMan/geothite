// Independent presentation worker: every projection and pixel is computed in Rust.
import init, { AnatomyView } from "./flygon/crystal_flygon.js";
let view;
self.onmessage = async ({ data }) => {
  const { id, kind } = data;
  try {
    let result;
    if (kind === "load") {
      await init();
      view?.free();
      view = new AnatomyView(data.records);
      result = { ready: true, somas: data.records.length / 5 };
    } else if (!view) throw Error("Anatomy is not loaded");
    else if (kind === "activity") {
      view.update_activity(data.indices);
      result = { updated: true };
    } else if (kind === "render") {
      const started = performance.now();
      view.animate(data.activityAge, data.motion);
      const pixels = view.render(
        data.width,
        data.height,
        data.yaw,
        data.pitch,
        data.zoom,
        data.group,
        JSON.stringify(data.style),
      );
      self.postMessage(
        {
          id,
          result: {
            pixels,
            width: data.width,
            height: data.height,
            wall_ms: performance.now() - started,
          },
        },
        [pixels.buffer],
      );
      return;
    } else if (kind === "select") {
      view.select(data.index, JSON.stringify(data.edges));
      result = { selected: data.index };
    } else if (kind === "pick") {
      result = {
        index: view.pick(
          data.width,
          data.height,
          data.x,
          data.y,
          data.yaw,
          data.pitch,
          data.zoom,
          data.group,
        ),
      };
    } else throw Error("Unknown view command");
    self.postMessage({ id, result });
  } catch (error) {
    self.postMessage({ id, error: String(error?.message || error) });
  }
};
