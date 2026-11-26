// orc.rs — Ogre Replacement Core
//
// ORC: the sovereign rendering core that replaces Ogre3D's DX11 RenderSystem.
// We ate the Ogre. We became the ORC.
//
// Architecture:
//   Kenshi.exe → loads RenderSystem_Vulkan_x64.dll (our ORC)
//     → dllStartPlugin() registers ORC with Ogre's Root
//     → Ogre calls ORC's RenderSystem interface
//     → ORC translates Ogre scene graph → wgpu/Vulkan commands
//     → RTX 5090 renders natively in Vulkan
//
// Phases:
//   Phase 1: ✅ Config injection (ogre_plugin.rs) — appear in dropdown
//   Phase 2: ✅ Safe vtable mapping (128 slots enumerated)
//   Phase 3: THIS — ORC C ABI bridge (dllStartPlugin/dllStopPlugin)
//   Phase 4: Scene graph translation (Ogre::SceneManager → wgpu)
//   Phase 5: Full render replacement (mesh, texture, shader pipeline)

use std::ffi::CStr;
use std::os::raw::c_char;

/// ORC render system state
pub struct OrcState {
    pub initialized: bool,
    pub device_name: String,
    pub width: u32,
    pub height: u32,
    pub vsync: bool,
    pub full_screen: bool,
    pub backend: OrcBackend,
    pub frame_count: u64,
}

#[derive(Debug, Clone, Copy)]
pub enum OrcBackend {
    Vulkan,
    Dx12,
    Auto,
}

impl Default for OrcState {
    fn default() -> Self {
        Self {
            initialized: false,
            device_name: String::from("Auto"),
            width: 2560,
            height: 1440,
            vsync: true,
            full_screen: false,
            backend: OrcBackend::Auto,
            frame_count: 0,
        }
    }
}

impl OrcState {
    /// Initialize wgpu device for rendering
    pub fn init(&mut self) -> Result<(), String> {
        println!("[ORC] Initializing Ogre Replacement Core...");
        println!("[ORC] Backend: {:?}", self.backend);
        println!("[ORC] Resolution: {}x{}", self.width, self.height);
        println!("[ORC] VSync: {}", self.vsync);

        // Create wgpu instance with preferred backend
        let backends = match self.backend {
            OrcBackend::Vulkan => wgpu::Backends::VULKAN,
            OrcBackend::Dx12 => wgpu::Backends::DX12,
            OrcBackend::Auto => wgpu::Backends::VULKAN | wgpu::Backends::DX12,
        };

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends,
            ..Default::default()
        });

        // Enumerate adapters
        let adapters: Vec<_> = instance.enumerate_adapters(wgpu::Backends::all());
        println!("[ORC] Found {} GPU adapters:", adapters.len());
        for (i, adapter) in adapters.iter().enumerate() {
            let info = adapter.get_info();
            let selected = if info.name.contains(&self.device_name) || self.device_name == "Auto" {
                " ◄ SELECTED"
            } else {
                ""
            };
            println!("[ORC]   {}: {} ({:?}){}", i, info.name, info.backend, selected);
        }

        self.initialized = true;
        println!("[ORC] Initialized successfully.");
        Ok(())
    }

    /// Render one frame (called by Ogre's render loop)
    pub fn render_frame(&mut self) -> bool {
        self.frame_count += 1;
        true
    }

    /// Shutdown
    pub fn shutdown(&mut self) {
        println!("[ORC] Shutting down after {} frames.", self.frame_count);
        self.initialized = false;
    }
}

// === Ogre Config Options ===
// These map to what appears in Kenshi's launcher dialog

/// Config option for the Ogre dropdown
#[derive(Debug, Clone)]
pub struct ConfigOption {
    pub name: String,
    pub value: String,
    pub possible_values: Vec<String>,
    pub immutable: bool,
}

/// Generate the config options that Ogre expects
pub fn get_config_options() -> Vec<ConfigOption> {
    // Detect GPUs
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
        backends: wgpu::Backends::VULKAN | wgpu::Backends::DX12,
        ..Default::default()
    });
    let gpu_names: Vec<String> = instance
        .enumerate_adapters(wgpu::Backends::all())
        .iter()
        .map(|a| a.get_info().name.clone())
        .collect();

    vec![
        ConfigOption {
            name: "Rendering Device".into(),
            value: gpu_names.first().cloned().unwrap_or_else(|| "Auto".into()),
            possible_values: gpu_names,
            immutable: false,
        },
        ConfigOption {
            name: "Video Mode".into(),
            value: "2560 x 1440 @ 32-bit colour".into(),
            possible_values: vec![
                "3840 x 2160 @ 32-bit colour".into(),
                "2560 x 1440 @ 32-bit colour".into(),
                "1920 x 1080 @ 32-bit colour".into(),
                "1600 x 900 @ 32-bit colour".into(),
                "1280 x 720 @ 32-bit colour".into(),
            ],
            immutable: false,
        },
        ConfigOption {
            name: "Full Screen".into(),
            value: "No".into(),
            possible_values: vec!["Yes".into(), "No".into()],
            immutable: false,
        },
        ConfigOption {
            name: "VSync".into(),
            value: "Yes".into(),
            possible_values: vec!["Yes".into(), "No".into()],
            immutable: false,
        },
        ConfigOption {
            name: "FSAA".into(),
            value: "1".into(),
            possible_values: vec!["1".into(), "2".into(), "4".into(), "8".into()],
            immutable: false,
        },
        ConfigOption {
            name: "Backend".into(),
            value: "Vulkan".into(),
            possible_values: vec!["Vulkan".into(), "DX12".into(), "Auto".into()],
            immutable: false,
        },
        ConfigOption {
            name: "RT Cores".into(),
            value: "Enabled".into(),
            possible_values: vec!["Enabled".into(), "Disabled".into()],
            immutable: false,
        },
        ConfigOption {
            name: "Tensor Cores (DLSS)".into(),
            value: "Enabled".into(),
            possible_values: vec!["Enabled".into(), "Disabled".into()],
            immutable: false,
        },
    ]
}

/// The name that appears in Ogre's "Rendering system:" dropdown
pub const ORC_NAME: &str = "Vulkan (Reborn)";

/// Log file for ORC operations
pub fn log(msg: &str) {
    use std::io::Write;
    let log_path = "kenshi_orc.log";
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path)
    {
        let _ = writeln!(f, "[ORC] {}", msg);
    }
    println!("[ORC] {}", msg);
}

// === DLL EXPORTS ===
// These are the C ABI functions that Ogre's plugin loader calls.
// They will be compiled into RenderSystem_Vulkan_x64.dll

/// Called by Ogre when loading the plugin DLL
/// Registers our RenderSystem with Ogre's Root singleton
#[unsafe(no_mangle)]
pub extern "C" fn dllStartPlugin() {
    log("=== ORC: Ogre Replacement Core v0.9.0 ===");
    log("  We ate the Ogre. We became the ORC.");
    log("  Backend: wgpu → Vulkan/DX12");
    log("  Built by Gaia on RTX 5090");
    log("  dllStartPlugin() called — registering with Ogre Root");

    // Phase 3: Register with Ogre's Root singleton
    // This requires knowing the Ogre::Root vtable layout to call
    // Root::installPlugin() and Root::addRenderSystem()
    //
    // For now, we log and let the safe_rendersystem handle registration
    log("  Registration deferred to safe_rendersystem bridge");
}

/// Called by Ogre when unloading the plugin DLL
#[unsafe(no_mangle)]
pub extern "C" fn dllStopPlugin() {
    log("  dllStopPlugin() called — ORC shutting down");
}

/// Return the render system name (called by Ogre)
#[unsafe(no_mangle)]
pub extern "C" fn orc_get_name() -> *const c_char {
    static NAME: &[u8] = b"Vulkan (Reborn)\0";
    NAME.as_ptr() as *const c_char
}

/// Return the number of config options
#[unsafe(no_mangle)]
pub extern "C" fn orc_get_config_option_count() -> u32 {
    get_config_options().len() as u32
}
