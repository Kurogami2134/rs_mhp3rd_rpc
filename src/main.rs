use discord_rich_presence::{activity, DiscordIpc, DiscordIpcClient};
use read_process_memory::{Pid, ProcessHandle};
pub mod interface;
use interface::Mhp3rdStatus;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use sysinfo::System;
use phf::phf_map;
use windows::Win32::UI::WindowsAndMessaging::EnumWindows;
use windows::Win32::UI::WindowsAndMessaging::GetWindowTextA;
use windows::Win32::UI::WindowsAndMessaging::SendMessageA;
use windows::Win32::Foundation::{HWND, LPARAM, WPARAM, TRUE, FALSE, BOOL};

// CONSTANTS
const MONSTERS: phf::Map<u8, &'static str> = phf_map! {
    0x13_u8 => "agnaktor",
    0x3c_u8 => "akantor",
    0x18_u8 => "alatreon",
    0x29_u8 => "amatsu",
    0x2a_u8 => "arzuros",
    0x34_u8 => "baleful_gigginox",
    0x05_u8 => "barioth",
    0x08_u8 => "barroth",
    0x14_u8 => "black_diablos",
    0x17_u8 => "brute_tigrex",
    0x3e_u8 => "bulldrome",
    0x33_u8 => "crimson_qurupeco",
    0x07_u8 => "deviljho",
    0x06_u8 => "diablos",
    0x2e_u8 => "duramboros",
    0x04_u8 => "gigginox",
    0x39_u8 => "glacial_agnaktor",
    0x0f_u8 => "gold_rathian",
    0x0e_u8 => "great_baggi",
    0x0c_u8 => "great_jaggi",
    0x2d_u8 => "great_wroggi",
    0x20_u8 => "green_nargacuga",
    0x36_u8 => "jade_barroth",
    0x19_u8 => "jhen_mohran",
    0x2b_u8 => "lagombi",
    0x3b_u8 => "nargacuga",
    0x2f_u8 => "nibelsnarf",
    0x38_u8 => "purple_ludroth",
    0x03_u8 => "qurupeco",
    0x02_u8 => "rathalos",
    0x01_u8 => "rathian",
    0x10_u8 => "royal_ludroth",
    0x35_u8 => "sand_barioth",
    0x12_u8 => "silver_rathalos",
    0x37_u8 => "steel_uragaan",
    0x3a_u8 => "tigrex",
    0x3d_u8 => "ukanlos",
    0x09_u8 => "uragaan",
    0x2c_u8 => "volvidon",
    0x28_u8 => "zinogre"
};

const WEAPONS: phf::Map<u8, &'static str> = phf_map! {
    5_u8 => "gs",
    6_u8 => "sns",
    7_u8 => "hm",
    8_u8 => "ln",
    9_u8 => "hbg",
    11_u8 => "lbg",
    12_u8 => "ls",
    13_u8 => "sa",
    14_u8 => "gl",
    15_u8 => "bow",
    16_u8 => "db",
    17_u8 => "hh"
};

const LOCATIONS: &[&str; 6] = &[
    "Yukumo Village",
    "Guild Hall",
    "Yukumo Farm",
    "House",
    "In Menu",
    "In Quest"
];
const LOBBY_IMGS: &[&str; 4] = &[
    "village",
    "guild",
    "farm",
    "house"
];

static mut EMU_HWND: HWND = HWND(0);

fn get_pid() -> Pid {
    let sys = System::new_all();
    let opts = sys.processes_by_name("PPSSPP");
    let cnt = opts.count();
    if cnt == 0 {
        println!("Emulator not found.");
        std::process::exit(0);
    } else {
        if cnt > 1 {
            println!("Multiple emulator processes open, picking first.");
        }
        match sys.processes_by_name("PPSSPP").next() {
            Some(proc) => proc.pid().as_u32(),
            None => {
                println!("Emulator not found.");
                std::process::exit(0)
            }
        }
    }
}

unsafe extern "system" fn enumerate_callback(hwnd: HWND, _lparam: LPARAM) -> BOOL {
    let mut name: [u8; 7] = [0_u8; 7];
    GetWindowTextA(hwnd, &mut name);
    if name == [80_u8, 80, 83, 83, 80, 80, 0] {
        EMU_HWND = hwnd;
        FALSE
    } else {
        TRUE
    }
}

fn get_base_address() -> usize {
    unsafe {
        let a = LPARAM(0);
        if let Ok(_) = EnumWindows(Some(enumerate_callback), a) {
            println!("Emulator window not found.");
            std::process::exit(0);
        }
        let base_address = SendMessageA(EMU_HWND, 0xB118, WPARAM(0), LPARAM(0)).0 as usize;
        let base_address = base_address + ((SendMessageA(EMU_HWND, 0xB118, WPARAM(0), LPARAM(1)).0 as usize) << 32);
        base_address
    }
}

fn main() -> () {
    let pid: Pid = get_pid();
    let handle: ProcessHandle = pid.try_into().unwrap();

    let mut info: Mhp3rdStatus = Mhp3rdStatus::new(get_base_address(), handle);

    let mut client = DiscordIpcClient::new("1110702590997577750").unwrap();

    client.connect().unwrap_or_else(|_| {
        println!("Discord client isn't running.");
        std::process::exit(0);
    });

    loop {
        info.update();
        
        let mut act = activity::Activity::new();
        let mut assets = activity::Assets::new();
        let det: String;
        
        
        if info.online {
            act = act.party(
                activity::Party::new().size([(info.players_in_room).try_into().unwrap(), 4])
            );
        }

        match info.location {
            4 => { // in menu
                assets = assets.large_image("logo").large_text("MHP3rd");
                act = act.state("In Menu");
            }
            5 => { // in quest
                det = match info.quest_name.as_ref() {
                    Some(name) => name.to_string(),
                    None => String::from("Unknown")
                };
                
                assets = assets.large_image(match info.hunting {
                    Some(m_id) => &MONSTERS[&m_id],
                    None => "unknown"
                });
                assets = assets.large_text(match info.hunting {
                    Some(m_id) => &MONSTERS[&m_id],
                    None => "unknown"
                });

                assets = assets.small_image(&WEAPONS[&info.weapon]);
                assets = assets.small_text(&WEAPONS[&info.weapon]);

                act = act.state(
                    if info.online { "In Quest (Online)" } else { "In Quest" }
                );
                
                let now: i64 = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("Time went backwards")
                    .as_secs()
                    .try_into()
                    .unwrap();
                let now: i64 = now - i64::try_from(info.elapsed_time).unwrap();
                act = act.timestamps(activity::Timestamps::new().start(now));

                act = act.details(&det);   
            },
            6 => { // quest return
                assets = assets.large_image("logo").large_text("MHP3rd");
                act = act.state(
                    if info.online {"Returning to Yukumo (Online)"} else {"Returning to Yukumo"}
                );
                
            }
            _ => { // in lobby
                assets = assets.large_image(LOBBY_IMGS[usize::try_from(info.location).unwrap()]);
                act = act.state(LOCATIONS[usize::try_from(info.location).unwrap()]);
                if info.location == 1 {
                    if info.online {
                        det = format!("(Online) | Room: {:0>2}", info.room+1);
                        act = act.details(&det);
                    } else {
                        act = act.details("(Offline)");
                    }
                }
            }
        }

        client.set_activity(act.assets(assets)).unwrap();

        thread::sleep(Duration::from_secs(15))
    }
    
    // client.close().unwrap();
}
