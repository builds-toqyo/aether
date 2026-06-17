use crate::nodes::{NodeExecutor, ExecutionContext, NodeResult, NodeError};
use aether_types::{Node, NodeType};
use uuid::Uuid;

pub struct GpuComputeNode {
    pub node: Node,
    pub shader_source: String,
    pub compute_pipeline: Option<wgpu::ComputePipeline>,
    pub bind_group_layout: Option<wgpu::BindGroupLayout>,
}

impl GpuComputeNode {
    pub fn new(node: Node, shader_source: String) -> Self {
        Self {
            node,
            shader_source,
            compute_pipeline: None,
            bind_group_layout: None,
        }
    }

    fn ensure_pipeline_initialized(&mut self, context: &ExecutionContext) -> NodeResult<()> {
        if self.compute_pipeline.is_some() {
            return Ok(());
        }

        let gpu_context = context.gpu_context
            .as_ref()
            .ok_or_else(|| NodeError::ExecutionFailed("GPU context not available".to_string()))?;

        let device = gpu_context.device
            .as_ref()
            .ok_or_else(|| NodeError::ExecutionFailed("GPU device not available".to_string()))?;

        let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(&format!("shader_{}", self.node.id)),
            source: wgpu::ShaderSource::Wgsl(self.shader_source.clone().into()),
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some(&format!("bind_group_layout_{}", self.node.id)),
            entries: &[],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some(&format!("pipeline_layout_{}", self.node.id)),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });

        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some(&format!("compute_pipeline_{}", self.node.id)),
            layout: Some(&pipeline_layout),
            module: &shader_module,
            entry_point: Some("main"),
            cache: None,
            compilation_options: Default::default(),
        });

        self.compute_pipeline = Some(compute_pipeline);
        self.bind_group_layout = Some(bind_group_layout);

        Ok(())
    }
}

impl NodeExecutor for GpuComputeNode {
    fn execute(&mut self, context: &mut ExecutionContext) -> NodeResult<()> {
        log::debug!("Executing GPU compute node: {}", self.node.name);

        self.ensure_pipeline_initialized(context)?;

        let gpu_context = context.gpu_context
            .as_ref()
            .ok_or_else(|| NodeError::ExecutionFailed("GPU context not available".to_string()))?;

        let device = gpu_context.device
            .as_ref()
            .ok_or_else(|| NodeError::ExecutionFailed("GPU device not available".to_string()))?;

        let queue = gpu_context.queue
            .as_ref()
            .ok_or_else(|| NodeError::ExecutionFailed("GPU queue not available".to_string()))?;

        let pipeline = self.compute_pipeline
            .as_ref()
            .ok_or_else(|| NodeError::ExecutionFailed("Pipeline not initialized".to_string()))?;

        let bind_group_layout = self.bind_group_layout
            .as_ref()
            .ok_or_else(|| NodeError::ExecutionFailed("Bind group layout not initialized".to_string()))?;

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(&format!("bind_group_{}", self.node.id)),
            layout: bind_group_layout,
            entries: &[],
        });

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some(&format!("compute_encoder_{}", self.node.id)),
        });

        let workgroup_count = (
            (context.resolution.0 + 15) / 16,
            (context.resolution.1 + 15) / 16,
            1,
        );

        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some(&format!("compute_pass_{}", self.node.id)),
                timestamp_writes: None,
            });
            compute_pass.set_pipeline(pipeline);
            compute_pass.set_bind_group(0, &bind_group, &[]);
            compute_pass.dispatch_workgroups(workgroup_count.0, workgroup_count.1, workgroup_count.2);
        }

        queue.submit(Some(encoder.finish()));

        Ok(())
    }

    fn node_type(&self) -> NodeType {
        NodeType::Custom("gpu_compute".to_string())
    }

    fn get_inputs(&self) -> Vec<Uuid> {
        self.node.inputs.iter().map(|pin| pin.id).collect()
    }

    fn get_outputs(&self) -> Vec<Uuid> {
        self.node.outputs.iter().map(|pin| pin.id).collect()
    }
}
