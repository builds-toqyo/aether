use serde::{Deserialize, Serialize};
use std::fmt;
use std::collections::HashMap;

use crate::shapes::primitives::{ShapePrimitive, Transform, BoundingBox, PathShape};
use crate::shapes::paths::Path;
use crate::shapes::boolean::{BooleanResult, BooleanOperation, AdvancedBoolean};


#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum LayerBlendMode {
    Normal,
    Multiply,
    Screen,
    Overlay,
    Darken,
    Lighten,
    ColorDodge,
    ColorBurn,
    HardLight,
    SoftLight,
    Difference,
    Exclusion,
}

impl fmt::Display for LayerBlendMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LayerBlendMode::Normal => write!(f, "Normal"),
            LayerBlendMode::Multiply => write!(f, "Multiply"),
            LayerBlendMode::Screen => write!(f, "Screen"),
            LayerBlendMode::Overlay => write!(f, "Overlay"),
            LayerBlendMode::Darken => write!(f, "Darken"),
            LayerBlendMode::Lighten => write!(f, "Lighten"),
            LayerBlendMode::ColorDodge => write!(f, "Color Dodge"),
            LayerBlendMode::ColorBurn => write!(f, "Color Burn"),
            LayerBlendMode::HardLight => write!(f, "Hard Light"),
            LayerBlendMode::SoftLight => write!(f, "Soft Light"),
            LayerBlendMode::Difference => write!(f, "Difference"),
            LayerBlendMode::Exclusion => write!(f, "Exclusion"),
        }
    }
}

/// Layer visibility state
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum LayerVisibility {
    Visible,
    Hidden,
    Locked,
}

impl Default for LayerVisibility {
    fn default() -> Self {
        LayerVisibility::Visible
    }
}

/// Shape layer
#[derive(Serialize)]
pub struct ShapeLayer { 
    pub id: String,
    pub name: String,
    #[serde(skip)]
    pub shape: Box<dyn ShapePrimitive>,
    pub transform: Transform,
    pub visibility: LayerVisibility,
    pub opacity: f64,
    pub blend_mode: LayerBlendMode,
    pub order: i32,
    pub selected: bool,
    pub locked: bool,
    pub metadata: HashMap<String, String>,
}

impl Clone for ShapeLayer {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            name: self.name.clone(),
            shape: self.shape.clone_box(),
            transform: self.transform.clone(),
            visibility: self.visibility.clone(),
            opacity: self.opacity,
            blend_mode: self.blend_mode.clone(),
            order: self.order,
            selected: self.selected,
            locked: self.locked,
            metadata: self.metadata.clone(),
        }
    }
}

impl std::fmt::Debug for ShapeLayer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ShapeLayer")
            .field("id", &self.id)
            .field("name", &self.name)
            .field("shape", &"<ShapePrimitive>")
            .field("transform", &self.transform)
            .field("visibility", &self.visibility)
            .field("opacity", &self.opacity)
            .field("blend_mode", &self.blend_mode)
            .field("order", &self.order)
            .field("selected", &self.selected)
            .field("locked", &self.locked)
            .field("metadata", &self.metadata)
            .finish()
    }
}

impl ShapeLayer {
    pub fn new<S: Into<String>>(
        id: S,
        name: S,
        shape: Box<dyn ShapePrimitive>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            shape,
            transform: Transform::identity(),
            visibility: LayerVisibility::Visible,
            opacity: 1.0,
            blend_mode: LayerBlendMode::Normal,
            order: 0,
            selected: false,
            locked: false,
            metadata: HashMap::new(),
        }
    }

    pub fn with_transform<S: Into<String>>(
        id: S,
        name: S,
        shape: Box<dyn ShapePrimitive>,
        transform: Transform,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            shape,
            transform,
            visibility: LayerVisibility::Visible,
            opacity: 1.0,
            blend_mode: LayerBlendMode::Normal,
            order: 0,
            selected: false,
            locked: false,
            metadata: HashMap::new(),
        }
    }

    pub fn transformed_shape(&self) -> Box<dyn ShapePrimitive> {
        let mut shape = self.shape.clone_box();
        shape.transform(&self.transform);
        shape
    }

    pub fn bounds(&self) -> BoundingBox {
        let transformed_shape = self.transformed_shape();
        transformed_shape.bounds()
    }

    pub fn path(&self) -> Path {
        let transformed_shape = self.transformed_shape();
        transformed_shape.to_path()
    }

    pub fn contains_point(&self, x: f64, y: f64) -> bool {
        if !self.is_visible() {
            return false;
        }

        let transformed_shape = self.transformed_shape();
        transformed_shape.contains_point(x, y)
    }

    pub fn area(&self) -> f64 {
        let transformed_shape = self.transformed_shape();
        transformed_shape.area()
    }

    pub fn perimeter(&self) -> f64 {
        let transformed_shape = self.transformed_shape();
        transformed_shape.perimeter()
    }

    pub fn is_visible(&self) -> bool {
        matches!(self.visibility, LayerVisibility::Visible) &&
        self.opacity > 0.0 &&
        !self.locked
    }

    pub fn set_visibility(&mut self, visibility: LayerVisibility) {
        self.visibility = visibility;
    }

    pub fn set_opacity(&mut self, opacity: f64) {
        self.opacity = opacity.clamp(0.0, 1.0);
    }

    pub fn set_blend_mode(&mut self, blend_mode: LayerBlendMode) {
        self.blend_mode = blend_mode;
    }

    pub fn set_transform(&mut self, transform: Transform) {
        self.transform = transform;
    }

    pub fn apply_transform(&mut self, transform: &Transform) {
        self.transform = self.transform.combine(transform);
    }

    pub fn select(&mut self) {
        self.selected = true;
    }

    pub fn deselect(&mut self) {
        self.selected = false;
    }

    pub fn lock(&mut self) {
        self.locked = true;
        self.deselect();
    }

    pub fn unlock(&mut self) {
        self.locked = false;
    }

    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }

    pub fn set_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }

    pub fn remove_metadata(&mut self, key: &str) -> Option<String> {
        self.metadata.remove(key)
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.id.is_empty() {
            return Err("Layer ID cannot be empty".to_string());
        }

        if self.name.is_empty() {
            return Err("Layer name cannot be empty".to_string());
        }

        if self.opacity < 0.0 || self.opacity > 1.0 {
            return Err("Layer opacity must be between 0.0 and 1.0".to_string());
        }

        self.shape.validate()
    }

    pub fn clone_with_id<S: Into<String>>(&self, new_id: S) -> Self {
        let mut clone = self.clone();
        clone.id = new_id.into();
        clone.selected = false;
        clone
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShapeLayerCollection {
    #[serde(skip)]
    pub layers: Vec<ShapeLayer>,
    pub name: String,
    pub global_transform: Transform,
    pub metadata: HashMap<String, String>,
}

impl ShapeLayerCollection {
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self {
            layers: Vec::new(),
            name: name.into(),
            global_transform: Transform::identity(),
            metadata: HashMap::new(),
        }
    }

    pub fn add_layer(&mut self, layer: ShapeLayer) -> Result<(), String> {
        layer.validate()?;

        if self.layers.iter().any(|l| l.id == layer.id) {
            return Err(format!("Layer with ID {} already exists", layer.id));
        }

        self.layers.push(layer);
        self.sort_layers_by_order();
        Ok(())
    }

    pub fn remove_layer(&mut self, id: &str) -> Option<ShapeLayer> {
        let index = self.layers.iter().position(|l| l.id == id)?;
        Some(self.layers.remove(index))
    }

    pub fn get_layer(&self, id: &str) -> Option<&ShapeLayer> {
        self.layers.iter().find(|l| l.id == id)
    }

    pub fn get_layer_mut(&mut self, id: &str) -> Option<&mut ShapeLayer> {
        self.layers.iter_mut().find(|l| l.id == id)
    }

    pub fn get_layer_at(&self, index: usize) -> Option<&ShapeLayer> {
        self.layers.get(index)
    }

    pub fn get_layer_at_mut(&mut self, index: usize) -> Option<&mut ShapeLayer> {
        self.layers.get_mut(index)
    }

    pub fn find_layer_index(&self, id: &str) -> Option<usize> {
        self.layers.iter().position(|l| l.id == id)
    }

    /// Move layer to new position
    pub fn move_layer(&mut self, id: &str, new_order: i32) -> Result<(), String> {
        let index = self.find_layer_index(id)
            .ok_or_else(|| format!("Layer with ID {} not found", id))?;

        let mut layer = self.layers.remove(index);
        layer.order = new_order;

        self.layers.push(layer);
        self.sort_layers_by_order();
        Ok(())
    }

    /// Move layer up in stack
    pub fn move_layer_up(&mut self, id: &str) -> Result<(), String> {
        let index = self.find_layer_index(id)
            .ok_or_else(|| format!("Layer with ID {} not found", id))?;

        if index == 0 {
            return Err("Layer is already at the top of the stack".to_string());
        }

        self.layers.swap(index, index - 1);
        self.update_layer_orders();
        Ok(())
    }

    /// Move layer down in stack
    pub fn move_layer_down(&mut self, id: &str) -> Result<(), String> {
        let index = self.find_layer_index(id)
            .ok_or_else(|| format!("Layer with ID {} not found", id))?;

        if index == self.layers.len() - 1 {
            return Err("Layer is already at the bottom of the stack".to_string());
        }

        self.layers.swap(index, index + 1);
        self.update_layer_orders();
        Ok(())
    }

    /// Get visible layers
    pub fn get_visible_layers(&self) -> Vec<&ShapeLayer> {
        self.layers.iter()
            .filter(|l| l.is_visible())
            .collect()
    }

    /// Get selected layers
    pub fn get_selected_layers(&self) -> Vec<&ShapeLayer> {
        self.layers.iter()
            .filter(|l| l.selected)
            .collect()
    }

    /// Get layers at point
    pub fn get_layers_at_point(&self, x: f64, y: f64) -> Vec<&ShapeLayer> {
        self.layers.iter()
            .filter(|l| l.contains_point(x, y))
            .collect()
    }

    /// Select layer at point
    pub fn select_layer_at_point(&mut self, x: f64, y: f64) -> Option<&ShapeLayer> {
        // Deselect all layers first
        for layer in &mut self.layers {
            layer.deselect();
        }

        // Find topmost layer at point
        for layer in self.layers.iter_mut().rev() {
            if layer.contains_point(x, y) {
                layer.select();
                return Some(layer as &ShapeLayer);
            }
        }

        None
    }

    /// Select multiple layers
    pub fn select_layers(&mut self, ids: &[&str]) {
        // Deselect all layers first
        for layer in &mut self.layers {
            layer.deselect();
        }

        // Select specified layers
        for id in ids {
            if let Some(layer) = self.get_layer_mut(id) {
                layer.select();
            }
        }
    }

    /// Deselect all layers
    pub fn deselect_all(&mut self) {
        for layer in &mut self.layers {
            layer.deselect();
        }
    }

    /// Get collection bounds
    pub fn bounds(&self) -> BoundingBox {
        if self.layers.is_empty() {
            return BoundingBox::default();
        }

        let visible_layers = self.get_visible_layers();
        if visible_layers.is_empty() {
            return BoundingBox::default();
        }

        let mut bounds = visible_layers[0].bounds();

        for layer in visible_layers.iter().skip(1) {
            bounds = bounds.union(&layer.bounds());
        }

        bounds.transform(&self.global_transform)
    }

    /// Get combined path of all visible layers
    pub fn combined_path(&self) -> Path {
        let mut builder = crate::shapes::paths::PathBuilder::new();

        for layer in self.get_visible_layers() {
            let path = layer.path();

            // Add path segments
            for segment in &path.segments {
                match segment.segment_type {
                    crate::shapes::paths::PathSegmentType::MoveTo => {
                        builder.move_to(segment.x, segment.y);
                    }
                    crate::shapes::paths::PathSegmentType::LineTo => {
                        builder.line_to(segment.x, segment.y);
                    }
                    crate::shapes::paths::PathSegmentType::QuadraticTo => {
                        builder.quadratic_to(segment.x, segment.y, segment.cp1_x, segment.cp1_y);
                    }
                    crate::shapes::paths::PathSegmentType::CubicTo => {
                        builder.bezier_to(segment.x, segment.y, segment.cp1_x, segment.cp1_y, segment.cp2_x, segment.cp2_y);
                    }
                    crate::shapes::paths::PathSegmentType::Close => {
                        builder.close();
                    }
                }
            }
        }

        builder.build()
    }

    /// Perform boolean operation on selected layers
    pub fn boolean_operation_selected(&self, operation: BooleanOperation) -> Result<BooleanResult, String> {
        let selected_layers = self.get_selected_layers();

        if selected_layers.len() < 2 {
            return Err("At least 2 layers must be selected for boolean operations".to_string());
        }

        let shapes: Vec<&dyn ShapePrimitive> = selected_layers.iter()
            .map(|l| l.shape.as_ref())
            .collect();

        let operations = vec![operation; selected_layers.len() - 1];
        AdvancedBoolean::multiple_operations(&shapes, &operations)
    }

    /// Flatten collection (combine all visible layers into one)
    pub fn flatten(&self) -> Result<ShapeLayer, String> {
        let visible_layers = self.get_visible_layers();

        if visible_layers.is_empty() {
            return Err("No visible layers to flatten".to_string());
        }

        if visible_layers.len() == 1 {
            let layer = visible_layers[0];
            return Ok(layer.clone_with_id("flattened"));
        }

        // Perform union of all visible layers
        let mut result_layer = visible_layers[0].clone_with_id("flattened");

        for layer in visible_layers.iter().skip(1) {
            let boolean_result = AdvancedBoolean::union(result_layer.shape.as_ref(), layer.shape.as_ref());

            if boolean_result.success {
                let path_shape = PathShape::from_path(boolean_result.path);
                result_layer.shape = Box::new(path_shape);
                result_layer.name = format!("Combined_{}", layer.name);
            } else {
                return Err(format!("Failed to combine layer '{}': {:?}",
                    layer.name, boolean_result.error));
            }
        }

        Ok(result_layer)
    }


    pub fn set_global_transform(&mut self, transform: Transform) {
        self.global_transform = transform;
    }


    pub fn apply_global_transform(&mut self, transform: &Transform) {
        self.global_transform = self.global_transform.combine(transform);

        for layer in &mut self.layers {
            layer.apply_transform(transform);
        }
    }


    pub fn get_stats(&self) -> LayerCollectionStats {
        let visible_count = self.get_visible_layers().len();
        let selected_count = self.get_selected_layers().len();
        let locked_count = self.layers.iter().filter(|l| l.locked).count();

        let total_area = self.layers.iter()
            .filter(|l| l.is_visible())
            .map(|l| l.area())
            .sum();

        let bounds = self.bounds();

        LayerCollectionStats {
            total_layers: self.layers.len(),
            visible_layers: visible_count,
            selected_layers: selected_count,
            locked_layers: locked_count,
            total_area,
            bounds_width: bounds.width(),
            bounds_height: bounds.height(),
        }
    }


    fn sort_layers_by_order(&mut self) {
        self.layers.sort_by_key(|l| l.order);
        self.update_layer_orders();
    }


    fn update_layer_orders(&mut self) {
        for (i, layer) in self.layers.iter_mut().enumerate() {
            layer.order = i as i32;
        }
    }


    pub fn validate(&self) -> Result<(), String> {
        if self.name.is_empty() {
            return Err("Collection name cannot be empty".to_string());
        }


        let mut ids = std::collections::HashSet::new();
        for layer in &self.layers {
            if ids.contains(&layer.id) {
                return Err(format!("Duplicate layer ID: {}", layer.id));
            }
            ids.insert(&layer.id);


            layer.validate()?;
        }

        Ok(())
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct LayerCollectionStats {

    pub total_layers: usize,

    pub visible_layers: usize,

    pub selected_layers: usize,

    pub locked_layers: usize,

    pub total_area: f64,

    pub bounds_width: f64,

    pub bounds_height: f64,
}

impl LayerCollectionStats {

    pub fn aspect_ratio(&self) -> f64 {
        if self.bounds_height > 0.0 {
            self.bounds_width / self.bounds_height
        } else {
            1.0
        }
    }


    pub fn average_area(&self) -> f64 {
        if self.visible_layers > 0 {
            self.total_area / self.visible_layers as f64
        } else {
            0.0
        }
    }
}


pub struct ShapeLayerCollectionBuilder {
    collection: ShapeLayerCollection,
}

impl ShapeLayerCollectionBuilder {

    pub fn new<S: Into<String>>(name: S) -> Self {
        Self {
            collection: ShapeLayerCollection::new(name),
        }
    }


    pub fn layer(mut self, layer: ShapeLayer) -> Self {
        let _ = self.collection.add_layer(layer);
        self
    }


    pub fn global_transform(mut self, transform: Transform) -> Self {
        self.collection.set_global_transform(transform);
        self
    }


    pub fn metadata<K: Into<String>, V: Into<String>>(mut self, key: K, value: V) -> Self {
        self.collection.metadata.insert(key.into(), value.into());
        self
    }


    pub fn build(self) -> Result<ShapeLayerCollection, String> {
        self.collection.validate()?;
        Ok(self.collection)
    }
}

impl Default for ShapeLayerCollection {
    fn default() -> Self {
        Self::new("Default")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shapes::primitives::{Rectangle, Circle};

    #[test]
    fn test_shape_layer() {
        let rect = Rectangle::new(0.0, 0.0, 100.0, 100.0);
        let mut layer = ShapeLayer::new("layer1", "Rectangle", Box::new(rect));

        assert_eq!(layer.id, "layer1");
        assert_eq!(layer.name, "Rectangle");
        assert!(layer.is_visible());
        assert_eq!(layer.opacity, 1.0);
        assert_eq!(layer.blend_mode, LayerBlendMode::Normal);

        layer.set_opacity(0.5);
        assert_eq!(layer.opacity, 0.5);

        layer.select();
        assert!(layer.selected);

        layer.lock();
        assert!(layer.locked);
        assert!(!layer.selected);
    }

    #[test]
    fn test_layer_collection() {
        let mut collection = ShapeLayerCollection::new("Test Collection");

        let rect1 = Rectangle::new(0.0, 0.0, 50.0, 50.0);
        let rect2 = Rectangle::new(25.0, 25.0, 50.0, 50.0);

        let layer1 = ShapeLayer::new("layer1", "Rect1", Box::new(rect1));
        let layer2 = ShapeLayer::new("layer2", "Rect2", Box::new(rect2));

        collection.add_layer(layer1).unwrap();
        collection.add_layer(layer2).unwrap();

        assert_eq!(collection.layers.len(), 2);
        assert_eq!(collection.get_visible_layers().len(), 2);

        let stats = collection.get_stats();
        assert_eq!(stats.total_layers, 2);
        assert_eq!(stats.visible_layers, 2);
        assert!(stats.total_area > 0.0);
    }

    #[test]
    fn test_layer_selection() {
        let mut collection = ShapeLayerCollection::new("Test");

        let rect = Rectangle::new(0.0, 0.0, 100.0, 100.0);
        let layer = ShapeLayer::new("layer1", "Rect", Box::new(rect));

        collection.add_layer(layer).unwrap();


        let selected = collection.select_layer_at_point(50.0, 50.0);
        assert!(selected.is_some());

        let selected_layers = collection.get_selected_layers();
        assert_eq!(selected_layers.len(), 1);


        collection.deselect_all();
        let selected_layers = collection.get_selected_layers();
        assert_eq!(selected_layers.len(), 0);
    }

    #[test]
    fn test_layer_reordering() {
        let mut collection = ShapeLayerCollection::new("Test");

        let rect1 = Rectangle::new(0.0, 0.0, 50.0, 50.0);
        let rect2 = Rectangle::new(25.0, 25.0, 50.0, 50.0);

        let layer1 = ShapeLayer::new("layer1", "Rect1", Box::new(rect1));
        let layer2 = ShapeLayer::new("layer2", "Rect2", Box::new(rect2));

        collection.add_layer(layer1).unwrap();
        collection.add_layer(layer2).unwrap();


        collection.move_layer_up("layer2").unwrap();

        let layer2_index = collection.find_layer_index("layer2").unwrap();
        assert_eq!(layer2_index, 0);
    }

    #[test]
    fn test_collection_builder() {
        let rect = Rectangle::new(0.0, 0.0, 100.0, 100.0);
        let layer = ShapeLayer::new("layer1", "Rect", Box::new(rect));

        let collection = ShapeLayerCollectionBuilder::new("Test Collection")
            .layer(layer)
            .global_transform(Transform::translation(10.0, 20.0))
            .metadata("author", "test")
            .build()
            .unwrap();

        assert_eq!(collection.name, "Test Collection");
        assert_eq!(collection.layers.len(), 1);
        assert_eq!(collection.global_transform.tx, 10.0);
        assert_eq!(collection.global_transform.ty, 20.0);
        assert_eq!(collection.metadata.get("author"), Some(&"test".to_string()));
    }

    #[test]
    fn test_layer_validation() {
        let rect = Rectangle::new(0.0, 0.0, 100.0, 100.0);
        let layer = ShapeLayer::new("", "", Box::new(rect));

        assert!(layer.validate().is_err());
    }

    #[test]
    fn test_collection_validation() {
        let mut collection = ShapeLayerCollection::new("");

        let rect = Rectangle::new(0.0, 0.0, 100.0, 100.0);
        let layer = ShapeLayer::new("layer1", "Rect", Box::new(rect));

        assert!(collection.validate().is_err());

        collection.name = "Test".to_string();
        collection.add_layer(layer).unwrap();

        assert!(collection.validate().is_ok());
    }
}
