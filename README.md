# Ternary Cartograph

**Ternary Cartograph** provides mapping and spatial representation for fleet topology visualization — building, projecting, and reading 2D maps of room and agent distributions across the fleet.

## Why It Matters

A fleet of hundreds of rooms distributed across geographic locations needs a map. Operators must see the topology at a glance: which rooms are near each other, where are the cluster boundaries, what's the spatial distribution of agent density. Ternary Cartograph provides the geometric primitives — points, regions, projections — and the spatial query operations — distance, containment, nearest-region — that make fleet visualization possible.

## How It Works

### Geometric Primitives

```
Point2D { x: f64, y: f64 }
    - distance_to(other) → √((x1-x2)² + (y1-y2)²)     O(1)
    - midpoint(other) → ((x1+x2)/2, (y1+y2)/2)          O(1)

MapRegion { top_left: Point2D, bottom_right: Point2D }
    - width() → bottom_right.x - top_left.x               O(1)
    - height() → top_left.y - bottom_right.y              O(1)
    - area() → width × height                             O(1)
    - contains(point) → bounds check                      O(1)
```

### Fleet Map

The `FleetMap` aggregates room positions and agent distributions:

```
rooms: HashMap<room_id, Point2D>         // room positions
regions: Vec<MapRegion>                   // bounded areas
agent_density: HashMap<region, count>     // agents per region
```

Room placement: **O(1)**. Density query: **O(1)**. Region lookup for point: **O(R)** for R regions (linear scan).

### Projection Modes

- **Geographic**: Map latitude/longitude to Point2D via equirectangular projection
- **Topological**: Position rooms by network distance (hop count) rather than physical distance
- **Force-directed**: Layout rooms with attractive (connected) and repulsive (all pairs) forces

Force-directed layout: **O(N²)** per iteration (all-pairs repulsion), I iterations for convergence.

### Nearest Room

```
nearest_room(point) → Option<room_id>:
    rooms.min_by(|a, b| a.distance_to(point).cmp(b.distance_to(point)))
```

Linear scan: **O(N)** for N rooms. Spatial index (quadtree): **O(log N)** average.

## Quick Start

```rust
use ternary_cartograph::{FleetMap, Point2D, MapRegion};

let mut map = FleetMap::new();
map.place_room("engine_room", Point2D::new(0.0, 0.0));
map.place_room("bridge", Point2D::new(100.0, 200.0));

let dist = map.room_distance("engine_room", "bridge");
println!("Distance: {:.1}", dist);

let region = MapRegion::new(Point2D::new(-50.0, 250.0), Point2D::new(150.0, -50.0));
assert!(region.contains(&Point2D::new(50.0, 50.0)));
```

## API

| Type | Description |
|------|-------------|
| `Point2D` | 2D coordinate with distance and midpoint |
| `MapRegion` | Axis-aligned rectangle with area and containment |
| `FleetMap` | Room positions, regions, agent density |
| `Projection` | Geographic, topological, force-directed modes |

## Architecture Notes

Ternary Cartograph provides the spatial visualization layer for fleet operators in SuperInstance. In γ + η = C, the map reveals γ (growth — where rooms are expanding) and η (avoidance — gaps in the map indicate avoided regions). Integrates with `ternary-compass` for orientation and `ternary-beacon` for distance-based discovery.

See [ARCHITECTURE.md](https://github.com/SuperInstance/SuperInstance/blob/main/ARCHITECTURE.md) for fleet topology.


### Projection Modes

- **Geographic**: Map lat/lon to Point2D via equirectangular projection: `x = lon × R × cos(lat₀), y = lat × R` where R = Earth radius
- **Topological**: Position rooms by network hop count rather than physical distance — captures latency-relevant structure
- **Force-directed**: Layout rooms with spring forces: attractive (connected rooms pull together) and repulsive (all pairs push apart). Per iteration: **O(N²)** all-pairs repulsion. Convergence: typically 50-500 iterations.

### Spatial Queries

```
nearest_room(point) → Option<room_id>:
    linear scan O(N) or quadtree O(log N)

rooms_in_region(region) → Vec<room_id>:
    linear scan O(N) or quadtree O(log N + K) for K results
```

### Agent Density Heatmap

The FleetMap tracks agent density per region:

```
density(region) = agent_count / region.area
```

Enables hotspot detection: regions with density > 2σ from mean are overloaded.

## References

1. Tobler, W. R. (1970). "A Computer Movie Simulating Urban Growth in the Detroit Region." *Economic Geography*, 46, 234–240. (First Law of Geography)
2. Eades, P. (1984). "A Heuristic for Graph Drawing." *Congressus Numerantium*, 42, 149–160.
3. Fruchterman, T. & Reingold, E. (1991). "Graph Drawing by Force-Directed Placement." *Software: Practice and Experience*, 21(11), 1129–1164.

## License

MIT
