// mercator.rs — Network Mercator Projection
// Renders the ENTIRE Gaia network as a 3D world map using the KenshiReborn engine.
// Two houses = two continents. Gates = cities. Connections = roads.
// The network IS the game world.
//
// The world (zones, nodes, links) is data: world/mercator.json, built into the engine and
// replaceable with --world <file>. Which gates are alive is never written down. The engine
// asks the mesh — what a gate answers at /api/mesh about itself and the other bodies — and
// asks again every few seconds while the window is open, so the map follows the house.
//
//   --mercator                          open the map; it follows the mesh while it is open
//   --mercator --snapshot <file.png>    draw it once without a window and write the picture,
//                                       and beside it <file.json>: where each name lands in it
//       --angle <degrees>               where the camera stands for the snapshot (default 150)
//   --world <file>                      another world file
//   --mesh <host:port>                  the gate to ask (default: this machine's own gate)

use std::collections::{BTreeMap, HashMap};
use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

use serde::Deserialize;

use crate::mesh3d::MeshData;
use crate::render3d::{self, Layout, Placed, SceneUpdate, TINT_FLAT, TINT_LIT};

const DEFAULT_WORLD: &str = include_str!("../world/mercator.json");
/// Where a gate answers on its own machine (the Windows body, the Linux body).
const LOCAL_GATES: [&str; 2] = ["127.0.0.1:8080", "127.0.0.1:8443"];
const POLL: Duration = Duration::from_secs(10);
const SNAPSHOT_SIZE: (u32, u32) = (1600, 900);
const SNAPSHOT_ANGLE_DEG: f32 = 150.0;
/// A node's `size` is the old fitted size; this turns it into the radius of its unit shape.
const NODE_RADIUS: f32 = 0.55;

// === THE WORLD FILE ===

#[derive(Deserialize)]
struct World {
    #[serde(default)]
    zones: Vec<Zone>,
    nodes: Vec<Node>,
    #[serde(default)]
    links: Vec<Link>,
}

#[derive(Deserialize)]
struct Zone {
    name: String,
    center: [f32; 3],
    size: [f32; 2],
    color: [f32; 3],
}

#[derive(Deserialize, Clone, Copy, PartialEq, Debug)]
#[serde(rename_all = "lowercase")]
enum Kind { Gate, Infra }

#[derive(Deserialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
enum Shape { Sphere, Cube, Octahedron, Diamond }

#[derive(Deserialize)]
struct Node {
    name: String,
    kind: Kind,
    #[serde(default)]
    zone: Option<String>,
    shape: Shape,
    size: f32,
    pos: [f32; 3],
    color: [f32; 3],
}

#[derive(Deserialize)]
#[serde(untagged)]
enum End { Node(String), Point([f32; 3]) }

#[derive(Deserialize)]
struct Link {
    name: String,
    from: End,
    to: End,
    color: [f32; 3],
}

fn read_world(text: &str) -> Result<World, String> {
    let world: World = serde_json::from_str(text).map_err(|e| e.to_string())?;
    let mut seen = std::collections::HashSet::new();
    for n in &world.nodes {
        if !seen.insert(n.name.as_str()) {
            return Err(format!("two nodes are named {}", n.name));
        }
    }
    for l in &world.links {
        for end in [&l.from, &l.to] {
            if let End::Node(name) = end {
                if !seen.contains(name.as_str()) {
                    return Err(format!("link {} ends at {}, which is not a node", l.name, name));
                }
            }
        }
    }
    Ok(world)
}

fn load_world(path: Option<&str>) -> Result<World, String> {
    match path {
        None => read_world(DEFAULT_WORLD),
        Some(p) => {
            let text = std::fs::read_to_string(p).map_err(|e| format!("{p}: {e}"))?;
            read_world(&text).map_err(|e| format!("{p}: {e}"))
        }
    }
}

// === THE MESH ===

/// One body of the mesh: what it says of itself, or what the asked gate says of it.
#[derive(Clone, PartialEq, Debug, Default)]
struct Body {
    answers: bool,
    tip: String,
    up: Vec<String>,
    down: Vec<String>,
}

/// The mesh as one gate tells it. `asked` is the gate that answered; None when none did.
#[derive(Clone, PartialEq, Debug, Default)]
struct Mesh {
    asked: Option<String>,
    bodies: BTreeMap<String, Body>,
}

/// How a gate stands in the mesh.
#[derive(Clone, Copy, PartialEq, Debug)]
enum Standing {
    /// A body of the mesh, and it answers.
    Answers,
    /// A body of the mesh that did not answer.
    Silent,
    /// Not a body of the mesh: the mesh says nothing about it.
    Outside,
    /// No gate could be asked.
    Unknown,
}

impl Mesh {
    fn standing(&self, name: &str) -> Standing {
        if self.asked.is_none() {
            return Standing::Unknown;
        }
        match self.bodies.get(name) {
            Some(b) if b.answers => Standing::Answers,
            Some(_) => Standing::Silent,
            None => Standing::Outside,
        }
    }
}

fn names(v: Option<&serde_json::Value>) -> Vec<String> {
    v.and_then(|x| x.as_array())
        .map(|a| a.iter().filter_map(|s| s.as_str().map(str::to_string)).collect())
        .unwrap_or_default()
}

/// Read a gate's answer at /api/mesh: itself, and the peers it asked.
fn read_mesh(text: &str) -> Result<BTreeMap<String, Body>, String> {
    let v: serde_json::Value = serde_json::from_str(text).map_err(|e| e.to_string())?;
    let me = v.get("self").ok_or("the answer has no self")?;
    let body = |b: &serde_json::Value, answers: bool| -> Option<(String, Body)> {
        let name = b.get("name")?.as_str()?.to_string();
        let pi = b.get("pi_body");
        Some((name, Body {
            answers,
            tip: b.get("tip").and_then(|t| t.as_str()).unwrap_or("").to_string(),
            up: names(pi.and_then(|p| p.get("up"))),
            down: names(pi.and_then(|p| p.get("down"))),
        }))
    };
    let mut bodies = BTreeMap::new();
    let (name, mine) = body(me, true).ok_or("the answering gate has no name")?;
    bodies.insert(name, mine);
    for p in v.get("peers").and_then(|p| p.as_array()).map(Vec::as_slice).unwrap_or(&[]) {
        let answers = p.get("answers").and_then(|a| a.as_bool()).unwrap_or(false);
        if let Some((name, peer)) = body(p, answers) {
            bodies.insert(name, peer);
        }
    }
    Ok(bodies)
}

/// The body of an HTTP answer, when its status is 200.
fn body_of(answer: &[u8]) -> Result<&str, String> {
    let split = answer.windows(4).position(|w| w == b"\r\n\r\n").ok_or("no end of headers")?;
    let head = std::str::from_utf8(&answer[..split]).map_err(|e| e.to_string())?;
    let status = head.lines().next().unwrap_or("");
    if status.split_whitespace().nth(1) != Some("200") {
        return Err(format!("the gate said: {status}"));
    }
    std::str::from_utf8(&answer[split + 4..]).map_err(|e| e.to_string())
}

fn http_get(addr: &str, path: &str) -> Result<String, String> {
    let sock = addr.to_socket_addrs().map_err(|e| e.to_string())?.next().ok_or("no address")?;
    let mut stream = TcpStream::connect_timeout(&sock, Duration::from_millis(1500)).map_err(|e| e.to_string())?;
    // A gate whose own view of its peers is stale asks them before it answers.
    stream.set_read_timeout(Some(Duration::from_secs(12))).map_err(|e| e.to_string())?;
    write!(stream, "GET {path} HTTP/1.0\r\nHost: {addr}\r\nAccept: application/json\r\nConnection: close\r\n\r\n")
        .map_err(|e| e.to_string())?;
    let mut answer = Vec::new();
    stream.read_to_end(&mut answer).map_err(|e| e.to_string())?;
    body_of(&answer).map(str::to_string)
}

/// Ask a gate about the mesh: the one given, or this machine's own.
fn ask_mesh(given: Option<&str>) -> Mesh {
    let gates: Vec<&str> = match given {
        Some(g) => vec![g],
        None => LOCAL_GATES.to_vec(),
    };
    for addr in gates {
        if let Ok(bodies) = http_get(addr, "/api/mesh").and_then(|text| read_mesh(&text)) {
            return Mesh { asked: Some(addr.to_string()), bodies };
        }
    }
    Mesh::default()
}

// === THE SCENE ===

/// What the map shows, in numbers.
#[derive(Clone, PartialEq, Debug, Default)]
struct Summary {
    gates: usize,
    answering: usize,
    silent: usize,
    outside: usize,
    /// Do all answering bodies stand on the same commit? None with fewer than two.
    level: Option<bool>,
    primals_up: usize,
    primals_down: usize,
    /// Bodies the mesh names that the world file does not have.
    unplaced: Vec<String>,
}

fn gate_color(own: [f32; 3], standing: Standing) -> [f32; 3] {
    match standing {
        Standing::Answers => own,
        Standing::Silent => [0.55, 0.10, 0.10],
        Standing::Outside | Standing::Unknown => [0.20 + own[0] * 0.12, 0.20 + own[1] * 0.12, 0.20 + own[2] * 0.12],
    }
}

fn tint(c: [f32; 3], k: f32, how: f32) -> [f32; 4] {
    [c[0] * k, c[1] * k, c[2] * k, how]
}

/// A body the mesh names that the world file does not have is drawn this big,
const UNPLACED_SIZE: f32 = 1.2;

/// and stands here: the i-th of them, above the middle.
fn unplaced_pos(i: usize) -> [f32; 3] {
    let a = i as f32 * 2.4;
    [3.0 * a.cos(), 5.0, 3.0 * a.sin()]
}

fn shape_mesh(name: &str, shape: Shape) -> MeshData {
    match shape {
        Shape::Sphere => gen_sphere(name, 12, 8),
        Shape::Cube => gen_cube(name),
        Shape::Octahedron => gen_octahedron(name),
        Shape::Diamond => gen_diamond(name),
    }
}

/// Lay the world out as it stands in the mesh: zones, wires, nodes, the mesh's own links
/// between the bodies that answer, and each answering gate's primals around it.
fn build(world: &World, mesh: &Mesh) -> (Vec<Placed>, Vec<[f32; 4]>, Summary) {
    let mut placed: Vec<Placed> = Vec::new();
    let mut tints: Vec<[f32; 4]> = Vec::new();
    let mut sum = Summary::default();

    for z in &world.zones {
        placed.push((gen_plane(&z.name, z.size[0], z.size[1]), z.center, 1.0));
        tints.push(tint(z.color, 0.55, TINT_FLAT));
    }

    // Where every gate stands, and where it is.
    let mut at: HashMap<&str, ([f32; 3], f32, Option<Standing>)> = HashMap::new();
    for n in &world.nodes {
        let standing = (n.kind == Kind::Gate).then(|| mesh.standing(&n.name));
        at.insert(n.name.as_str(), (n.pos, n.size * NODE_RADIUS, standing));
    }
    // A body the mesh names that the world file does not have still gets a place: above the middle.
    let extra: Vec<&String> = mesh.bodies.keys().filter(|name| !at.contains_key(name.as_str())).collect();
    for (i, name) in extra.iter().enumerate() {
        at.insert(name.as_str(), (unplaced_pos(i), UNPLACED_SIZE * NODE_RADIUS, Some(mesh.standing(name))));
        sum.unplaced.push((*name).clone());
    }

    for l in &world.links {
        let mut touched: Vec<Standing> = Vec::new();
        let mut point = |end: &End| -> [f32; 3] {
            match end {
                End::Point(p) => *p,
                End::Node(name) => {
                    let (pos, _, standing) = at[name.as_str()];
                    touched.extend(standing);
                    pos
                }
            }
        };
        let (from, to) = (point(&l.from), point(&l.to));
        if from == to {
            continue;
        }
        // A wire is lit by a gate that answers, dim at a gate that does not, plain between machines.
        let k = if touched.contains(&Standing::Answers) { 1.0 } else if touched.is_empty() { 0.6 } else { 0.25 };
        placed.push((gen_wire(from, to, 0.05), [0.0; 3], 1.0));
        tints.push(tint(l.color, k, TINT_FLAT));
    }

    for n in &world.nodes {
        let color = match at[n.name.as_str()].2 {
            Some(standing) => gate_color(n.color, standing),
            None => n.color,
        };
        placed.push((shape_mesh(&n.name, n.shape), n.pos, n.size * NODE_RADIUS));
        tints.push(tint(color, 1.0, TINT_LIT));
    }
    for name in &extra {
        let (pos, radius, standing) = at[name.as_str()];
        placed.push((gen_diamond(name), pos, radius));
        tints.push(tint(gate_color([0.9, 0.9, 0.9], standing.unwrap_or(Standing::Unknown)), 1.0, TINT_LIT));
    }

    for (_, _, standing) in at.values() {
        match standing {
            Some(Standing::Answers) => { sum.gates += 1; sum.answering += 1; }
            Some(Standing::Silent) => { sum.gates += 1; sum.silent += 1; }
            Some(Standing::Outside) | Some(Standing::Unknown) => { sum.gates += 1; sum.outside += 1; }
            None => {}
        }
    }

    // The mesh itself: an arch between every two bodies that answer, over the wires, green when
    // they are level and amber when they stand on different commits.
    let answering: Vec<(&String, &Body)> = mesh.bodies.iter().filter(|(_, b)| b.answers).collect();
    if answering.len() > 1 {
        sum.level = Some(answering.iter().all(|(_, b)| !b.tip.is_empty() && b.tip == answering[0].1.tip));
    }
    for (i, (a_name, a)) in answering.iter().enumerate() {
        for (b_name, b) in &answering[i + 1..] {
            let (a_pos, a_r, _) = at[a_name.as_str()];
            let (b_pos, b_r, _) = at[b_name.as_str()];
            let level = !a.tip.is_empty() && a.tip == b.tip;
            let color = if level { [0.2, 1.0, 0.5] } else { [1.0, 0.7, 0.1] };
            let (a_top, b_top) = ([a_pos[0], a_pos[1] + a_r + 0.3, a_pos[2]], [b_pos[0], b_pos[1] + b_r + 0.3, b_pos[2]]);
            let crown = [(a_top[0] + b_top[0]) / 2.0, a_top[1].max(b_top[1]) + 2.0, (a_top[2] + b_top[2]) / 2.0];
            for (from, to) in [(a_top, crown), (crown, b_top)] {
                placed.push((gen_wire(from, to, 0.09), [0.0; 3], 1.0));
                tints.push(tint(color, 1.0, TINT_FLAT));
            }
        }
    }

    // The primals each answering gate says it runs: a ring of small bodies around it.
    for (name, body) in &answering {
        let (pos, radius, _) = at[name.as_str()];
        let count = body.up.len() + body.down.len();
        sum.primals_up += body.up.len();
        sum.primals_down += body.down.len();
        for (i, primal) in body.up.iter().chain(body.down.iter()).enumerate() {
            let a = i as f32 / count as f32 * std::f32::consts::TAU;
            let ring = radius + 0.55;
            let color = if i < body.up.len() { [0.35, 1.0, 0.7] } else { [1.0, 0.25, 0.2] };
            placed.push((gen_sphere(primal, 8, 6), [pos[0] + ring * a.cos(), pos[1], pos[2] + ring * a.sin()], 0.16));
            tints.push(tint(color, 1.0, TINT_LIT));
        }
    }

    (placed, tints, sum)
}

// === THE NAMES ===

/// A name to write on a snapshot: what it is, how it stands, and the point just above it.
struct Label {
    name: String,
    kind: &'static str,
    zone: Option<String>,
    standing: &'static str,
    tip: String,
    up: Vec<String>,
    down: Vec<String>,
    pos: [f32; 3],
}

fn labels(world: &World, mesh: &Mesh) -> Vec<Label> {
    let of = |name: &str, kind: Kind, zone: Option<String>, pos: [f32; 3], radius: f32| {
        let body = mesh.bodies.get(name);
        let standing = match (kind, mesh.standing(name)) {
            (Kind::Infra, _) => "machine",
            (_, Standing::Answers) => "answers",
            (_, Standing::Silent) => "silent",
            (_, Standing::Outside) => "outside",
            (_, Standing::Unknown) => "unknown",
        };
        Label {
            name: name.to_string(),
            kind: if kind == Kind::Gate { "gate" } else { "infra" },
            zone,
            standing,
            tip: body.map(|b| b.tip.clone()).unwrap_or_default(),
            up: body.map(|b| b.up.clone()).unwrap_or_default(),
            down: body.map(|b| b.down.clone()).unwrap_or_default(),
            pos: [pos[0], pos[1] + radius, pos[2]],
        }
    };
    let mut out: Vec<Label> = world.nodes.iter()
        .map(|n| of(&n.name, n.kind, n.zone.clone(), n.pos, n.size * NODE_RADIUS))
        .collect();
    let extra = mesh.bodies.keys().filter(|name| !world.nodes.iter().any(|n| &n.name == *name));
    out.extend(extra.enumerate().map(|(i, name)| of(name, Kind::Gate, None, unplaced_pos(i), UNPLACED_SIZE * NODE_RADIUS)));
    out
}

/// What a page needs to write names on a snapshot: the state of the mesh the picture was drawn
/// from, and where each name lands in it.
fn names_file(world: &World, mesh: &Mesh, sum: &Summary, placed: &[Placed], tints: &[[f32; 4]],
              size: (u32, u32), angle_deg: f32) -> serde_json::Value {
    let names = labels(world, mesh);
    let mut points: Vec<[f32; 3]> = names.iter().map(|l| l.pos).collect();
    points.extend(world.zones.iter().map(|z| z.center));
    let at = render3d::project(placed, tints, Layout::World, size.0, size.1, angle_deg.to_radians(), &points);
    let spot = |i: usize| at[i].map(|p| {
        let r = |v: f32| (v as f64 * 10000.0).round() / 10000.0;
        [r(p[0]), r(p[1])]
    });
    let nodes: Vec<serde_json::Value> = names.iter().enumerate().map(|(i, l)| serde_json::json!({
        "name": l.name, "kind": l.kind, "zone": l.zone, "standing": l.standing,
        "tip": l.tip, "primals_up": l.up, "primals_down": l.down, "at": spot(i),
    })).collect();
    let zones: Vec<serde_json::Value> = world.zones.iter().enumerate().map(|(i, z)| serde_json::json!({
        "name": z.name, "at": spot(names.len() + i),
    })).collect();
    let taken = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    serde_json::json!({
        "what": "The names for the MERCATOR snapshot beside this file, and the state of the mesh it was drawn from. `at` is where a name lands in the picture: fractions of its width and height from the top left corner.",
        "taken_unix": taken,
        "size": [size.0, size.1],
        "angle": angle_deg,
        "mesh": {
            "answered": mesh.asked.is_some(), "level": sum.level,
            "gates": sum.gates, "answering": sum.answering, "silent": sum.silent, "outside": sum.outside,
            "primals_up": sum.primals_up, "primals_down": sum.primals_down,
        },
        "nodes": nodes,
        "zones": zones,
    })
}

fn title(mesh: &Mesh, sum: &Summary) -> String {
    let Some(asked) = &mesh.asked else {
        return format!("GAIA MERCATOR — no gate answered about the mesh · {} gates drawn unlit", sum.gates);
    };
    let level = match sum.level {
        Some(true) => " · level",
        Some(false) => " · not level",
        None => "",
    };
    let silent = if sum.silent > 0 { format!(", {} silent", sum.silent) } else { String::new() };
    let down = if sum.primals_down > 0 { format!(", {} down", sum.primals_down) } else { String::new() };
    format!("GAIA MERCATOR — {} of {} gates answer in the mesh{}{} · {} primals up{} · asked {}",
        sum.answering, sum.gates, silent, level, sum.primals_up, down, asked)
}

fn report(world: &World, mesh: &Mesh, sum: &Summary, given: Option<&str>) {
    match &mesh.asked {
        Some(asked) => println!("◆ THE MESH, as {} tells it\n", asked),
        None => println!("◆ THE MESH: no gate answered ({}). Every gate is drawn unlit.\n",
            given.map(str::to_string).unwrap_or_else(|| LOCAL_GATES.join(", "))),
    }
    for n in &world.nodes {
        let state = match (n.kind, mesh.standing(&n.name), mesh.bodies.get(&n.name)) {
            (Kind::Infra, _, _) => "machine".to_string(),
            (_, Standing::Answers, Some(b)) => {
                let primals = if b.up.is_empty() && b.down.is_empty() { String::new() }
                    else { format!(" · {} primals up, {} down", b.up.len(), b.down.len()) };
                format!("ANSWERS · tip {}{}", b.tip, primals)
            }
            (_, Standing::Silent, _) => "SILENT (a body of the mesh that did not answer)".to_string(),
            (_, Standing::Outside, _) => "not a body of the mesh".to_string(),
            _ => "unknown".to_string(),
        };
        println!("  {:<13} {:<8} {}", n.name, n.zone.as_deref().unwrap_or("roaming"), state);
    }
    for name in &sum.unplaced {
        println!("  {:<13} {:<8} named by the mesh, not in the world file: placed above the middle", name, "-");
    }
    println!("\n  {}\n", title(mesh, sum));
}

// === PROCEDURAL GEOMETRY GENERATORS ===

fn gen_sphere(name: &str, slices: u32, stacks: u32) -> MeshData {
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut indices = Vec::new();

    for i in 0..=stacks {
        let phi = std::f32::consts::PI * i as f32 / stacks as f32;
        for j in 0..=slices {
            let theta = 2.0 * std::f32::consts::PI * j as f32 / slices as f32;
            let x = phi.sin() * theta.cos();
            let y = phi.cos();
            let z = phi.sin() * theta.sin();
            positions.push([x, y, z]);
            normals.push([x, y, z]);
        }
    }
    for i in 0..stacks {
        for j in 0..slices {
            let a = i * (slices + 1) + j;
            let b = a + slices + 1;
            indices.push(a); indices.push(b); indices.push(a + 1);
            indices.push(a + 1); indices.push(b); indices.push(b + 1);
        }
    }
    MeshData { name: name.to_string(), positions, normals, indices }
}

fn gen_cube(name: &str) -> MeshData {
    let p: Vec<[f32;3]> = vec![
        [-1.0,-1.0,-1.0],[1.0,-1.0,-1.0],[1.0,1.0,-1.0],[-1.0,1.0,-1.0],
        [-1.0,-1.0,1.0],[1.0,-1.0,1.0],[1.0,1.0,1.0],[-1.0,1.0,1.0],
    ];
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut indices = Vec::new();
    let faces: [(usize,usize,usize,usize,[f32;3]);6] = [
        (0,1,2,3,[0.0,0.0,-1.0]),(5,4,7,6,[0.0,0.0,1.0]),
        (4,0,3,7,[-1.0,0.0,0.0]),(1,5,6,2,[1.0,0.0,0.0]),
        (3,2,6,7,[0.0,1.0,0.0]),(4,5,1,0,[0.0,-1.0,0.0]),
    ];
    for (a,b,c,d,n) in &faces {
        let base = positions.len() as u32;
        positions.extend_from_slice(&[p[*a],p[*b],p[*c],p[*d]]);
        normals.extend_from_slice(&[*n,*n,*n,*n]);
        indices.extend_from_slice(&[base,base+1,base+2,base,base+2,base+3]);
    }
    MeshData { name: name.to_string(), positions, normals, indices }
}

fn gen_octahedron(name: &str) -> MeshData {
    let p = [[1.0,0.0,0.0],[-1.0,0.0,0.0],[0.0,1.0,0.0],[0.0,-1.0,0.0],[0.0,0.0,1.0],[0.0,0.0,-1.0f32]];
    let tris: [[usize;3];8] = [[0,2,4],[2,1,4],[1,3,4],[3,0,4],[2,0,5],[1,2,5],[3,1,5],[0,3,5]];
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut indices = Vec::new();
    for tri in &tris {
        let base = positions.len() as u32;
        let a = p[tri[0]]; let b = p[tri[1]]; let c = p[tri[2]];
        let n = cross([b[0]-a[0],b[1]-a[1],b[2]-a[2]], [c[0]-a[0],c[1]-a[1],c[2]-a[2]]);
        let l = (n[0]*n[0]+n[1]*n[1]+n[2]*n[2]).sqrt().max(0.001);
        let nn = [n[0]/l, n[1]/l, n[2]/l];
        positions.extend_from_slice(&[a,b,c]);
        normals.extend_from_slice(&[nn,nn,nn]);
        indices.extend_from_slice(&[base,base+1,base+2]);
    }
    MeshData { name: name.to_string(), positions, normals, indices }
}

fn gen_diamond(name: &str) -> MeshData {
    // Elongated octahedron — taller Y
    let p = [[1.0,0.0,0.0],[-1.0,0.0,0.0],[0.0,1.5,0.0],[0.0,-1.5,0.0],[0.0,0.0,1.0],[0.0,0.0,-1.0f32]];
    let tris: [[usize;3];8] = [[0,2,4],[2,1,4],[1,3,4],[3,0,4],[2,0,5],[1,2,5],[3,1,5],[0,3,5]];
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut indices = Vec::new();
    for tri in &tris {
        let base = positions.len() as u32;
        let a = p[tri[0]]; let b = p[tri[1]]; let c = p[tri[2]];
        let n = cross([b[0]-a[0],b[1]-a[1],b[2]-a[2]], [c[0]-a[0],c[1]-a[1],c[2]-a[2]]);
        let l = (n[0]*n[0]+n[1]*n[1]+n[2]*n[2]).sqrt().max(0.001);
        let nn = [n[0]/l, n[1]/l, n[2]/l];
        positions.extend_from_slice(&[a,b,c]);
        normals.extend_from_slice(&[nn,nn,nn]);
        indices.extend_from_slice(&[base,base+1,base+2]);
    }
    MeshData { name: name.to_string(), positions, normals, indices }
}

fn gen_wire(from: [f32;3], to: [f32;3], thickness: f32) -> MeshData {
    // A thin rectangular prism connecting two points
    let dir = [to[0]-from[0], to[1]-from[1], to[2]-from[2]];
    let len = (dir[0]*dir[0]+dir[1]*dir[1]+dir[2]*dir[2]).sqrt();
    if len < 0.001 { return gen_cube("wire"); }

    // Find perpendicular vectors
    let d = [dir[0]/len, dir[1]/len, dir[2]/len];
    let up = if d[1].abs() < 0.9 { [0.0,1.0,0.0] } else { [1.0,0.0,0.0] };
    let right = cross(d, up);
    let rl = (right[0]*right[0]+right[1]*right[1]+right[2]*right[2]).sqrt().max(0.001);
    let r = [right[0]/rl*thickness, right[1]/rl*thickness, right[2]/rl*thickness];
    let u = cross(d, [r[0],r[1],r[2]]);
    let ul = (u[0]*u[0]+u[1]*u[1]+u[2]*u[2]).sqrt().max(0.001);
    let u = [u[0]/ul*thickness, u[1]/ul*thickness, u[2]/ul*thickness];

    let corners = [
        [from[0]-r[0]-u[0], from[1]-r[1]-u[1], from[2]-r[2]-u[2]],
        [from[0]+r[0]-u[0], from[1]+r[1]-u[1], from[2]+r[2]-u[2]],
        [from[0]+r[0]+u[0], from[1]+r[1]+u[1], from[2]+r[2]+u[2]],
        [from[0]-r[0]+u[0], from[1]-r[1]+u[1], from[2]-r[2]+u[2]],
        [to[0]-r[0]-u[0], to[1]-r[1]-u[1], to[2]-r[2]-u[2]],
        [to[0]+r[0]-u[0], to[1]+r[1]-u[1], to[2]+r[2]-u[2]],
        [to[0]+r[0]+u[0], to[1]+r[1]+u[1], to[2]+r[2]+u[2]],
        [to[0]-r[0]+u[0], to[1]-r[1]+u[1], to[2]-r[2]+u[2]],
    ];

    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut indices = Vec::new();
    let faces: [(usize,usize,usize,usize);6] = [
        (0,1,2,3),(5,4,7,6),(4,0,3,7),(1,5,6,2),(3,2,6,7),(4,5,1,0),
    ];
    for (a,b,c,dd) in &faces {
        let base = positions.len() as u32;
        let pa = corners[*a]; let pb = corners[*b]; let pc = corners[*c]; let pd = corners[*dd];
        let n = cross([pb[0]-pa[0],pb[1]-pa[1],pb[2]-pa[2]], [pc[0]-pa[0],pc[1]-pa[1],pc[2]-pa[2]]);
        let l = (n[0]*n[0]+n[1]*n[1]+n[2]*n[2]).sqrt().max(0.001);
        let nn = [n[0]/l,n[1]/l,n[2]/l];
        positions.extend_from_slice(&[pa,pb,pc,pd]);
        normals.extend_from_slice(&[nn,nn,nn,nn]);
        indices.extend_from_slice(&[base,base+1,base+2,base,base+2,base+3]);
    }
    MeshData { name: "wire".to_string(), positions, normals, indices }
}

fn gen_plane(name: &str, width: f32, depth: f32) -> MeshData {
    let hw = width / 2.0;
    let hd = depth / 2.0;
    MeshData {
        name: name.to_string(),
        positions: vec![[-hw,0.0,-hd],[hw,0.0,-hd],[hw,0.0,hd],[-hw,0.0,hd]],
        normals: vec![[0.0,1.0,0.0];4],
        indices: vec![0,1,2,0,2,3],
    }
}

fn cross(a: [f32;3], b: [f32;3]) -> [f32;3] {
    [a[1]*b[2]-a[2]*b[1], a[2]*b[0]-a[0]*b[2], a[0]*b[1]-a[1]*b[0]]
}

fn arg_value<'a>(args: &'a [String], flag: &str) -> Option<&'a str> {
    args.iter().position(|a| a == flag).and_then(|i| args.get(i + 1)).map(String::as_str)
}

pub fn run_mercator(args: &[String]) {
    println!("╔══════════════════════════════════════════════════════════╗");
    println!("║  GAIA NETWORK MERCATOR — Wave 245                      ║");
    println!("║  Two Houses · One Organism · /22 Flat Mesh             ║");
    println!("║  Vulkan · Sovereign Cartography · the map asks the mesh ║");
    println!("╚══════════════════════════════════════════════════════════╝\n");

    let world = match load_world(arg_value(args, "--world")) {
        Ok(w) => w,
        Err(e) => {
            eprintln!("the world file could not be read: {e}");
            std::process::exit(1);
        }
    };
    let given = arg_value(args, "--mesh").map(str::to_string);
    let mesh = ask_mesh(given.as_deref());
    let (meshes, tints, sum) = build(&world, &mesh);
    report(&world, &mesh, &sum, given.as_deref());

    if let Some(path) = arg_value(args, "--snapshot") {
        let angle = arg_value(args, "--angle").and_then(|a| a.parse::<f32>().ok()).unwrap_or(SNAPSHOT_ANGLE_DEG);
        let (w, h) = SNAPSHOT_SIZE;
        match render3d::snapshot(&meshes, &tints, Layout::World, w, h, angle.to_radians(), path) {
            Ok(gpu) => {
                println!("◆ SNAPSHOT {} — {}x{}, {} objects, drawn by {}", path, w, h, meshes.len(), gpu);
                let beside = std::path::Path::new(path).with_extension("json");
                let names = names_file(&world, &mesh, &sum, &meshes, &tints, (w, h), angle);
                let text = serde_json::to_string_pretty(&names).unwrap_or_default() + "\n";
                match std::fs::write(&beside, text) {
                    Ok(()) => println!("◆ NAMES    {} — where each name lands in the picture", beside.display()),
                    Err(e) => eprintln!("the names file could not be written: {e}"),
                }
            }
            Err(e) => {
                eprintln!("the snapshot could not be made: {e}");
                std::process::exit(1);
            }
        }
        return;
    }

    println!("◆ LAUNCHING MERCATOR RENDERER — {} objects, asking the mesh every {} s\n", meshes.len(), POLL.as_secs());

    // The map follows the mesh: ask again, and send the window a new scene when the answer changed.
    let (tx, rx) = std::sync::mpsc::channel();
    let first_title = title(&mesh, &sum);
    std::thread::spawn(move || {
        let mut last = mesh;
        loop {
            std::thread::sleep(POLL);
            let now = ask_mesh(given.as_deref());
            if now == last {
                continue;
            }
            let (meshes, tints, sum) = build(&world, &now);
            println!("◆ THE MESH CHANGED");
            report(&world, &now, &sum, given.as_deref());
            if tx.send(SceneUpdate { meshes, tints, title: title(&now, &sum) }).is_err() {
                break;
            }
            last = now;
        }
    });

    let event_loop = winit::event_loop::EventLoop::new().expect("event loop");
    let mut app = render3d::App3D {
        gpu: None,
        window: None,
        meshes,
        title: first_title,
        frame_count: 0,
        start: std::time::Instant::now(),
        tints,
        layout: Layout::World,
        updates: Some(rx),
    };
    event_loop.run_app(&mut app).expect("run");
}

#[cfg(test)]
mod tests {
    use super::*;

    const ANSWER: &str = r#"{
      "self": {"name": "northGate", "tip": "aaa111", "pi_body": {"up": ["beardog", "songbird"], "down": ["squirrel"]}},
      "peers": [
        {"name": "eastGate", "tip": "bbb222", "pi_body": null, "answers": true},
        {"name": "ironGate", "answers": false}
      ],
      "level": false
    }"#;

    fn mesh() -> Mesh {
        Mesh { asked: Some("test".into()), bodies: read_mesh(ANSWER).unwrap() }
    }

    #[test]
    fn every_node_has_a_name_and_a_place_in_the_snapshot() {
        let world = read_world(DEFAULT_WORLD).unwrap();
        let m = mesh();
        let (placed, tints, sum) = build(&world, &m);
        let names = names_file(&world, &m, &sum, &placed, &tints, SNAPSHOT_SIZE, SNAPSHOT_ANGLE_DEG);
        let nodes = names["nodes"].as_array().unwrap();
        assert_eq!(nodes.len(), 14);
        let standing = |name: &str| nodes.iter().find(|n| n["name"] == name).unwrap()["standing"].as_str().unwrap().to_string();
        assert_eq!(standing("northGate"), "answers");
        assert_eq!(standing("ironGate"), "silent");
        assert_eq!(standing("southGate"), "outside");
        assert_eq!(standing("Router"), "machine");
        for n in nodes.iter().chain(names["zones"].as_array().unwrap()) {
            for v in n["at"].as_array().unwrap() {
                assert!((0.0..=1.0).contains(&v.as_f64().unwrap()), "{} is outside the picture", n["name"]);
            }
        }
        assert_eq!(names["mesh"]["answering"], 2);
        assert_eq!(names["mesh"]["level"], false);
        assert!(title(&m, &sum).contains("2 of 10 gates answer in the mesh, 1 silent · not level"));
    }

    #[test]
    fn the_built_in_world_reads_and_its_links_end_at_nodes() {
        let world = read_world(DEFAULT_WORLD).unwrap();
        assert_eq!(world.nodes.iter().filter(|n| n.kind == Kind::Gate).count(), 10);
        assert_eq!(world.links.len(), 11);
        assert_eq!(world.zones.len(), 3);
    }

    #[test]
    fn a_link_to_a_node_that_is_not_there_is_refused() {
        let text = r#"{"nodes": [{"name": "a", "kind": "gate", "shape": "cube", "size": 1, "pos": [0,0,0], "color": [1,1,1]}],
                       "links": [{"name": "w", "from": "a", "to": "b", "color": [1,1,1]}]}"#;
        assert!(read_world(text).err().unwrap().contains("which is not a node"));
    }

    #[test]
    fn a_gate_stands_in_the_mesh_as_the_mesh_says() {
        let m = mesh();
        assert_eq!(m.standing("northGate"), Standing::Answers);
        assert_eq!(m.standing("eastGate"), Standing::Answers);
        assert_eq!(m.standing("ironGate"), Standing::Silent);
        assert_eq!(m.standing("southGate"), Standing::Outside);
        assert_eq!(Mesh::default().standing("northGate"), Standing::Unknown);
        assert_eq!(m.bodies["northGate"].up, ["beardog", "songbird"]);
        assert_eq!(m.bodies["northGate"].down, ["squirrel"]);
    }

    #[test]
    fn the_scene_counts_what_the_mesh_says() {
        let world = read_world(DEFAULT_WORLD).unwrap();
        let (placed, tints, sum) = build(&world, &mesh());
        assert_eq!(placed.len(), tints.len());
        assert_eq!((sum.gates, sum.answering, sum.silent, sum.outside), (10, 2, 1, 7));
        assert_eq!(sum.level, Some(false));
        assert_eq!((sum.primals_up, sum.primals_down), (2, 1));
        // 3 zones + 11 wires + 14 nodes + the two halves of 1 mesh arch + 3 primals
        assert_eq!(placed.len(), 33);
        assert!(sum.unplaced.is_empty());
    }

    #[test]
    fn with_no_gate_to_ask_every_gate_is_unlit_and_nothing_is_claimed() {
        let world = read_world(DEFAULT_WORLD).unwrap();
        let (placed, _, sum) = build(&world, &Mesh::default());
        assert_eq!((sum.answering, sum.silent, sum.outside), (0, 0, 10));
        assert_eq!(sum.level, None);
        assert_eq!(placed.len(), 28);
        assert!(title(&Mesh::default(), &sum).contains("no gate answered"));
    }

    #[test]
    fn two_bodies_on_one_commit_are_level() {
        let answer = ANSWER.replace("bbb222", "aaa111");
        let m = Mesh { asked: Some("test".into()), bodies: read_mesh(&answer).unwrap() };
        let (_, _, sum) = build(&read_world(DEFAULT_WORLD).unwrap(), &m);
        assert_eq!(sum.level, Some(true));
        assert!(title(&m, &sum).contains("2 of 10 gates answer in the mesh, 1 silent · level"));
    }

    #[test]
    fn a_body_the_world_file_lacks_is_still_drawn() {
        let answer = ANSWER.replace("ironGate", "moonGate");
        let m = Mesh { asked: Some("test".into()), bodies: read_mesh(&answer).unwrap() };
        let (placed, _, sum) = build(&read_world(DEFAULT_WORLD).unwrap(), &m);
        assert_eq!(sum.unplaced, ["moonGate"]);
        assert_eq!(sum.gates, 11);
        assert_eq!(placed.len(), 34);
    }

    #[test]
    fn only_a_200_answer_is_read() {
        assert_eq!(body_of(b"HTTP/1.0 200 OK\r\nContent-Type: application/json\r\n\r\n{\"a\":1}").unwrap(), "{\"a\":1}");
        assert!(body_of(b"HTTP/1.0 404 Not Found\r\n\r\nno").is_err());
        assert!(body_of(b"garbage").is_err());
    }
}
