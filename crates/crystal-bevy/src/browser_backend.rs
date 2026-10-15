//! A modern bundle must never use Bevy's compile-time-incompatible GL layout.
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
pub async fn probe() -> bool {
    let force_gl = web_sys::window().and_then(|window| window.location().search().ok())
        .and_then(|search| web_sys::UrlSearchParams::new_with_str(&search).ok())
        .and_then(|params| params.get("renderer")).as_deref() == Some("webgl");
    if force_gl { return false; }
    let available = async {
        let navigator = js_sys::Reflect::get(&js_sys::global(), &JsValue::from_str("navigator")).ok()?;
        let gpu = js_sys::Reflect::get(&navigator, &JsValue::from_str("gpu")).ok()?;
        if gpu.is_null() || gpu.is_undefined() { return None; }
        let request = js_sys::Reflect::get(&gpu, &JsValue::from_str("requestAdapter")).ok()?
            .dyn_into::<js_sys::Function>().ok()?;
        let promise = request.call0(&gpu).ok()?.dyn_into::<js_sys::Promise>().ok()?;
        let adapter = JsFuture::from(promise).await.ok()?;
        (!adapter.is_null() && !adapter.is_undefined()).then_some(())
    }.await.is_some();
    available
}
/// Navigate before constructing the game if adapter availability changed since
/// the loader's probe. Reloading also releases the unused modern WASM instance.
pub fn fallback_to_webgl() -> Result<(), JsValue> {
    let window = web_sys::window().ok_or_else(|| JsValue::from_str("browser window is unavailable"))?;
    let location = window.location();
    let params = web_sys::UrlSearchParams::new_with_str(&location.search()?)?;
    params.set("renderer", "webgl");
    let search: String = params.to_string().into();
    location.replace(&format!("{}{}?{}{}", location.origin()?, location.pathname()?, search, location.hash()?))
}
pub(crate) fn settings() -> bevy::render::settings::WgpuSettings {
    use bevy::render::settings::{Backends, WgpuSettings};
    let mut settings = WgpuSettings::default();
    settings.backends = Some(Backends::BROWSER_WEBGPU);
    settings
}
