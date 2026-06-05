# ternary-cartograph: Mapping and spatial representation for fleet topology

Maps fleet rooms, agents, and territories into a 2D spatial layout that human operators can read and navigate.

## Why This Exists

A fleet of rooms and agents is hard to visualize. When an operator needs to see where things are — which rooms cluster together, which territories overlap, what the overall layout looks like — they need a map. This crate provides the primitives to build that map: markers, regions, routes, projections, and legends.

## Core Concepts

- **Cartograph**: The map builder. You place markers, draw routes, and define regions, then build a `FleetMap`.
- **FleetMap**: An immutable spatial snapshot. Query bounds, find nearest markers, compute route lengths, check density.
- **TerritoryMark**: A claimed region with an owner and priority. Overlapping claims resolve by priority.
- **CartographicProjection**: Projects high-dimensional data (latency, load, agent count per room) down to 2D coordinates for map placement.
- **MapLegend**: A lookup table of symbols and their meanings (e.g., "🔴" → "agent active").
- **MapScale**: Zoom levels (Room → Floor → Building → Fleet) controlling what detail is visible.

## Quick Start

```toml
[dependencies]
ternary-cartograph = "0.1"
```

```rust
use ternary_cartograph::{Cartograph, FleetMap, MapRegion, Point2D, MapScale};

let mut cart = Cartograph::new();
cart.place_marker("engine-room", Point2D::new(0.0, 0.0));
cart.place_marker("bridge", Point2D::new(10.0, 5.0));
cart.connect("engine-room", "bridge");
cart.define_region("critical", MapRegion::new(
    Point2D::new(-1.0, -1.0),
    Point2D::new(11.0, 6.0),
));

let map: FleetMap = cart.build_map();
assert_eq!(map.marker_count(), 2);
assert!(map.has_direct_route("engine-room", "bridge"));

let scale = MapScale::Room;
assert!(scale.visible_features().contains(&"agents"));
```

## API Overview

| Type | Description |
|------|-------------|
| `Cartograph` | Builder that accumulates markers, routes, regions and produces a `FleetMap` |
| `FleetMap` | Immutable spatial map with query methods (bounds, nearest, density) |
| `TerritoryMark` | A named, owned region with priority for conflict resolution |
| `CartographicProjection` | Weighted projection from N-dimensional data to 2D points |
| `MapLegend` | Symbol → meaning lookup for map decoration |
| `MapScale` | Zoom level enum (Room/Floor/Building/Fleet) with feature visibility |
| `Point2D` | Basic 2D coordinate with distance and midpoint operations |
| `MapRegion` | Axis-aligned bounding box with containment and area queries |

## How It Works

The `Cartograph` accumulates spatial data in `HashMap`s and `Vec`s. When you call `build_map()`, it clones the data into a `FleetMap` — a read-only snapshot you can query. There's no incremental updating; if the fleet changes, build a new map.

Territory overlap detection uses axis-aligned bounding box (AABB) intersection. This is fast but imprecise — two diagonal regions that barely touch may report as overlapping. For complex shapes, you'd need a different approach.

The `CartographicProjection` uses a simple weighted-sum strategy: even-indexed dimensions contribute to X, odd to Y, each scaled by a weight. This is intentionally simple — for more sophisticated projections (t-SNE, PCA), use `ternary-projection`.

## Known Limitations

- **No incremental updates**: Building a new map clones all data. For fleets with thousands of rooms, this allocates.
- **AABB-only regions**: Territory overlap detection is rectangular. Non-rectangular regions aren't supported.
- **No routing algorithm**: `FleetMap` stores direct connections only. Shortest-path or A* routing isn't included.
- **No coordinate normalization**: The projection doesn't normalize output to a viewport. You'll need to handle that in your renderer.

## Use Cases

- **Fleet dashboard**: Visualize all rooms on a 2D map with agent density heatmaps.
- **Territory management**: Show which teams own which rooms, detect overlapping claims.
- **Zoom controls**: Let operators drill down from fleet overview to individual room detail.
- **Map legends**: Generate human-readable symbol decodings for printed or rendered maps.

## Ecosystem Context

Part of the SuperInstance ternary fleet. Related crates:
- `ternary-room`: Defines the room model that cartograph maps.
- `ternary-topology`: Network topology that could feed into cartograph projections.
- `ternary-projection`: More sophisticated dimensionality reduction.

## License

MIT

## See Also
- **ternary-navigator** — related
- **ternary-compass** — related
- **ternary-observatory** — related
- **ternary-beacon** — related
- **ternary-frontier** — related

