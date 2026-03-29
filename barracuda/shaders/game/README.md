# ludoSpring Game Shaders

WGSL compute shaders for 2D game engine operations. Dispatched via
barraCuda `ComputeDispatch` or toadStool `compute.submit`.

## Shaders

| Shader | Purpose | Dispatch Pattern |
|--------|---------|-----------------|
| `fog_of_war.wgsl` | Per-tile visibility from viewer position — **Bresenham line-of-sight** with terrain wall occlusion (not distance-only radial masks) | 1 thread/tile, single dispatch |
| `tile_lighting.wgsl` | Point light propagation with 1/d² falloff | 1 thread/tile, single dispatch |
| `pathfind_wavefront.wgsl` | BFS wavefront expansion (one ring/dispatch) | 1 thread/tile, loop until frontier=0 |

## Inherited from barraCuda (ecoPrimals)

| Shader | Origin | Purpose |
|--------|--------|---------|
| `perlin_2d.wgsl` | barraCuda `ops/procedural/` | Terrain generation |
| `dda_raycast.wgsl` | exp030 validated | Batch line-of-sight |

## Dispatch

### Via barraCuda (in-process)

```rust
ComputeDispatch::new(&device, "fog_of_war")
    .shader(FOG_WGSL, "main")
    .uniform(0, &params_buf)
    .storage_read(1, &terrain_buf)
    .storage_read(2, &prev_vis_buf)
    .storage_rw(3, &out_vis_buf)
    .dispatch_1d(tile_count)
    .submit()?;
```

### Via toadStool (IPC)

```json
{
  "jsonrpc": "2.0",
  "method": "compute.submit",
  "params": {
    "job_type": "Custom",
    "plugin": "wgsl",
    "payload": {
      "shader": "fog_of_war",
      "grid_w": 100,
      "grid_h": 100,
      "viewer_x": 50.0,
      "viewer_y": 50.0,
      "sight_radius": 8
    }
  },
  "id": 1
}
```

### Via coralReef (native compile)

```json
{
  "jsonrpc": "2.0",
  "method": "shader.compile.wgsl",
  "params": {
    "source": "<fog_of_war.wgsl contents>",
    "entry_point": "main"
  },
  "id": 1
}
```

## License

AGPL-3.0-or-later
