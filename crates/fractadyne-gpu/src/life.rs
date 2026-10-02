//! The Life tile stepper on the GPU (design/automata.md §4.3) — the app's first compute pipeline,
//! and the twin of `fractadyne_core::life::Universe`: the same 64×64-cell tiles, the same halo of
//! stored tiles around every occupied one, the same rule table. `life.wgsl`'s `cs_step` advances
//! the stored tiles a generation per dispatch; after each batch `cs_stats` counts each tile's cells
//! that differ from the background, and the host grows and shrinks the tile set from that.
//!
//! **Why a batch may run [`BATCH`] generations without the host looking:** a live cell moves at most
//! one cell a generation, so a pattern that starts a batch inside its occupied tiles cannot cross
//! the one-tile (64-cell) halo in fewer than 64. The shader's breach counter — a cell left on an
//! edge whose neighbour is not stored — is the tripwire that this holds; it must read zero.

use egui_wgpu::wgpu;
use fractadyne_core::life::{Rule, Topology, Universe, TILE, TILE_CELLS};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

const SOURCE: &str = include_str!("life.wgsl");
const TILE_BYTES: u64 = TILE_CELLS as u64;
/// Generations a batch runs between the host's looks at the tile set (well inside the halo's 63).
pub const BATCH: u32 = 16;
/// Per-generation parameters sit 256 bytes apart (the uniform dynamic-offset alignment).
const GEN_STRIDE: u64 = 256;
/// The `Gen` uniform's size in `life.wgsl`: the 512-bit rule as four vec4s, then four u32.
const GEN_SIZE: u64 = 80;
/// The most tiles a pool may hold: a dispatch's y dimension is one per active tile.
pub const MAX_TILES: u32 = 65_535;

/// What can stop the GPU stepper.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum LifeGpuError {
    /// The universe outgrew the tile pool. The state is still exact as of the last batch.
    #[error("the universe needs {needed} tiles and the GPU pool holds {capacity}")]
    PoolFull { needed: usize, capacity: u32 },
    #[error("GPU read-back failed: {0}")]
    Readback(String),
}

/// What one call to [`LifeGpu::step`] did.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct StepReport {
    pub generations: u64,
    pub batches: u32,
    /// Tiles stepped in the last batch.
    pub tiles: usize,
}

/// The GPU universe.
pub struct LifeGpu {
    capacity: u32,
    pools: [wgpu::Buffer; 2],
    /// Which pool holds the current generation.
    cur: usize,
    gens: wgpu::Buffer,
    active_buf: wgpu::Buffer,
    links_buf: wgpu::Buffer,
    clip_buf: wgpu::Buffer,
    breach_buf: wgpu::Buffer,
    stats_uniform: wgpu::Buffer,
    population_buf: wgpu::Buffer,
    readback: wgpu::Buffer,
    step_pipeline: wgpu::ComputePipeline,
    stats_pipeline: wgpu::ComputePipeline,
    /// Indexed by the SOURCE pool.
    step_groups: [wgpu::BindGroup; 2],
    /// Indexed by the pool counted.
    stats_groups: [wgpu::BindGroup; 2],

    rule: Rule,
    topology: Topology,
    background: u8,
    generation: u64,
    slot_of: HashMap<(i64, i64), u32>,
    coord_of: Vec<Option<(i64, i64)>>,
    free: Vec<u32>,
    /// The slots stepped, in dispatch order.
    active: Vec<u32>,
    /// Per active index, from the last read-back: cells that differ from the background.
    population: Vec<u32>,
    breaches: u64,
}

fn storage(read_only: bool) -> wgpu::BindingType {
    wgpu::BindingType::Buffer {
        ty: wgpu::BufferBindingType::Storage { read_only },
        has_dynamic_offset: false,
        min_binding_size: None,
    }
}

fn entry(binding: u32, ty: wgpu::BindingType) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry { binding, visibility: wgpu::ShaderStages::COMPUTE, ty, count: None }
}

fn buffer(device: &wgpu::Device, label: &str, size: u64, usage: wgpu::BufferUsages) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor { label: Some(label), size: size.max(16), usage, mapped_at_creation: false })
}

/// The tile pool a device can hold: up to `budget` bytes per pool (two pools), the device's
/// storage binding limit and [`MAX_TILES`].
pub fn pool_capacity(device: &wgpu::Device, budget: u64) -> u32 {
    let by_binding = u64::from(device.limits().max_storage_buffer_binding_size) / TILE_BYTES;
    let by_buffer = device.limits().max_buffer_size / TILE_BYTES;
    (budget / TILE_BYTES).min(by_binding).min(by_buffer).min(u64::from(MAX_TILES)) as u32
}

impl LifeGpu {
    /// A GPU universe with room for `capacity` tiles (see [`pool_capacity`]), holding an empty
    /// plane under Life until [`LifeGpu::load`].
    pub fn new(device: &wgpu::Device, capacity: u32) -> LifeGpu {
        let capacity = capacity.clamp(9, MAX_TILES);
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("life.wgsl"),
            source: wgpu::ShaderSource::Wgsl(SOURCE.into()),
        });
        use wgpu::BufferUsages as U;
        let pool_size = u64::from(capacity) * TILE_BYTES;
        let pools = [0, 1].map(|i| buffer(device, &format!("life.pool{i}"), pool_size, U::STORAGE | U::COPY_DST | U::COPY_SRC));
        let gens = buffer(device, "life.gens", GEN_STRIDE * u64::from(BATCH), U::UNIFORM | U::COPY_DST);
        let active_buf = buffer(device, "life.active", 4 * u64::from(capacity), U::STORAGE | U::COPY_DST);
        let links_buf = buffer(device, "life.links", 32 * u64::from(capacity), U::STORAGE | U::COPY_DST);
        let clip_buf = buffer(device, "life.clip", 16 * u64::from(capacity), U::STORAGE | U::COPY_DST);
        let breach_buf = buffer(device, "life.breach", 16, U::STORAGE | U::COPY_DST | U::COPY_SRC);
        let stats_uniform = buffer(device, "life.stats", 16, U::UNIFORM | U::COPY_DST);
        let population_buf = buffer(device, "life.population", 4 * u64::from(capacity), U::STORAGE | U::COPY_SRC);
        let readback = buffer(device, "life.readback", 4 * u64::from(capacity) + 16, U::MAP_READ | U::COPY_DST);

        let step_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("life.step.layout"),
            entries: &[
                entry(
                    0,
                    wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: true,
                        min_binding_size: wgpu::BufferSize::new(GEN_SIZE),
                    },
                ),
                entry(1, storage(true)),
                entry(2, storage(false)),
                entry(3, storage(true)),
                entry(4, storage(true)),
                entry(5, storage(true)),
                entry(6, storage(false)),
            ],
        });
        let stats_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("life.stats.layout"),
            entries: &[
                entry(
                    7,
                    wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None },
                ),
                entry(8, storage(true)),
                entry(9, storage(true)),
                entry(10, storage(false)),
            ],
        });
        let pipeline = |layout: &wgpu::BindGroupLayout, entry_point: &str| {
            let pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some(entry_point),
                bind_group_layouts: &[layout],
                push_constant_ranges: &[],
            });
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some(entry_point),
                layout: Some(&pl),
                module: &module,
                entry_point: Some(entry_point),
                compilation_options: Default::default(),
                cache: None,
            })
        };
        let step_pipeline = pipeline(&step_layout, "cs_step");
        let stats_pipeline = pipeline(&stats_layout, "cs_stats");
        fn whole(b: &wgpu::Buffer) -> wgpu::BindingResource<'_> {
            b.as_entire_binding()
        }
        let step_groups = [0, 1].map(|s| {
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("life.step.group"),
                layout: &step_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                            buffer: &gens,
                            offset: 0,
                            size: wgpu::BufferSize::new(GEN_SIZE),
                        }),
                    },
                    wgpu::BindGroupEntry { binding: 1, resource: whole(&pools[s]) },
                    wgpu::BindGroupEntry { binding: 2, resource: whole(&pools[1 - s]) },
                    wgpu::BindGroupEntry { binding: 3, resource: whole(&active_buf) },
                    wgpu::BindGroupEntry { binding: 4, resource: whole(&links_buf) },
                    wgpu::BindGroupEntry { binding: 5, resource: whole(&clip_buf) },
                    wgpu::BindGroupEntry { binding: 6, resource: whole(&breach_buf) },
                ],
            })
        });
        let stats_groups = [0, 1].map(|s| {
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("life.stats.group"),
                layout: &stats_layout,
                entries: &[
                    wgpu::BindGroupEntry { binding: 7, resource: whole(&stats_uniform) },
                    wgpu::BindGroupEntry { binding: 8, resource: whole(&pools[s]) },
                    wgpu::BindGroupEntry { binding: 9, resource: whole(&active_buf) },
                    wgpu::BindGroupEntry { binding: 10, resource: whole(&population_buf) },
                ],
            })
        });
        LifeGpu {
            capacity,
            pools,
            cur: 0,
            gens,
            active_buf,
            links_buf,
            clip_buf,
            breach_buf,
            stats_uniform,
            population_buf,
            readback,
            step_pipeline,
            stats_pipeline,
            step_groups,
            stats_groups,
            rule: Rule::life(),
            topology: Topology::Plane,
            background: 0,
            generation: 0,
            slot_of: HashMap::new(),
            coord_of: vec![None; capacity as usize],
            free: (0..capacity).rev().collect(),
            active: Vec::new(),
            population: Vec::new(),
            breaches: 0,
        }
    }

    pub fn capacity(&self) -> u32 {
        self.capacity
    }

    pub fn rule(&self) -> &Rule {
        &self.rule
    }

    pub fn topology(&self) -> Topology {
        self.topology
    }

    pub fn generation(&self) -> u64 {
        self.generation
    }

    pub fn background(&self) -> u8 {
        self.background
    }

    /// Tiles stored (and stepped): the occupied ones and their halo.
    pub fn tile_count(&self) -> usize {
        self.active.len()
    }

    /// Cells that differ from the background, as of the last batch (or load).
    pub fn population(&self) -> u64 {
        self.population.iter().map(|&p| u64::from(p)).sum()
    }

    /// Times the breach tripwire fired since the last load. Zero, or the halo argument failed.
    pub fn breaches(&self) -> u64 {
        self.breaches
    }

    /// Replace the GPU universe with `u`: its rule, topology, background, generation and cells.
    pub fn load(&mut self, queue: &wgpu::Queue, u: &Universe) -> Result<(), LifeGpuError> {
        self.rule = u.rule().clone();
        self.topology = u.topology();
        self.background = u.background();
        self.generation = u.generation();
        self.breaches = 0;
        self.slot_of.clear();
        self.coord_of.iter_mut().for_each(|c| *c = None);
        self.free = (0..self.capacity).rev().collect();
        self.active.clear();
        queue.write_buffer(&self.breach_buf, 0, &[0u8; 16]);
        let stored: Vec<(i64, i64)> = u.tiles().map(|(k, _)| k).collect();
        let wanted = self.wanted(&stored);
        if wanted.len() > self.capacity as usize {
            return Err(LifeGpuError::PoolFull { needed: wanted.len(), capacity: self.capacity });
        }
        let blank = [self.background; TILE_CELLS];
        let contents: HashMap<(i64, i64), &[u8; TILE_CELLS]> = u.tiles().collect();
        for key in wanted {
            let slot = self.free.pop().expect("capacity checked");
            self.slot_of.insert(key, slot);
            self.coord_of[slot as usize] = Some(key);
            self.active.push(slot);
            let cells = contents.get(&key).copied().unwrap_or(&blank);
            queue.write_buffer(&self.pools[self.cur], u64::from(slot) * TILE_BYTES, cells);
        }
        self.population = self.active.iter().map(|s| {
            let key = self.coord_of[*s as usize].expect("an active slot has a tile");
            contents.get(&key).map_or(0, |t| t.iter().filter(|&&c| c != self.background).count() as u32)
        }).collect();
        self.upload_tables(queue);
        Ok(())
    }

    /// The tiles to store for these occupied ones: on a plane, each with its eight neighbours; a
    /// torus and a bounded plane keep every tile of their area.
    fn wanted(&self, occupied: &[(i64, i64)]) -> Vec<(i64, i64)> {
        match self.topology {
            Topology::Plane => {
                let mut set = HashSet::with_capacity(occupied.len() * 4);
                for &(tx, ty) in occupied {
                    for dy in -1..=1 {
                        for dx in -1..=1 {
                            set.insert((tx + dx, ty + dy));
                        }
                    }
                }
                let mut v: Vec<_> = set.into_iter().collect();
                v.sort_unstable();
                v
            }
            _ => {
                let mut v = occupied.to_vec();
                v.sort_unstable();
                v
            }
        }
    }

    /// The key of tile `(tx, ty)`'s neighbour at `(dx, dy)` — wrapped on a torus.
    fn neighbour_key(&self, (tx, ty): (i64, i64), dx: i64, dy: i64) -> (i64, i64) {
        match self.topology {
            Topology::Torus { width, height } => {
                ((tx + dx).rem_euclid(i64::from(width) / TILE), (ty + dy).rem_euclid(i64::from(height) / TILE))
            }
            _ => (tx + dx, ty + dy),
        }
    }

    /// Upload the active list, every active slot's neighbour links and its live rectangle.
    fn upload_tables(&self, queue: &wgpu::Queue) {
        let active: Vec<u32> = self.active.clone();
        let mut links = vec![-1i32; self.capacity as usize * 8];
        let mut clip = vec![[0i32, 0, 64, 64]; self.capacity as usize];
        for &slot in &self.active {
            let key = self.coord_of[slot as usize].expect("an active slot has a tile");
            let mut k = 0;
            for dy in -1..=1 {
                for dx in -1..=1 {
                    if dx == 0 && dy == 0 {
                        continue;
                    }
                    let n = self.neighbour_key(key, dx, dy);
                    links[slot as usize * 8 + k] = self.slot_of.get(&n).map_or(-1, |&s| s as i32);
                    k += 1;
                }
            }
            if let Topology::Bounded { x, y, width, height } = self.topology {
                let (ox, oy) = (key.0 * TILE, key.1 * TILE);
                let lo = |v: i64, o: i64| (v - o).clamp(0, TILE) as i32;
                clip[slot as usize] =
                    [lo(x, ox), lo(y, oy), lo(x + i64::from(width), ox), lo(y + i64::from(height), oy)];
            }
        }
        queue.write_buffer(&self.active_buf, 0, bytemuck::cast_slice(&active));
        queue.write_buffer(&self.links_buf, 0, bytemuck::cast_slice(&links));
        queue.write_buffer(&self.clip_buf, 0, bytemuck::cast_slice(&clip));
    }

    /// Advance `generations`, in batches of at most [`BATCH`], the host resizing the tile set
    /// between batches. Stops early with [`LifeGpuError::PoolFull`] if the universe outgrows the
    /// pool; the GPU state is then exact as of [`LifeGpu::generation`].
    pub fn step(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, generations: u64) -> Result<StepReport, LifeGpuError> {
        let mut report = StepReport::default();
        let mut left = generations;
        while left > 0 {
            let k = left.min(u64::from(BATCH)) as u32;
            self.run_batch(device, queue, k)?;
            left -= u64::from(k);
            report.generations += u64::from(k);
            report.batches += 1;
            report.tiles = self.active.len();
            self.resize(queue)?;
        }
        Ok(report)
    }

    fn run_batch(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, k: u32) -> Result<(), LifeGpuError> {
        let n = self.active.len() as u32;
        let table = self.rule.table_words();
        let mut bg = self.background;
        let mut gens = vec![0u8; (GEN_STRIDE * u64::from(k)) as usize];
        for g in 0..k as usize {
            let next = if self.topology == Topology::Plane { self.rule.next_background(bg) } else { 0 };
            let mut words = [0u32; 20];
            words[..16].copy_from_slice(&table);
            words[16] = u32::from(self.rule.states());
            words[17] = u32::from(bg);
            words[18] = u32::from(next);
            words[19] = n;
            let at = g * GEN_STRIDE as usize;
            gens[at..at + GEN_SIZE as usize].copy_from_slice(bytemuck::cast_slice(&words));
            bg = next;
        }
        queue.write_buffer(&self.gens, 0, &gens);
        queue.write_buffer(&self.stats_uniform, 0, bytemuck::cast_slice(&[u32::from(bg), n, 0, 0]));

        let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("life.batch") });
        let mut src = self.cur;
        if n > 0 {
            for g in 0..k {
                let mut pass = enc.begin_compute_pass(&wgpu::ComputePassDescriptor { label: Some("life.step"), timestamp_writes: None });
                pass.set_pipeline(&self.step_pipeline);
                pass.set_bind_group(0, &self.step_groups[src], &[(u64::from(g) * GEN_STRIDE) as u32]);
                pass.dispatch_workgroups(16, n, 1);
                drop(pass);
                src = 1 - src;
            }
            let mut pass = enc.begin_compute_pass(&wgpu::ComputePassDescriptor { label: Some("life.stats"), timestamp_writes: None });
            pass.set_pipeline(&self.stats_pipeline);
            pass.set_bind_group(0, &self.stats_groups[src], &[]);
            pass.dispatch_workgroups(n, 1, 1);
            drop(pass);
            enc.copy_buffer_to_buffer(&self.population_buf, 0, &self.readback, 0, 4 * u64::from(n));
        }
        let at = 4 * u64::from(self.capacity);
        enc.copy_buffer_to_buffer(&self.breach_buf, 0, &self.readback, at, 16);
        queue.submit([enc.finish()]);

        let words = read_back(device, &self.readback, at + 16)?;
        self.population = words[..n as usize].to_vec();
        self.breaches = u64::from(words[(at / 4) as usize]);
        self.cur = src;
        self.background = bg;
        self.generation += u64::from(k);
        Ok(())
    }

    /// After a batch: keep each occupied tile and its halo, free the rest, give new tiles the
    /// background's cells.
    fn resize(&mut self, queue: &wgpu::Queue) -> Result<(), LifeGpuError> {
        if self.topology != Topology::Plane {
            return Ok(());
        }
        let occupied: Vec<(i64, i64)> = self
            .active
            .iter()
            .zip(&self.population)
            .filter(|&(_, &p)| p > 0)
            .map(|(&s, _)| self.coord_of[s as usize].expect("an active slot has a tile"))
            .collect();
        let wanted = self.wanted(&occupied);
        if wanted.len() > self.capacity as usize {
            return Err(LifeGpuError::PoolFull { needed: wanted.len(), capacity: self.capacity });
        }
        let keep: HashSet<(i64, i64)> = wanted.iter().copied().collect();
        let mut pop_of: HashMap<u32, u32> = self.active.iter().copied().zip(self.population.iter().copied()).collect();
        for &slot in &self.active {
            let key = self.coord_of[slot as usize].expect("an active slot has a tile");
            if !keep.contains(&key) {
                self.slot_of.remove(&key);
                self.coord_of[slot as usize] = None;
                self.free.push(slot);
            }
        }
        let blank = [self.background; TILE_CELLS];
        self.active.clear();
        self.population.clear();
        for key in wanted {
            let slot = match self.slot_of.get(&key) {
                Some(&s) => s,
                None => {
                    let s = self.free.pop().expect("capacity checked");
                    self.slot_of.insert(key, s);
                    self.coord_of[s as usize] = Some(key);
                    queue.write_buffer(&self.pools[self.cur], u64::from(s) * TILE_BYTES, &blank);
                    pop_of.insert(s, 0);
                    s
                }
            };
            self.active.push(slot);
            self.population.push(pop_of.get(&slot).copied().unwrap_or(0));
        }
        self.upload_tables(queue);
        Ok(())
    }

    /// The universe as the core's [`Universe`]: what saving, the CPU twin and the self-test read.
    pub fn download(&self, device: &wgpu::Device, queue: &wgpu::Queue) -> Result<Universe, LifeGpuError> {
        let mut u = Universe::new(self.rule.clone(), self.topology).map_err(LifeGpuError::Readback)?;
        if self.background != 0 && !u.set_background(self.background) {
            return Err(LifeGpuError::Readback("the background cannot be represented".into()));
        }
        u.set_generation(self.generation);
        let n = self.active.len() as u64;
        if n == 0 {
            return Ok(u);
        }
        let staging = buffer(device, "life.download", n * TILE_BYTES, wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST);
        let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("life.download") });
        for (i, &slot) in self.active.iter().enumerate() {
            enc.copy_buffer_to_buffer(&self.pools[self.cur], u64::from(slot) * TILE_BYTES, &staging, i as u64 * TILE_BYTES, TILE_BYTES);
        }
        queue.submit([enc.finish()]);
        let words = read_back(device, &staging, n * TILE_BYTES)?;
        let bytes: &[u8] = bytemuck::cast_slice(&words);
        for (i, &slot) in self.active.iter().enumerate() {
            let key = self.coord_of[slot as usize].expect("an active slot has a tile");
            let tile: &[u8; TILE_CELLS] = bytes[i * TILE_CELLS..(i + 1) * TILE_CELLS].try_into().expect("a tile");
            u.put_tile(key, tile);
        }
        Ok(u)
    }
}

impl LifeGpu {
    /// The slot of the tile at `key` — wrapped on a torus — or `None` when it is not stored.
    fn slot_for_tile(&self, key: (i64, i64)) -> Option<u32> {
        let key = match self.topology {
            Topology::Torus { width, height } => {
                (key.0.rem_euclid(i64::from(width) / TILE), key.1.rem_euclid(i64::from(height) / TILE))
            }
            _ => key,
        };
        self.slot_of.get(&key).copied()
    }

    /// Each stored tile with its count of cells that differ from the background.
    fn tile_populations(&self) -> impl Iterator<Item = ((i64, i64), u32)> + '_ {
        self.active
            .iter()
            .zip(&self.population)
            .map(|(&s, &p)| (self.coord_of[s as usize].expect("an active slot has a tile"), p))
    }
}

/// What the app asks of the GPU universe on a frame (`MandelbrotParams::life`).
#[derive(Clone)]
pub struct LifeFrame {
    /// Replace the GPU universe with [`LifeFrame::load`] when this changes: a new pattern, an edit,
    /// a reset. The app bumps it; the GPU side applies each id once.
    pub load_id: u64,
    pub load: Arc<Universe>,
    /// The generation the app wants on screen: the GPU steps towards it, at most
    /// [`LifeFrame::max_steps`] a frame. A TARGET rather than a count, so a frame egui lays out
    /// twice (and paints once) cannot lose or double the generations it asked for.
    pub target: u64,
    pub max_steps: u64,
    /// Which cells the view shows.
    pub window: fractadyne_core::life::CellWindow,
    /// Where the GPU universe reports back.
    pub status: Arc<Mutex<LifeStatus>>,
    /// Download the universe into [`LifeStatus::downloaded`] this frame (saving, editing).
    pub download: bool,
}

/// What the GPU universe reports to the app.
#[derive(Clone, Default)]
pub struct LifeStatus {
    /// The device runs compute shaders; `false` (a GL adapter) means Life cannot run here.
    pub available: bool,
    /// The last [`LifeFrame::load_id`] applied.
    pub load_id: u64,
    pub generation: u64,
    pub population: u64,
    pub tiles: usize,
    pub capacity: u32,
    pub breaches: u64,
    /// Why stepping stopped (the pool is full); the app pauses and shows it.
    pub error: Option<String>,
    /// The universe, when a download was asked for.
    pub downloaded: Option<Universe>,
}

/// The `LifeView` uniform of `life_display.wgsl`.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct LifeViewU {
    origin: [f32; 2],
    step: f32,
    coarse: u32,
    grid: [u32; 2],
    bin: f32,
    background: u32,
    states: u32,
    _pad: [u32; 3],
}

/// Fine grids (a slot per tile) up to this many entries; past it the display bins densities.
const MAX_FINE_GRID: u64 = 1 << 16;
/// Coarse grids hold at most this many bins.
const MAX_COARSE_GRID: u64 = 1 << 16;

const DISPLAY_SOURCE: &str = include_str!("life_display.wgsl");

/// The GPU universe plus its display pass, owned by the renderer (created on the first Life frame).
pub(crate) struct LifeRenderer {
    gpu: LifeGpu,
    pipeline: wgpu::RenderPipeline,
    bgl: wgpu::BindGroupLayout,
    uniform: wgpu::Buffer,
    grid: wgpu::Buffer,
    grid_cap: u64,
    /// Display bind groups, one per pool (indexed like `LifeGpu::pools`).
    groups: [wgpu::BindGroup; 2],
    load_id: Option<u64>,
    /// Bumped whenever the cells change: part of the display key.
    changes: u64,
}

/// Whether this device can run the Life stepper: compute shaders with seven storage buffers.
pub fn life_available(device: &wgpu::Device) -> bool {
    let l = device.limits();
    l.max_compute_workgroups_per_dimension >= MAX_TILES
        && l.max_compute_invocations_per_workgroup >= 64
        && l.max_storage_buffers_per_shader_stage >= 7
}

/// The tile pool a live view asks for: 256 MiB a pool, within the device's limits.
const LIVE_POOL_BYTES: u64 = 256 << 20;

impl LifeRenderer {
    pub(crate) fn new(device: &wgpu::Device, iter_bgl: &wgpu::BindGroupLayout) -> LifeRenderer {
        Self::with_capacity(device, iter_bgl, pool_capacity(device, LIVE_POOL_BYTES))
    }

    /// With a pool of `capacity` tiles (the checks use small ones).
    pub(crate) fn with_capacity(device: &wgpu::Device, iter_bgl: &wgpu::BindGroupLayout, capacity: u32) -> LifeRenderer {
        let gpu = LifeGpu::new(device, capacity);
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("life_display.wgsl"),
            source: wgpu::ShaderSource::Wgsl(DISPLAY_SOURCE.into()),
        });
        let frag = |binding, ty| wgpu::BindGroupLayoutEntry { binding, visibility: wgpu::ShaderStages::FRAGMENT, ty, count: None };
        let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("life.display.layout"),
            entries: &[
                frag(0, wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None }),
                frag(1, storage(true)),
                frag(2, storage(true)),
            ],
        });
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("life.display"),
            bind_group_layouts: &[iter_bgl, &bgl],
            push_constant_ranges: &[],
        });
        let pipeline =
            crate::fullscreen_pipeline(device, &module, &layout, "fs_life", &[crate::ITER_FORMAT, crate::ITER_FORMAT], "life.display");
        use wgpu::BufferUsages as U;
        let uniform = buffer(device, "life.view", std::mem::size_of::<LifeViewU>() as u64, U::UNIFORM | U::COPY_DST);
        let grid_cap = 1024;
        let grid = buffer(device, "life.grid", 4 * grid_cap, U::STORAGE | U::COPY_DST);
        let groups = Self::make_groups(device, &bgl, &uniform, &gpu, &grid);
        LifeRenderer { gpu, pipeline, bgl, uniform, grid, grid_cap, groups, load_id: None, changes: 0 }
    }

    fn make_groups(
        device: &wgpu::Device,
        bgl: &wgpu::BindGroupLayout,
        uniform: &wgpu::Buffer,
        gpu: &LifeGpu,
        grid: &wgpu::Buffer,
    ) -> [wgpu::BindGroup; 2] {
        [0, 1].map(|i| {
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("life.display.group"),
                layout: bgl,
                entries: &[
                    wgpu::BindGroupEntry { binding: 0, resource: uniform.as_entire_binding() },
                    wgpu::BindGroupEntry { binding: 1, resource: gpu.pools[i].as_entire_binding() },
                    wgpu::BindGroupEntry { binding: 2, resource: grid.as_entire_binding() },
                ],
            })
        })
    }

    /// Apply this frame's commands — load, step, download — publish the status, and prepare the
    /// display of `size` texels at `ss` texels a pixel. Returns the display key: it changes
    /// whenever what the display pass would draw changes.
    pub(crate) fn update(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, frame: &LifeFrame, size: [u32; 2], ss: u32) -> u64 {
        let mut error = None;
        if self.load_id != Some(frame.load_id) {
            if let Err(e) = self.gpu.load(queue, &frame.load) {
                error = Some(e.to_string());
            }
            self.load_id = Some(frame.load_id);
            self.changes += 1;
        }
        let steps = frame.target.saturating_sub(self.gpu.generation()).min(frame.max_steps);
        if steps > 0 && error.is_none() {
            match self.gpu.step(device, queue, steps) {
                Ok(r) => self.changes += u64::from(r.generations > 0),
                Err(e) => {
                    self.changes += 1;
                    error = Some(e.to_string());
                }
            }
        }
        let downloaded = if frame.download { self.gpu.download(device, queue).ok() } else { None };
        if let Ok(mut s) = frame.status.lock() {
            s.available = true;
            s.load_id = frame.load_id;
            s.generation = self.gpu.generation();
            s.population = self.gpu.population();
            s.tiles = self.gpu.tile_count();
            s.capacity = self.gpu.capacity();
            s.breaches = self.gpu.breaches();
            if error.is_some() {
                s.error = error;
            }
            if downloaded.is_some() {
                s.downloaded = downloaded;
            }
        }
        self.prepare_display(device, queue, frame, size, ss)
    }

    fn prepare_display(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, frame: &LifeFrame, size: [u32; 2], ss: u32) -> u64 {
        let w = &frame.window;
        let step = w.cells_per_px / f64::from(ss.max(1));
        let span = [f64::from(size[0]) * step, f64::from(size[1]) * step];
        let tiles = |o: f64, s: f64| ((o + s) / TILE as f64).ceil().max(1.0) as u64;
        let (gx, gy) = (tiles(w.origin[0], span[0]), tiles(w.origin[1], span[1]));
        let bg = self.gpu.background();
        let mut u = LifeViewU {
            origin: [w.origin[0] as f32, w.origin[1] as f32],
            step: step as f32,
            coarse: 0,
            grid: [gx as u32, gy as u32],
            bin: TILE as f32,
            background: u32::from(bg),
            states: u32::from(self.gpu.rule().states()),
            _pad: [0; 3],
        };
        let entries: Vec<i32> = if step <= 8.0 && gx * gy <= MAX_FINE_GRID {
            let mut v = vec![-1i32; (gx * gy) as usize];
            for j in 0..gy {
                for i in 0..gx {
                    if let Some(s) = self.gpu.slot_for_tile((w.tile_x0 + i as i64, w.tile_y0 + j as i64)) {
                        v[(j * gx + i) as usize] = s as i32;
                    }
                }
            }
            v
        } else {
            // Bins of 64·2^k cells, few enough to fill from the stored tiles' populations.
            let mut k = 0u32;
            let bins = |k: u32| {
                let b = (TILE as f64) * f64::from(1u32 << k);
                (((w.origin[0] + span[0]) / b).ceil() as u64, ((w.origin[1] + span[1]) / b).ceil() as u64)
            };
            while k < 30 && {
                let (bx, by) = bins(k);
                bx * by > MAX_COARSE_GRID
            } {
                k += 1;
            }
            let (bx, by) = bins(k);
            let per = 1i64 << k; // tiles a bin side
            let empty = if bg == 1 { 1.0f32 } else { 0.0 };
            let mut sum = vec![0f64; (bx * by) as usize];
            let mut stored = vec![0u32; (bx * by) as usize];
            for ((tx, ty), pop) in self.gpu.tile_populations() {
                let (i, j) = ((tx - w.tile_x0).div_euclid(per), (ty - w.tile_y0).div_euclid(per));
                if i < 0 || j < 0 || i as u64 >= bx || j as u64 >= by {
                    continue;
                }
                let at = (j as u64 * bx + i as u64) as usize;
                let live = if bg == 1 { TILE_CELLS as u32 - pop } else { pop };
                sum[at] += f64::from(live);
                stored[at] += 1;
            }
            let cells = (TILE_CELLS as f64) * (per * per) as f64;
            u.coarse = 1;
            u.grid = [bx as u32, by as u32];
            u.bin = (TILE * per) as f32;
            sum.iter()
                .zip(&stored)
                .map(|(&s, &n)| {
                    // Unstored tiles in the bin are all background.
                    let unstored = (per * per) as f64 - f64::from(n);
                    let d = (s + unstored * f64::from(empty) * TILE_CELLS as f64) / cells;
                    (d as f32).to_bits() as i32
                })
                .collect()
        };
        if entries.len() as u64 > self.grid_cap {
            self.grid_cap = (entries.len() as u64).next_power_of_two();
            self.grid = buffer(device, "life.grid", 4 * self.grid_cap, wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST);
            self.groups = Self::make_groups(device, &self.bgl, &self.uniform, &self.gpu, &self.grid);
        }
        queue.write_buffer(&self.grid, 0, bytemuck::cast_slice(&entries));
        queue.write_buffer(&self.uniform, 0, bytemuck::bytes_of(&u));
        // The key: the cells, the mapping, the texture.
        let mut h = std::collections::hash_map::DefaultHasher::new();
        use std::hash::{Hash, Hasher};
        (self.changes, w.tile_x0, w.tile_y0, w.origin[0].to_bits(), w.origin[1].to_bits(), step.to_bits(), size, ss).hash(&mut h);
        h.finish()
    }

    /// Draw the display pass: bind group 0 is the view's iterate group (for the counters).
    pub(crate) fn draw<'a>(&'a self, pass: &mut wgpu::RenderPass<'a>, iter_bg: &'a wgpu::BindGroup) {
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, iter_bg, &[]);
        pass.set_bind_group(1, &self.groups[self.gpu.cur], &[]);
        pass.draw(0..3, 0..1);
    }
}

/// Map `buffer`'s first `len` bytes and copy them out as words.
fn read_back(device: &wgpu::Device, buffer: &wgpu::Buffer, len: u64) -> Result<Vec<u32>, LifeGpuError> {
    let slice = buffer.slice(..len);
    let (tx, rx) = std::sync::mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |r| {
        let _ = tx.send(r);
    });
    let _ = device.poll(wgpu::Maintain::Wait);
    match rx.recv() {
        Ok(Ok(())) => {}
        Ok(Err(e)) => return Err(LifeGpuError::Readback(e.to_string())),
        Err(e) => return Err(LifeGpuError::Readback(e.to_string())),
    }
    let words = bytemuck::cast_slice::<u8, u32>(&slice.get_mapped_range()).to_vec();
    buffer.unmap();
    Ok(words)
}

/// Device checks (the self-test's `life` rows).
pub mod check;

#[cfg(test)]
mod tests;
