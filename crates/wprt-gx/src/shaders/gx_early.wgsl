// Appended to gx.wgsl when the adapter supports Features::SHADER_EARLY_DEPTH_TEST (Vulkan).
// [earlydepthstencil] variant (D3D11 early_pixel_shader).
@fragment
@early_depth_test(force)
fn pixel_main_early(p: PixelInput) -> PixelOutput {
    return pixel_body(p);
}

@fragment
@early_depth_test(force)
fn pixel_main_early_single(p: PixelInput) -> SingleOutput {
    var output: SingleOutput;
    output.color = pixel_body(p).color;
    return output;
}
