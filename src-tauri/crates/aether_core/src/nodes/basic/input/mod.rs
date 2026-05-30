use aether_types::{Node, NodeType, InputPin, OutputPin, Parameter, ParameterValue};
use crate::nodes::ExecutionContext;
use std::collections::HashMap;
use uuid::Uuid;
use log::debug;

#[derive(Debug, Clone, PartialEq)]
pub enum MediaType {
    Video,
    Image,
    Audio,
    Sequence,
}

mod video_decoder;
mod image_decoder;
mod audio_decoder;
mod sequence_loader;

pub use video_decoder::{VideoDecoder, VideoFrameMetadata};
pub use image_decoder::{ImageDecoder, DecodedImageFrame, RGBImageFrame};
pub use audio_decoder::{AudioDecoder, AudioMetadata};
pub use sequence_loader::SequenceLoader;


#[derive(Debug, Clone)]
pub struct InputNode {

    node: Node,

    media_type: MediaType,

    media_path: Option<String>,

    frame_cache: HashMap<u64, ParameterValue>,

    video_decoder: VideoDecoder,

    image_decoder: ImageDecoder,

    audio_decoder: AudioDecoder,

    sequence_loader: SequenceLoader,
}

impl InputNode {

    pub fn new(node: Node) -> Self {
        let media_type = node.node_type.clone().into();

        Self {
            node,
            media_type,
            media_path: None,
            frame_cache: HashMap::new(),
            video_decoder: VideoDecoder::new(),
            image_decoder: ImageDecoder::new(),
            audio_decoder: AudioDecoder::new(),
            sequence_loader: SequenceLoader::new(),
        }
    }


    pub fn create_standard(name: String) -> Node {
        let mut node = Node::new(NodeType::Input, name);


        let output_pin = OutputPin {
            id: Uuid::new_v4(),
            name: "output".to_string(),
            data_type: aether_types::PinDataType::Image,
            value: ParameterValue::None,
        };
        node.add_output(output_pin);


        let media_type_param = Parameter {
            id: Uuid::new_v4(),
            name: "media_type".to_string(),
            data_type: aether_types::PinDataType::String,
            value: ParameterValue::String("image".to_string()),
            default_value: ParameterValue::String("image".to_string()),
            min_value: None,
            max_value: None,
        };
        node.add_parameter(media_type_param);

        let media_path_param = Parameter {
            id: Uuid::new_v4(),
            name: "media_path".to_string(),
            data_type: aether_types::PinDataType::String,
            value: ParameterValue::None,
            default_value: ParameterValue::None,
            min_value: None,
            max_value: None,
        };
        node.add_parameter(media_path_param);

        let sequence_pattern_param = Parameter {
            id: Uuid::new_v4(),
            name: "sequence_pattern".to_string(),
            data_type: aether_types::PinDataType::String,
            value: ParameterValue::None,
            default_value: ParameterValue::None,
            min_value: None,
            max_value: None,
        };
        node.add_parameter(sequence_pattern_param);

        node
    }


    pub fn get_media_type(&self) -> MediaType {
        self.media_type.clone()
    }


    pub fn set_media_path(&mut self, path: String) {
        self.media_path = Some(path);
    }


    pub fn get_media_path(&self) -> Option<&String> {
        self.media_path.as_ref()
    }


    pub fn set_sequence_pattern(&mut self, pattern: String) {
        self.sequence_loader.set_sequence_pattern(pattern);
    }


    pub fn get_sequence_pattern(&self) -> Option<&String> {
        self.sequence_loader.get_sequence_pattern()
    }


    pub fn generate_frame(&mut self, frame: u64) -> ParameterValue {

        if let Some(cached_frame) = self.frame_cache.get(&frame) {
            return cached_frame.clone();
        }

        let result = match self.media_type {
            MediaType::Video => self.load_video_frame(frame),
            MediaType::Image => self.load_image_frame(frame),
            MediaType::Audio => self.load_audio_frame(frame),
            MediaType::Sequence => self.load_sequence_frame(frame),
        };


        self.frame_cache.insert(frame, result.clone());
        result
    }


    fn load_video_frame(&mut self, frame: u64) -> ParameterValue {
        if let Some(media_path) = &self.media_path {
            debug!("Loading video frame {} from file: {}", frame, media_path);

            let texture_id = self.video_decoder.decode_video_frame_with_ffmpeg(frame, media_path);
            ParameterValue::Image(texture_id)
        } else {
            debug!("No media path set for video input");
            ParameterValue::None
        }
    }


    fn load_image_frame(&mut self, _frame: u64) -> ParameterValue {
        if let Some(media_path) = &self.media_path {
            debug!("Loading image from file: {}", media_path);

            self.image_decoder.decode_image_with_ffmpeg(media_path)
        } else {
            debug!("No media path set for image input");
            ParameterValue::None
        }
    }


    fn load_audio_frame(&mut self, frame: u64) -> ParameterValue {
        if let Some(media_path) = &self.media_path {
            debug!("Loading audio frame {} from file: {}", frame, media_path);

            let audio_data = self.audio_decoder.decode_audio_frame_with_ffmpeg(frame, media_path);
            ParameterValue::Audio(audio_data)
        } else {
            debug!("No media path set for audio input");
            ParameterValue::None
        }
    }


    fn load_sequence_frame(&mut self, frame: u64) -> ParameterValue {
        if let Some(media_path) = &self.media_path {
            self.sequence_loader.load_sequence_frame(frame, media_path)
        } else {
            debug!("No media path set for sequence input");
            ParameterValue::None
        }
    }


    pub fn clear_cache(&mut self) {
        self.frame_cache.clear();
    }


    pub fn cache_size(&self) -> usize {
        self.frame_cache.len()
    }
}

impl From<NodeType> for MediaType {
    fn from(node_type: NodeType) -> Self {
        match node_type {
            NodeType::Input => MediaType::Image,
            _ => MediaType::Image,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_node_creation() {
        let node = InputNode::create_standard("Test Input".to_string());
        let input_node = InputNode::new(node);

        assert_eq!(input_node.get_media_type(), MediaType::Image);
        assert_eq!(input_node.cache_size(), 0);
    }

    #[test]
    fn test_media_path_setting() {
        let node = InputNode::create_standard("Test".to_string());
        let mut input_node = InputNode::new(node);

        input_node.set_media_path("/path/to/media.mp4".to_string());
        assert_eq!(input_node.get_media_path(), Some(&"/path/to/media.mp4".to_string()));
    }

    #[test]
    fn test_sequence_pattern_setting() {
        let node = InputNode::create_standard("Test".to_string());
        let mut input_node = InputNode::new(node);

        input_node.set_sequence_pattern("output_%04d.png".to_string());
        assert_eq!(input_node.get_sequence_pattern(), Some(&"output_%04d.png".to_string()));
    }

    #[test]
    fn test_cache_operations() {
        let node = InputNode::create_standard("Test".to_string());
        let mut input_node = InputNode::new(node);

        assert_eq!(input_node.cache_size(), 0);

        input_node.clear_cache();
        assert_eq!(input_node.cache_size(), 0);
    }
}
