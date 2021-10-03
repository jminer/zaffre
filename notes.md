
There is a partial implemention of NV-path:

https://github.com/atilimcetin/path-rendering

However, I see in that code that when you add a cubic Bezier curve, it subdivides the curve into
eight cubic curves. Then it converts each to a quadratic curve and adds them to the path. However,
doing so is only an approximation, as quadratic curves cannot represent all the possible shapes
a cubic curve can:
http://stackoverflow.com/questions/13911963/cubic-bezier-to-quadratic

It should adaptively subdivide into as many quadratic curves as needed to so that the
approximation isn't off by too much.

Non-subdivide Bezier curve intersections:

https://github.com/thelonious/kld-intersections/blob/0f0aecea6923c0042c72f514fa914cac95a72686/lib/Intersection.js

Finding self-intersection of a cubic curve with a loop:

https://comp.graphics.algorithms.narkive.com/tqLNEZqM/cubic-bezier-self-intersections

The current plan for this library:

- Filling will use the approach Blinn and Loop did in their paper. Quadratic and cubic curves will
  be filled with no approximations.
- Stroking will draw quadratic curves exactly. However, cubic curves will be approximated with
  enough quadratic curves that they won't be further than X% of the stroke width off.

Update in 2019:

Compositing images is the main use case in a UI. DirectWrite can render text to a DirectX texture, then I can use a Vulkan extension to convert that to a Vulkan texture, and render it to a quad. Using sampler arrays, I can bind many textures and make one draw call. Nicol Bolas describes options here, but support for shaderSampledImageArrayDynamicIndexing is good now, so sampler arrays is probably the way to go: https://stackoverflow.com/questions/36772607/vulkan-texture-rendering-on-multiple-meshes

Do keep in mind the `maxDescriptorSetSampledImages` limit, which is as low as 96 on mobile.


https://www.nvidia.com/docs/IO/8228/BatchBatchBatch.pdf
https://www.reddit.com/r/vulkan/comments/48ixtp/some_initial_vulkan_vs_opengl_performance_tests/
https://www.reddit.com/r/opengl/comments/4u8qyv/opengl_limited_number_of_textures_how_can_you/d5nt752/

## Ash

In a GitHub issue, Ash developers said that you usually aren't supposed to call build() on struct builders. They implement Deref, so just pass them in place of the struct. Helps check that the lifetime of slices is long enough.

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

See answer to "How are sRGB formats and the sRGB color space related?" in Vulkan spec. An image with an SRGB format when read by a shader will apply the transfer function when converting the channel value to a float, and when writing it will apply the transfer function when converting the float to a channel value (to an 8-bit uint or whatever).

For covering extra pixels to do antialiasing with curve rendering, this extensions is perfect:

https://www.saschawillems.de/blog/2018/03/04/conservative-rasterization-in-vulkan-using-vk_ext_conservative_rasterization/

However, it is only supported for GeForce 9xx and newer, and AMD Radeon RX Vega and newer. Maybe I could just depend on multisample AA for previous cards, then eventually everything will support conservative rasterization and have perfect AA.

Don't rely on vkAcquireNextImageKHR to ever block:

https://www.reddit.com/r/vulkan/comments/b37762/command_queue_grows_indefinitely_on_intel_gpus/

vulkan-tutorial.com and many other places I found set the srcAlphaBlendFactor to ONE and dstAlphaBlendFactor to ZERO, but that just overwrites the alpha with the src value, which is wrong. Instead, it should be a mix like (1 - (1 - srcAlpha) * (1 - dstAlpha)) but a blend factor called that doesn't exist:

https://stackoverflow.com/questions/37532428/opengl-default-pipeline-alpha-blending-does-not-make-any-sense-for-the-alpha-com?rq=1

## Vulkan pipeline derivatives

https://stackoverflow.com/a/59312390/69671

## Vulkan descriptor set types

I think I finally understand the difference between the different image descriptor types, although I can't really find this stuff stated explicitly anywhere.

- `VK_DESCRIPTOR_TYPE_SAMPLER` is used to pass a sampler object to a shader. It isn't useful alone. The only way to use the sampler object in the shader is by using it with a sampled image object passed in another variable.
- `VK_DESCRIPTOR_TYPE_SAMPLED_IMAGE` is used to pass a sampled image object to a shader. It is to be used with a sampler passed in another variable. You can pass a couple samplers and one sampled image to sample the same image multiple ways. Or pass a couple sampled images and one sampler to sample multiple images the same way.
- `VK_DESCRIPTOR_TYPE_COMBINED_IMAGE_SAMPLER` is used to pass a sampler and a sampled image to a shader together in one variable. It might be faster than passing them separately.
- `VK_DESCRIPTOR_TYPE_STORAGE_IMAGE` is used to pass an image object to a shader when you don't want to use a sampler on it and just want to read the texels.

## Vulkan formats PACK16/PACK32

https://stackoverflow.com/questions/36098103/what-does-pack8-16-32-mean-in-vkformat-names

## Vulkan formats suffixes

https://docs.rs/vulkano/0.6.2/vulkano/format/index.html

## Vulkan transfer queue operations

https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/VkPipelineStageFlagBits.html

> `VK_PIPELINE_STAGE_TRANSFER_BIT` specifies the following commands:
>
> - All copy commands, including `vkCmdCopyQueryPoolResults`
> - `vkCmdBlitImage`
> - `vkCmdResolveImage`
> - All clear commands, with the exception of `vkCmdClearAttachments`

## Vulkan TILING_LINEAR staging images

https://www.reddit.com/r/vulkan/comments/71k4gy/why_is_vk_image_tiling_linear_so_limited/dnchgcp/

## Vulkan GLSL

https://github.com/KhronosGroup/GLSL/blob/master/extensions/khr/GL_KHR_vulkan_glsl.txt

## sRGB to Linear Conversion and vice versa

The Vulkan spec references the "sRGB EOTF" section in this spec:

https://www.khronos.org/registry/DataFormat/specs/1.2/dataformat.1.2.html

It is in section 13.3.

## Bicubic interpolation

https://stackoverflow.com/questions/13501081/efficient-bicubic-filtering-code-in-glsl

However, I don't think this is bicubic interpolation, just some bicubic filter. See the example in this answer:

https://stackoverflow.com/a/42179924/69671

The bicubic result is not interpolated, as the original colors are lost. You get a different result if you scale up the image using bicubic interpolation in Gimp. Nicol Bolas's comment on that question is correct in that it is impossible to do bicubic interpolation with 4 bilinear samples. You have to actually read 16 texels and interpolate them in the shader.

