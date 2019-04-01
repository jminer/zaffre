
There is a partial implemention of NV-path:

https://github.com/atilimcetin/path-rendering

However, I see in that code that when you add a cubic Bezier curve, it subdivides the curve into
eight cubic curves. Then it converts each to a quadratic curve and adds them to the path. However,
doing so is only an approximation, as quadratic curves cannot represent all the possible shapes
a cubic curve can:
http://stackoverflow.com/questions/13911963/cubic-bezier-to-quadratic

It should adaptively subdivide into as many quadratic curves as needed to so that the
approximation isn't off by too much.

The current plan for this library:

- Filling will use the approach Blinn and Loop did in their paper. Quadratic and cubic curves will
  be filled with no approximations.
- Stroking will draw quadratic curves exactly. However, cubic curves will be approximated with
  enough quadratic curves that they won't be further than X% of the stroke width off.

Update in 2019:

Compositing images is the main use case in a UI. DirectWrite can render text to a DirectX texture, then I can use a Vulkan extension to convert that to a Vulkan texture, and render it to a quad. Using sampler arrays, I can bind many textures and make one draw call. Nicol Bolas describes options here, but support for shaderSampledImageArrayDynamicIndexing is good now, so sampler arrays is probably the way to go: https://stackoverflow.com/questions/36772607/vulkan-texture-rendering-on-multiple-meshes


https://www.nvidia.com/docs/IO/8228/BatchBatchBatch.pdf
https://www.reddit.com/r/vulkan/comments/48ixtp/some_initial_vulkan_vs_opengl_performance_tests/
https://www.reddit.com/r/opengl/comments/4u8qyv/opengl_limited_number_of_textures_how_can_you/d5nt752/


## Vulkan

Really really useful when working with Vulkan because it shows what hardware supports:

https://vulkan.gpuinfo.org/

https://computergraphics.stackexchange.com/questions/7504/vulkan-how-does-host-coherence-work
https://gpuopen.com/vulkan-device-memory/
https://devtalk.nvidia.com/default/topic/1038172/vulkan/vulkan-memoryheaps-and-their-memorytypes/

AMD has a (on the Radeon R9 380) third memory heap that is 256MB and is DEVICE_LOCAL (stored in GPU RAM) but also HOST_VISIBLE (accessible from CPU).

https://www.reddit.com/r/vulkan/comments/7wwrs5/how_to_get_image_from_swapchain_for_screenshot/

About queue families and why it is useful to use a transfer queue for transfers instead of a graphics queue:

https://stackoverflow.com/questions/55272626/what-is-actually-a-queue-family-in-vulkan

See answer to "How are sRGB formats and the sRGB color space related?" in Vulkan spec.

For covering extra pixels to do antialiasing with curve rendering, this extensions is perfect:

https://www.saschawillems.de/blog/2018/03/04/conservative-rasterization-in-vulkan-using-vk_ext_conservative_rasterization/

However, it is only supported for GeForce 9xx and newer, and AMD Radeon RX Vega and newer. Maybe I could just depend on multisample AA for previous cards, then eventually everything will support conservative rasterization and have perfect AA.

Don't rely on vkAcquireNextImageKHR to ever block:

https://www.reddit.com/r/vulkan/comments/b37762/command_queue_grows_indefinitely_on_intel_gpus/
