//! Naga normalization for WGSL handle aliases accepted by native backends.
//! Browser WGSL requires texture/sampler references directly in expressions.
pub(crate) fn normalize(source: &str) -> Result<String, String> {
    let mut module = naga::front::wgsl::parse_str(source).map_err(|e| e.emit_to_string(source))?;
    let info = naga::valid::Validator::new(naga::valid::ValidationFlags::all(),
        naga::valid::Capabilities::all()).validate(&module).map_err(|e| e.to_string())?;
    fn remove_aliases(function: &mut naga::Function, info: &naga::valid::FunctionInfo,
        types: &naga::UniqueArena<naga::Type>) {
        function.named_expressions.retain(|handle, _|
            !matches!(info[*handle].ty.inner_with(types), naga::TypeInner::Image { .. } | naga::TypeInner::Sampler { .. }));
    }
    for (handle, function) in module.functions.iter_mut() {
        remove_aliases(function, &info[handle], &module.types);
    }
    for (index, entry) in module.entry_points.iter_mut().enumerate() {
        remove_aliases(&mut entry.function, info.get_entry_point(index), &module.types);
    }
    let output = naga::back::wgsl::write_string(&module, &info, naga::back::wgsl::WriterFlags::empty())
        .map_err(|e| e.to_string())?;
    // Material tags are constant over a primitive. Native backends permit the
    // existing per-material grain derivatives, while Tint cannot prove that
    // coherence from interpolated tag coordinates. Preserve that material
    // behavior without relaxing diagnostics for sprites or other shaders.
    if source.contains("var water_mask:") {
        Ok(format!("diagnostic(off, derivative_uniformity);\n{output}"))
    } else { Ok(output) }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn removes_resource_lets_and_preserves_numeric_code_and_entry_points() {
        let source = "@group(0) @binding(0) var color_texture: texture_2d<f32>;\n@group(0) @binding(1) var color_sampler: sampler;\n@fragment fn fragment() -> @location(0) vec4<f32> { let texture = color_texture; let sampler_ = color_sampler; let uv = vec2(0.5); let size = vec2<f32>(textureDimensions(texture)); return textureSample(texture, sampler_, uv) + textureSample(texture, sampler_, uv) / size.x; }";
        let result = normalize(source).unwrap();
        assert!(!result.contains("let texture"));
        assert!(!result.contains("let sampler"));
        assert!(result.contains("fn fragment("));
        assert!(result.contains("textureSample(color_texture, color_sampler"));
        let module = naga::front::wgsl::parse_str(&result).unwrap();
        naga::valid::Validator::new(naga::valid::ValidationFlags::all(),
            naga::valid::Capabilities::all()).validate(&module).unwrap();
    }
}
