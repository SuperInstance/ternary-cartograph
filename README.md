# ternary-cartograph

A spatial mapping and fleet topology visualization library. Provides 2D point geometry, bounding regions, territory claims with conflict resolution, multi-resolution map scales, and dimensionality-reduction projections for rendering high-dimensional fleet state onto human-readable 2D maps.

## Why It Matters

A fleet of autonomous agents operating across rooms, floors, and buildings generates state that is inherently spatial yet multi-dimensional. Operators need to *see* where agents are, which regions they claim, and how they connect — at varying levels of detail from individual agents up to the entire fleet as a single dot.

This crate provides the cartographic primitives for that visualization: points, regions, routes, territories, projections, and a discrete zoom scale (Room → Floor → Building → Fleet) that controls which features are visible at each level.

Within the **γ + η = C** framework:

| Symbol | Domain |
|--------|--------|
| γ | Spatial state: marker positions, region bounds, territory claims |
| η | Projection and scale decisions: what to show, how to project |
| C | Geometric constraints: containment, overlap, bounding boxes |

## How It Works

### Euclidean Geometry

All distances use the standard L2 norm:

$$d(p_1, p_2) = \sqrt{(x_1 - x_2)^2 + (y_1 - y_2)^2}$$

### Bounding Box Computation

For a set of *n* markers, the bounding box is computed in O(n):

$$x_{\min} = \min_i x_i, \quad x_{\max} = \max_i x_i, \quad y_{\min} = \min_i y_i, \quad y_{\max} = \max_i y_i$$

### AABB Overlap Test

Territory overlap uses the separating axis theorem for axis-aligned bounding boxes (AABB):

$$\text{overlap} \iff x_1^{\text{min}} < x_2^{\text{max}} \;\land\; x_1^{\text{max}} > x_2^{\text{min}} \;\land\; y_1^{\text{min}} < y_2^{\text{max}} \;\land\; y_1^{\text{max}} > y_2^{\text{min}}$$

This is an O(1) constant-time check per pair.

### Cartographic Projection

High-dimensional data is projected to 2D via weighted parity splitting:

$$x = \sum_{i \text{ even}} w_i \cdot d_i + \Delta_x, \qquad y = \sum_{i \text{ odd}} w_i \cdot d_i + \Delta_y$$

where $w_i$ are per-dimension weights and $(\Delta_x, \Delta_y)$ is an offset. This is a simplified form of **principal component projection** without the eigendecomposition step — fast enough for real-time rendering but lossy for high-dimensional data.

### Nearest-Marker Query

```rust
pub fn nearest_marker(&self, point: &Point2D) -> Option<&str>
```

This is an O(n) linear scan. For large marker counts, a k-d tree or R-tree would reduce this to O(log n), but fleet scales (typically < 500 markers) do not justify the complexity.

### Discrete Map Scale

The zoom hierarchy follows a 10× scaling factor per level:

| Level | Scale | Zoom Factor | Visible Features |
|-------|-------|-------------|------------------|
| 0 — Room | 1.0 | 1.0 | agents, connections, messages |
| 1 — Floor | 0.1 | 10⁻¹ | rooms, corridors, clusters |
| 2 — Building | 0.01 | 10⁻² | floors, elevators, services |
| 3 — Fleet | 0.001 | 10⁻³ | buildings, network-links, regions |

### Complexity

| Operation | Time | Notes |
|-----------|------|-------|
| `place_marker` | O(1) amortized | HashMap insert |
| `connect` | O(1) | Vec push with existence check |
| `bounds` | O(n) | Single pass over markers |
| `nearest_marker` | O(n) | Linear scan |
| `markers_within` | O(n) | Linear scan with radius filter |
| `density_by_region` | O(n · m) | n markers × m regions |
| `project` | O(d) | d = dimensionality of input |

## Quick Start

```rust
use ternary_cartograph::{Cartograph, Point2D, MapRegion, FleetMap, MapScale, TerritoryMark};

let mut cart = Cartograph::new();

// Place agents
cart.place_marker("agent-1", Point2D::new(0.0, 0.0));
cart.place_marker("agent-2", Point2D::new(3.0, 4.0));
cart.connect("agent-1", "agent-2");

// Define a region
cart.define_region("zone-alpha", MapRegion::new(
    Point2D::new(-1.0, -1.0),
    Point2D::new(5.0, 5.0),
));

let map = cart.build_map();

// Query
assert_eq!(map.nearest_marker(&Point2D::new(1.0, 1.0)), Some("agent-1"));
assert!((map.total_route_length() - 5.0).abs() < 1e-10);

// Territory claims
let t1 = TerritoryMark::new("alpha", MapRegion::new(
    Point2D::new(0.0, 0.0), Point2D::new(5.0, 5.0),
), "agent-1", 1);
let t2 = TerritoryMark::new("beta", MapRegion::new(
    Point2D::new(3.0, 3.0), Point2D::new(8.0, 8.0),
), "agent-2", 2);
assert!(t1.overlaps(&t2));
assert_eq!(t1.resolve_conflict(&t2), Some("beta".into())); // higher priority wins
```

## API

### Core Types

- **`Point2D`** — `(x, y)` with `distance_to`, `midpoint`.
- **`MapRegion`** — Axis-aligned rectangle with `width`, `height`, `area`, `contains`, `center`.
- **`Cartograph`** — Builder: `place_marker`, `connect`, `define_region`, `build_map`.
- **`FleetMap`** — Immutable map: `bounds`, `total_route_length`, `nearest_marker`, `markers_within`, `has_direct_route`, `density_by_region`.
- **`TerritoryMark`** — Claimed region with `overlaps`, `resolve_conflict`.
- **`CartographicProjection`** — Weighted dimensionality reducer: `project`, `project_all`.
- **`MapLegend`** — Symbol → meaning dictionary.
- **`MapScale`** — Discrete zoom enum: `Room`, `Floor`, `Building`, `Fleet`.

## Architecture Notes

The cartograph is a **constructive** model: you accumulate spatial data via the builder, then freeze it into a `FleetMap` for queries. This separates the mutation phase (agents moving, regions being defined) from the query phase (operators inspecting the map).

The `TerritoryMark` conflict resolution uses a simple priority integer — this is deliberately human-assignable rather than automatically computed, because territory disputes in agent fleets are policy decisions, not geometric ones.

The `CartographicProjection` is intentionally simple (weighted sum, not PCA) because it runs in O(d) per point and must execute every frame for real-time visualization. For offline analysis, use a proper PCA or UMAP implementation.

## References

- **Berg, M. D., Cheong, O., Kreveld, M., & Overmars, M.** (2008). *Computational Geometry* (3rd ed.). — AABB intersection, convex hulls, spatial data structures.
- **Cormen, T. H., et al.** (2009). *Introduction to Algorithms* (3rd ed.), Ch. 33 — Computational geometry basics.
- **Tufte, E. R.** (1990). *Envisioning Information*. — Multi-resolution display and visual density.
- **Paterson, M. S., & Yao, F. F.** (1990). "Binary Space Partitions for Axis-Aligned Rectangles." — AABB partition trees.

## License

MIT
