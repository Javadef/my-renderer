// Graphics pipeline creation and management
//
// The graphics pipeline defines how vertices are processed and rasterized.
// It includes: vertex input, shaders, rasterization, depth/stencil, blending.

use anyhow::{Context, Result};
use ash::vk;
use super::VulkanDevice;

/// Create a render pass for basic color attachment rendering with depth
pub fn create_render_pass(device: &VulkanDevice, format: vk::Format) -> Result<vk::RenderPass> {
    // Color attachment (the swapchain image)
    let color_attachment = vk::AttachmentDescription::builder()
        .format(format)
        .samples(vk::SampleCountFlags::TYPE_1)
        .load_op(vk::AttachmentLoadOp::CLEAR)
        .store_op(vk::AttachmentStoreOp::STORE)
        .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
        .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .final_layout(vk::ImageLayout::PRESENT_SRC_KHR)
        .build();
    
    // Depth attachment
    let depth_attachment = vk::AttachmentDescription::builder()
        .format(vk::Format::D32_SFLOAT)
        .samples(vk::SampleCountFlags::TYPE_1)
        .load_op(vk::AttachmentLoadOp::CLEAR)
        .store_op(vk::AttachmentStoreOp::DONT_CARE) // Don't need to store depth
        .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
        .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .final_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
        .build();
    
    // Reference to color attachment
    let color_attachment_ref = vk::AttachmentReference::builder()
        .attachment(0)
        .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
        .build();
    
    // Reference to depth attachment
    let depth_attachment_ref = vk::AttachmentReference::builder()
        .attachment(1)
        .layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
        .build();
    
    // Single subpass with color and depth
    let color_attachments = &[color_attachment_ref];
    let subpass = vk::SubpassDescription::builder()
        .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
        .color_attachments(color_attachments)
        .depth_stencil_attachment(&depth_attachment_ref)
        .build();
    
    // Subpass dependency
    let dependency = vk::SubpassDependency::builder()
        .src_subpass(vk::SUBPASS_EXTERNAL)
        .dst_subpass(0)
        .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS)
        .src_access_mask(vk::AccessFlags::empty())
        .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS)
        .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE)
        .build();
    
    let attachments = &[color_attachment, depth_attachment];
    let subpasses = &[subpass];
    let dependencies = &[dependency];
    
    let render_pass_info = vk::RenderPassCreateInfo::builder()
        .attachments(attachments)
        .subpasses(subpasses)
        .dependencies(dependencies);
    
    unsafe {
        device.device.create_render_pass(&render_pass_info, None)
            .context("Failed to create render pass")
    }
}

/// Create framebuffers for each swapchain image (with depth attachment)
pub fn create_framebuffers(
    device: &VulkanDevice,
    image_views: &[vk::ImageView],
    depth_image_view: vk::ImageView,
    render_pass: vk::RenderPass,
    extent: vk::Extent2D,
) -> Result<Vec<vk::Framebuffer>> {
    image_views.iter().map(|&image_view| {
        let attachments = &[image_view, depth_image_view];
        let framebuffer_info = vk::FramebufferCreateInfo::builder()
            .render_pass(render_pass)
            .attachments(attachments)
            .width(extent.width)
            .height(extent.height)
            .layers(1);
        
        unsafe {
            device.device.create_framebuffer(&framebuffer_info, None)
                .context("Failed to create framebuffer")
        }
    }).collect()
}

/// Vertex input description for our cube vertices (position + color)
pub fn get_vertex_input_info() -> (
    Vec<vk::VertexInputBindingDescription>,
    Vec<vk::VertexInputAttributeDescription>,
) {
    // One binding for interleaved position + normal + color data
    let binding = vk::VertexInputBindingDescription::builder()
        .binding(0)
        .stride((9 * std::mem::size_of::<f32>()) as u32) // 3 pos + 3 normal + 3 color
        .input_rate(vk::VertexInputRate::VERTEX)
        .build();
    
    // Position attribute (location 0)
    let position_attr = vk::VertexInputAttributeDescription::builder()
        .binding(0)
        .location(0)
        .format(vk::Format::R32G32B32_SFLOAT)
        .offset(0)
        .build();
    
    // Normal attribute (location 1)
    let normal_attr = vk::VertexInputAttributeDescription::builder()
        .binding(0)
        .location(1)
        .format(vk::Format::R32G32B32_SFLOAT)
        .offset(12) // After 3 floats
        .build();
    
    // Color attribute (location 2)
    let color_attr = vk::VertexInputAttributeDescription::builder()
        .binding(0)
        .location(2)
        .format(vk::Format::R32G32B32_SFLOAT)
        .offset(24) // After 6 floats
        .build();
    
    (vec![binding], vec![position_attr, normal_attr, color_attr])
}

/// Create a basic graphics pipeline for rendering the cube
pub fn create_graphics_pipeline(
    device: &VulkanDevice,
    render_pass: vk::RenderPass,
    extent: vk::Extent2D,
    vert_shader: vk::ShaderModule,
    frag_shader: vk::ShaderModule,
) -> Result<(vk::Pipeline, vk::PipelineLayout)> {
    // Shader stages
    let entry_point = std::ffi::CString::new("main").unwrap();
    
    let vert_stage = vk::PipelineShaderStageCreateInfo::builder()
        .stage(vk::ShaderStageFlags::VERTEX)
        .module(vert_shader)
        .name(&entry_point)
        .build();
    
    let frag_stage = vk::PipelineShaderStageCreateInfo::builder()
        .stage(vk::ShaderStageFlags::FRAGMENT)
        .module(frag_shader)
        .name(&entry_point)
        .build();
    
    let shader_stages = &[vert_stage, frag_stage];
    
    // Vertex input
    let (bindings, attributes) = get_vertex_input_info();
    let vertex_input_info = vk::PipelineVertexInputStateCreateInfo::builder()
        .vertex_binding_descriptions(&bindings)
        .vertex_attribute_descriptions(&attributes);
    
    // Input assembly
    let input_assembly = vk::PipelineInputAssemblyStateCreateInfo::builder()
        .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
        .primitive_restart_enable(false);
    
    // Viewport and scissor
    let viewport = vk::Viewport::builder()
        .x(0.0)
        .y(0.0)
        .width(extent.width as f32)
        .height(extent.height as f32)
        .min_depth(0.0)
        .max_depth(1.0)
        .build();
    
    let scissor = vk::Rect2D::builder()
        .offset(vk::Offset2D { x: 0, y: 0 })
        .extent(extent)
        .build();
    
    let viewports = &[viewport];
    let scissors = &[scissor];
    let viewport_state = vk::PipelineViewportStateCreateInfo::builder()
        .viewports(viewports)
        .scissors(scissors);
    
    // Rasterization
    let rasterizer = vk::PipelineRasterizationStateCreateInfo::builder()
        .depth_clamp_enable(false)
        .rasterizer_discard_enable(false)
        .polygon_mode(vk::PolygonMode::FILL)
        .line_width(1.0)
        .cull_mode(vk::CullModeFlags::BACK)
        .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
        .depth_bias_enable(false);
    
    // Multisampling (disabled)
    let multisampling = vk::PipelineMultisampleStateCreateInfo::builder()
        .sample_shading_enable(false)
        .rasterization_samples(vk::SampleCountFlags::TYPE_1);
    
    // Depth testing - ESSENTIAL for correct 3D rendering!
    let depth_stencil = vk::PipelineDepthStencilStateCreateInfo::builder()
        .depth_test_enable(true)
        .depth_write_enable(true)
        .depth_compare_op(vk::CompareOp::LESS) // Closer objects win
        .depth_bounds_test_enable(false)
        .stencil_test_enable(false);
    
    // Color blending (no blending, opaque)
    let color_blend_attachment = vk::PipelineColorBlendAttachmentState::builder()
        .color_write_mask(vk::ColorComponentFlags::RGBA)
        .blend_enable(false)
        .build();
    
    let color_blend_attachments = &[color_blend_attachment];
    let color_blending = vk::PipelineColorBlendStateCreateInfo::builder()
        .logic_op_enable(false)
        .attachments(color_blend_attachments);
    
    // Push constant for MVP matrix and model matrix (128 bytes for 2x 4x4 matrices)
    let push_constant_range = vk::PushConstantRange::builder()
        .stage_flags(vk::ShaderStageFlags::VERTEX)
        .offset(0)
        .size(128)
        .build();
    
    let push_constant_ranges = &[push_constant_range];
    
    // Pipeline layout
    let layout_info = vk::PipelineLayoutCreateInfo::builder()
        .push_constant_ranges(push_constant_ranges);
    
    let pipeline_layout = unsafe {
        device.device.create_pipeline_layout(&layout_info, None)
            .context("Failed to create pipeline layout")?
    };
    
    // Create pipeline
    let pipeline_info = vk::GraphicsPipelineCreateInfo::builder()
        .stages(shader_stages)
        .vertex_input_state(&vertex_input_info)
        .input_assembly_state(&input_assembly)
        .viewport_state(&viewport_state)
        .rasterization_state(&rasterizer)
        .multisample_state(&multisampling)
        .depth_stencil_state(&depth_stencil)
        .color_blend_state(&color_blending)
        .layout(pipeline_layout)
        .render_pass(render_pass)
        .subpass(0)
        .build();
    
    let pipelines = unsafe {
        device.device.create_graphics_pipelines(
            vk::PipelineCache::null(),
            &[pipeline_info],
            None,
        ).map_err(|(_, e)| e)
            .context("Failed to create graphics pipeline")?
    };
    
    Ok((pipelines[0], pipeline_layout))
}
