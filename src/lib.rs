#![forbid(unsafe_code)]

//! Mapping and spatial representation for fleet topology visualization.
//!
//! Provides data structures for building, projecting, and reading maps
//! of a fleet's rooms and agent distribution. Designed for human operators
//! who need to understand fleet layout at a glance.

use std::collections::HashMap;

/// A point in 2D space used for map projections.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Point2D {
    pub x: f64,
    pub y: f64,
}

impl Point2D {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    /// Euclidean distance to another point.
    pub fn distance_to(&self, other: &Point2D) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }

    /// Midpoint between this and another point.
    pub fn midpoint(&self, other: &Point2D) -> Point2D {
        Point2D::new((self.x + other.x) / 2.0, (self.y + other.y) / 2.0)
    }
}

/// A bounded rectangular region on the map.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MapRegion {
    pub top_left: Point2D,
    pub bottom_right: Point2D,
}

impl MapRegion {
    pub fn new(top_left: Point2D, bottom_right: Point2D) -> Self {
        Self { top_left, bottom_right }
    }

    pub fn width(&self) -> f64 {
        self.bottom_right.x - self.top_left.x
    }

    pub fn height(&self) -> f64 {
        self.bottom_right.y - self.top_left.y
    }

    pub fn area(&self) -> f64 {
        self.width() * self.height()
    }

    pub fn contains(&self, point: &Point2D) -> bool {
        point.x >= self.top_left.x
            && point.x <= self.bottom_right.x
            && point.y >= self.top_left.y
            && point.y <= self.bottom_right.y
    }

    pub fn center(&self) -> Point2D {
        self.top_left.midpoint(&self.bottom_right)
    }
}

/// The main cartographic builder. Accumulates spatial data and produces fleet maps.
#[derive(Clone, Debug)]
pub struct Cartograph {
    /// Named entities and their positions.
    markers: HashMap<String, Point2D>,
    /// Named connections between entities.
    routes: Vec<(String, String)>,
    /// Named regions with territory marks.
    regions: HashMap<String, MapRegion>,
}

impl Cartograph {
    pub fn new() -> Self {
        Self {
            markers: HashMap::new(),
            routes: Vec::new(),
            regions: HashMap::new(),
        }
    }

    /// Place a named marker at a position.
    pub fn place_marker(&mut self, name: &str, position: Point2D) {
        self.markers.insert(name.to_string(), position);
    }

    /// Connect two markers by name. No-op if either doesn't exist.
    pub fn connect(&mut self, from: &str, to: &str) {
        if self.markers.contains_key(from) && self.markers.contains_key(to) {
            self.routes.push((from.to_string(), to.to_string()));
        }
    }

    /// Define a named region.
    pub fn define_region(&mut self, name: &str, region: MapRegion) {
        self.regions.insert(name.to_string(), region);
    }

    /// Get the position of a named marker.
    pub fn marker_position(&self, name: &str) -> Option<Point2D> {
        self.markers.get(name).copied()
    }

    /// List all marker names.
    pub fn marker_names(&self) -> Vec<&str> {
        self.markers.keys().map(|s| s.as_str()).collect()
    }

    /// List all routes.
    pub fn routes(&self) -> &[(String, String)] {
        &self.routes
    }

    /// Find which region(s) a point falls within.
    pub fn regions_containing(&self, point: &Point2D) -> Vec<&str> {
        self.regions
            .iter()
            .filter(|(_, region)| region.contains(point))
            .map(|(name, _)| name.as_str())
            .collect()
    }

    /// Build a FleetMap from accumulated data.
    pub fn build_map(&self) -> FleetMap {
        FleetMap {
            markers: self.markers.clone(),
            routes: self.routes.clone(),
            regions: self.regions.clone(),
        }
    }

    pub fn marker_count(&self) -> usize {
        self.markers.len()
    }

    pub fn route_count(&self) -> usize {
        self.routes.len()
    }

    pub fn region_count(&self) -> usize {
        self.regions.len()
    }
}

impl Default for Cartograph {
    fn default() -> Self {
        Self::new()
    }
}

/// A completed spatial map of the fleet.
#[derive(Clone, Debug)]
pub struct FleetMap {
    markers: HashMap<String, Point2D>,
    routes: Vec<(String, String)>,
    regions: HashMap<String, MapRegion>,
}

impl FleetMap {
    /// Compute the bounding box of all markers.
    pub fn bounds(&self) -> Option<MapRegion> {
        if self.markers.is_empty() {
            return None;
        }
        let mut min_x = f64::MAX;
        let mut min_y = f64::MAX;
        let mut max_x = f64::MIN;
        let mut max_y = f64::MIN;
        for p in self.markers.values() {
            min_x = min_x.min(p.x);
            min_y = min_y.min(p.y);
            max_x = max_x.max(p.x);
            max_y = max_y.max(p.y);
        }
        Some(MapRegion::new(
            Point2D::new(min_x, min_y),
            Point2D::new(max_x, max_y),
        ))
    }

    /// Total length of all routes.
    pub fn total_route_length(&self) -> f64 {
        self.routes
            .iter()
            .filter_map(|(from, to)| {
                let p1 = self.markers.get(from)?;
                let p2 = self.markers.get(to)?;
                Some(p1.distance_to(p2))
            })
            .sum()
    }

    /// Find the nearest marker to a point.
    pub fn nearest_marker(&self, point: &Point2D) -> Option<&str> {
        self.markers
            .iter()
            .min_by(|(_, a), (_, b)| {
                a.distance_to(point)
                    .partial_cmp(&b.distance_to(point))
                    .unwrap()
            })
            .map(|(name, _)| name.as_str())
    }

    /// Find markers within a radius of a point.
    pub fn markers_within(&self, center: &Point2D, radius: f64) -> Vec<&str> {
        self.markers
            .iter()
            .filter(|(_, pos)| pos.distance_to(center) <= radius)
            .map(|(name, _)| name.as_str())
            .collect()
    }

    /// Get the route path from one marker to another (direct only).
    pub fn has_direct_route(&self, from: &str, to: &str) -> bool {
        self.routes
            .iter()
            .any(|(a, b)| (a == from && b == to) || (a == to && b == from))
    }

    /// Count markers in each region.
    pub fn density_by_region(&self) -> HashMap<String, usize> {
        self.regions
            .iter()
            .map(|(name, region)| {
                let count = self
                    .markers
                    .values()
                    .filter(|p| region.contains(p))
                    .count();
                (name.clone(), count)
            })
            .collect()
    }

    /// Number of markers on this map.
    pub fn marker_count(&self) -> usize {
        self.markers.len()
    }

    /// Number of routes on this map.
    pub fn route_count(&self) -> usize {
        self.routes.len()
    }
}

/// A territory claim on the map. Agents or rooms can claim regions.
#[derive(Clone, Debug, PartialEq)]
pub struct TerritoryMark {
    pub name: String,
    pub region: MapRegion,
    pub owner: String,
    pub priority: u32,
}

impl TerritoryMark {
    pub fn new(name: &str, region: MapRegion, owner: &str, priority: u32) -> Self {
        Self {
            name: name.to_string(),
            region,
            owner: owner.to_string(),
            priority,
        }
    }

    /// Check if this territory overlaps with another.
    pub fn overlaps(&self, other: &TerritoryMark) -> bool {
        // AABB overlap check
        let a = &self.region;
        let b = &other.region;
        a.top_left.x < b.bottom_right.x
            && a.bottom_right.x > b.top_left.x
            && a.top_left.y < b.bottom_right.y
            && a.bottom_right.y > b.top_left.y
    }

    /// Resolve overlap: higher priority wins. Returns the winner's name, or None if equal.
    pub fn resolve_conflict(&self, other: &TerritoryMark) -> Option<String> {
        if self.priority > other.priority {
            Some(self.name.clone())
        } else if other.priority > self.priority {
            Some(other.name.clone())
        } else {
            None
        }
    }
}

/// Projects high-dimensional data down to 2D for map visualization.
#[derive(Clone, Debug)]
pub struct CartographicProjection {
    /// Scale factors for each dimension.
    weights: Vec<f64>,
    /// Offset to apply after projection.
    offset: Point2D,
}

impl CartographicProjection {
    /// Create a projection that picks the first two dimensions with optional weights.
    pub fn new(weights: Vec<f64>, offset: Point2D) -> Self {
        Self { weights, offset }
    }

    /// Simple identity projection for 2D data.
    pub fn identity() -> Self {
        Self {
            weights: vec![1.0, 1.0],
            offset: Point2D::new(0.0, 0.0),
        }
    }

    /// Project a high-dimensional point to 2D.
    /// Uses weighted sum: x = sum(dim[even_i] * weight[i]), y = sum(dim[odd_i] * weight[i]).
    pub fn project(&self, dimensions: &[f64]) -> Point2D {
        let mut x = 0.0;
        let mut y = 0.0;
        for (i, &val) in dimensions.iter().enumerate() {
            let w = self.weights.get(i).copied().unwrap_or(0.0);
            if i % 2 == 0 {
                x += val * w;
            } else {
                y += val * w;
            }
        }
        Point2D::new(x + self.offset.x, y + self.offset.y)
    }

    /// Project multiple points.
    pub fn project_all(&self, points: &[Vec<f64>]) -> Vec<Point2D> {
        points.iter().map(|p| self.project(p)).collect()
    }
}

/// Decodes map symbols and colors for human-readable legends.
#[derive(Clone, Debug, PartialEq)]
pub struct MapLegend {
    entries: HashMap<String, String>,
}

impl MapLegend {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    /// Register a symbol and its meaning.
    pub fn define(&mut self, symbol: &str, meaning: &str) {
        self.entries.insert(symbol.to_string(), meaning.to_string());
    }

    /// Look up what a symbol means.
    pub fn lookup(&self, symbol: &str) -> Option<&str> {
        self.entries.get(symbol).map(|s| s.as_str())
    }

    /// All defined symbols.
    pub fn symbols(&self) -> Vec<&str> {
        self.entries.keys().map(|s| s.as_str()).collect()
    }

    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }
}

impl Default for MapLegend {
    fn default() -> Self {
        Self::new()
    }
}

/// Controls zoom level and what detail is visible at each scale.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MapScale {
    /// Individual agents visible.
    Room,
    /// Room clusters visible.
    Floor,
    /// Floor groups visible.
    Building,
    /// Entire fleet as one view.
    Fleet,
}

impl MapScale {
    /// Rough scale factor relative to Room level.
    pub fn zoom_factor(&self) -> f64 {
        match self {
            MapScale::Room => 1.0,
            MapScale::Floor => 0.1,
            MapScale::Building => 0.01,
            MapScale::Fleet => 0.001,
        }
    }

    /// Which details are visible at this scale.
    pub fn visible_features(&self) -> &[&str] {
        match self {
            MapScale::Room => &["agents", "connections", "messages"],
            MapScale::Floor => &["rooms", "corridors", "clusters"],
            MapScale::Building => &["floors", "elevators", "services"],
            MapScale::Fleet => &["buildings", "network-links", "regions"],
        }
    }

    /// Zoom in one level, if possible.
    pub fn zoom_in(&self) -> Option<MapScale> {
        match self {
            MapScale::Room => None,
            MapScale::Floor => Some(MapScale::Room),
            MapScale::Building => Some(MapScale::Floor),
            MapScale::Fleet => Some(MapScale::Building),
        }
    }

    /// Zoom out one level, if possible.
    pub fn zoom_out(&self) -> Option<MapScale> {
        match self {
            MapScale::Room => Some(MapScale::Floor),
            MapScale::Floor => Some(MapScale::Building),
            MapScale::Building => Some(MapScale::Fleet),
            MapScale::Fleet => None,
        }
    }

    pub fn from_level(level: u8) -> Option<Self> {
        match level {
            0 => Some(MapScale::Room),
            1 => Some(MapScale::Floor),
            2 => Some(MapScale::Building),
            3 => Some(MapScale::Fleet),
            _ => None,
        }
    }

    pub fn level(&self) -> u8 {
        match self {
            MapScale::Room => 0,
            MapScale::Floor => 1,
            MapScale::Building => 2,
            MapScale::Fleet => 3,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_point_distance() {
        let a = Point2D::new(0.0, 0.0);
        let b = Point2D::new(3.0, 4.0);
        assert!((a.distance_to(&b) - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_point_midpoint() {
        let a = Point2D::new(0.0, 0.0);
        let b = Point2D::new(4.0, 6.0);
        let mid = a.midpoint(&b);
        assert_eq!(mid, Point2D::new(2.0, 3.0));
    }

    #[test]
    fn test_region_contains() {
        let region = MapRegion::new(Point2D::new(0.0, 0.0), Point2D::new(10.0, 10.0));
        assert!(region.contains(&Point2D::new(5.0, 5.0)));
        assert!(!region.contains(&Point2D::new(15.0, 5.0)));
    }

    #[test]
    fn test_region_area() {
        let region = MapRegion::new(Point2D::new(0.0, 0.0), Point2D::new(5.0, 10.0));
        assert_eq!(region.area(), 50.0);
    }

    #[test]
    fn test_cartograph_place_markers() {
        let mut cart = Cartograph::new();
        cart.place_marker("room-a", Point2D::new(1.0, 2.0));
        cart.place_marker("room-b", Point2D::new(3.0, 4.0));
        assert_eq!(cart.marker_count(), 2);
        assert_eq!(cart.marker_position("room-a"), Some(Point2D::new(1.0, 2.0)));
    }

    #[test]
    fn test_cartograph_connect_valid() {
        let mut cart = Cartograph::new();
        cart.place_marker("a", Point2D::new(0.0, 0.0));
        cart.place_marker("b", Point2D::new(1.0, 1.0));
        cart.connect("a", "b");
        assert_eq!(cart.route_count(), 1);
    }

    #[test]
    fn test_cartograph_connect_missing_marker_ignored() {
        let mut cart = Cartograph::new();
        cart.place_marker("a", Point2D::new(0.0, 0.0));
        cart.connect("a", "nonexistent");
        assert_eq!(cart.route_count(), 0);
    }

    #[test]
    fn test_cartograph_regions_containing() {
        let mut cart = Cartograph::new();
        cart.define_region("zone-1", MapRegion::new(Point2D::new(0.0, 0.0), Point2D::new(5.0, 5.0)));
        let results = cart.regions_containing(&Point2D::new(3.0, 3.0));
        assert!(results.contains(&"zone-1"));
    }

    #[test]
    fn test_fleet_map_bounds() {
        let mut cart = Cartograph::new();
        cart.place_marker("a", Point2D::new(0.0, 0.0));
        cart.place_marker("b", Point2D::new(10.0, 20.0));
        let map = cart.build_map();
        let bounds = map.bounds().unwrap();
        assert_eq!(bounds.top_left, Point2D::new(0.0, 0.0));
        assert_eq!(bounds.bottom_right, Point2D::new(10.0, 20.0));
    }

    #[test]
    fn test_fleet_map_bounds_empty() {
        let map = Cartograph::new().build_map();
        assert!(map.bounds().is_none());
    }

    #[test]
    fn test_fleet_map_total_route_length() {
        let mut cart = Cartograph::new();
        cart.place_marker("a", Point2D::new(0.0, 0.0));
        cart.place_marker("b", Point2D::new(3.0, 4.0));
        cart.connect("a", "b");
        let map = cart.build_map();
        assert!((map.total_route_length() - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_fleet_map_nearest_marker() {
        let mut cart = Cartograph::new();
        cart.place_marker("near", Point2D::new(1.0, 1.0));
        cart.place_marker("far", Point2D::new(100.0, 100.0));
        let map = cart.build_map();
        assert_eq!(map.nearest_marker(&Point2D::new(2.0, 2.0)), Some("near"));
    }

    #[test]
    fn test_fleet_map_markers_within_radius() {
        let mut cart = Cartograph::new();
        cart.place_marker("close", Point2D::new(1.0, 0.0));
        cart.place_marker("far", Point2D::new(50.0, 0.0));
        let map = cart.build_map();
        let within = map.markers_within(&Point2D::new(0.0, 0.0), 5.0);
        assert!(within.contains(&"close"));
        assert!(!within.contains(&"far"));
    }

    #[test]
    fn test_fleet_map_has_direct_route() {
        let mut cart = Cartograph::new();
        cart.place_marker("a", Point2D::new(0.0, 0.0));
        cart.place_marker("b", Point2D::new(1.0, 1.0));
        cart.connect("a", "b");
        let map = cart.build_map();
        assert!(map.has_direct_route("a", "b"));
        assert!(map.has_direct_route("b", "a"));
        assert!(!map.has_direct_route("a", "c"));
    }

    #[test]
    fn test_fleet_map_density_by_region() {
        let mut cart = Cartograph::new();
        cart.define_region("zone", MapRegion::new(Point2D::new(0.0, 0.0), Point2D::new(5.0, 5.0)));
        cart.place_marker("inside", Point2D::new(2.0, 2.0));
        cart.place_marker("outside", Point2D::new(20.0, 20.0));
        let map = cart.build_map();
        let density = map.density_by_region();
        assert_eq!(density.get("zone"), Some(&1));
    }

    #[test]
    fn test_territory_mark_overlap() {
        let t1 = TerritoryMark::new(
            "alpha",
            MapRegion::new(Point2D::new(0.0, 0.0), Point2D::new(5.0, 5.0)),
            "agent-1",
            1,
        );
        let t2 = TerritoryMark::new(
            "beta",
            MapRegion::new(Point2D::new(3.0, 3.0), Point2D::new(8.0, 8.0)),
            "agent-2",
            2,
        );
        assert!(t1.overlaps(&t2));
    }

    #[test]
    fn test_territory_mark_no_overlap() {
        let t1 = TerritoryMark::new(
            "alpha",
            MapRegion::new(Point2D::new(0.0, 0.0), Point2D::new(5.0, 5.0)),
            "agent-1",
            1,
        );
        let t2 = TerritoryMark::new(
            "beta",
            MapRegion::new(Point2D::new(10.0, 10.0), Point2D::new(15.0, 15.0)),
            "agent-2",
            2,
        );
        assert!(!t1.overlaps(&t2));
    }

    #[test]
    fn test_territory_conflict_resolution() {
        let t1 = TerritoryMark::new("a", MapRegion::new(Point2D::new(0.0, 0.0), Point2D::new(5.0, 5.0)), "o1", 1);
        let t2 = TerritoryMark::new("b", MapRegion::new(Point2D::new(0.0, 0.0), Point2D::new(5.0, 5.0)), "o2", 2);
        assert_eq!(t1.resolve_conflict(&t2), Some("b".to_string()));
    }

    #[test]
    fn test_projection_identity() {
        let proj = CartographicProjection::identity();
        let p = proj.project(&[3.0, 4.0]);
        assert_eq!(p, Point2D::new(3.0, 4.0));
    }

    #[test]
    fn test_projection_weighted() {
        let proj = CartographicProjection::new(
            vec![2.0, 3.0, 0.5, 1.0],
            Point2D::new(0.0, 0.0),
        );
        let p = proj.project(&[1.0, 1.0, 1.0, 1.0]);
        // x = 1*2 + 1*0.5 = 2.5, y = 1*3 + 1*1 = 4.0
        assert_eq!(p, Point2D::new(2.5, 4.0));
    }

    #[test]
    fn test_legend_define_and_lookup() {
        let mut legend = MapLegend::new();
        legend.define("🔴", "agent active");
        legend.define("⬛", "room offline");
        assert_eq!(legend.lookup("🔴"), Some("agent active"));
        assert_eq!(legend.lookup("❓"), None);
        assert_eq!(legend.entry_count(), 2);
    }

    #[test]
    fn test_map_scale_zoom() {
        assert_eq!(MapScale::Room.zoom_out(), Some(MapScale::Floor));
        assert_eq!(MapScale::Room.zoom_in(), None);
        assert_eq!(MapScale::Fleet.zoom_in(), Some(MapScale::Building));
        assert_eq!(MapScale::Fleet.zoom_out(), None);
    }

    #[test]
    fn test_map_scale_from_level() {
        assert_eq!(MapScale::from_level(0), Some(MapScale::Room));
        assert_eq!(MapScale::from_level(3), Some(MapScale::Fleet));
        assert_eq!(MapScale::from_level(5), None);
    }

    #[test]
    fn test_map_scale_visible_features() {
        assert!(MapScale::Room.visible_features().contains(&"agents"));
        assert!(MapScale::Fleet.visible_features().contains(&"regions"));
    }
}
