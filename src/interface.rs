use read_process_memory::*;

const MONSTERS: [u8; 40] = [19, 60, 24, 41, 42, 52, 5, 8, 20, 23, 62, 51, 7, 6, 46, 4, 57, 
    15, 14, 12, 45, 32, 54, 25, 43, 59, 47, 56, 3, 2, 1, 16, 53, 18, 55, 58, 61, 9, 44, 40];

fn read_int(address: usize, handle: &ProcessHandle) -> u32 {
    let bytes = copy_address(address, 4, handle)
        .expect("Interface not connected from emulator.");
    u32::from_le_bytes(bytes.try_into().unwrap())
}

fn read_short(address: usize, handle: &ProcessHandle) -> u32 {
    let mut bytes = copy_address(address, 2, handle)
        .expect("Interface not connected from emulator.");
    bytes.push(0);
    bytes.push(0);
    u32::from_le_bytes(bytes.try_into().unwrap())
}

fn read_byte(address: usize, handle: &ProcessHandle) -> u8 {
    let bytes = copy_address(address, 1, handle)
        .expect("Interface not connected from emulator.");
    bytes[0]
}

fn read_bool(address: usize, handle: &ProcessHandle) -> bool {
    let bytes = copy_address(address, 1, handle)
        .expect("Interface not connected from emulator.");
    bytes[0] != 0
}

fn read_string(mut address: usize, handle: &ProcessHandle) -> String {
    let mut string: String = String::new();
    let mut byte: u8;

    loop {
        byte = read_byte(address, handle);
        if byte == 0_u8 {break;}
        string.push(char::from(byte));
        address += 1;
    }

    string
}

enum ReadableTask {
    Game,
    Lobby
}

enum Task {
    Readable(ReadableTask),
    Unknown
}

pub struct Mhp3rdStatus {
    handle: ProcessHandle,
    base_address: usize,
    pub location: u8,
    pub room: u8,
    pub weapon: u8,
    pub online: bool,
    pub players_in_room: u8,
    pub in_quest: bool,
    pub hunting: Option<u8>,
    pub elapsed_time: u32,
    pub quest_name: Option<String>,
}

impl Mhp3rdStatus {
    pub fn update(&mut self) -> () {
        let task = self.get_task();
        match task {
            Task::Readable(task) => {
                self.location = read_byte((self.base_address + 0x08B2495D).try_into().unwrap(), &self.handle);
                self.room = read_byte((self.base_address + 0x09BA8DCE).try_into().unwrap(), &self.handle);
                self.weapon = read_byte((self.base_address + 0x09B49235).try_into().unwrap(), &self.handle);
                
                self.online = read_bool((self.base_address + 0x08A2991C).try_into().unwrap(), &self.handle);
                if self.online {
                    self.players_in_room = read_byte((self.base_address + 0x09B4684F).try_into().unwrap(), &self.handle);
                } else {
                    self.players_in_room = 0;
                }
                

                match task {
                    ReadableTask::Game => {
                        if read_byte((self.base_address + 0x09BAC044).try_into().unwrap(), &self.handle) > 3 {
                            self.location = 6; // returning from quest
                            self.in_quest = false;
                            self.hunting = None;
                            self.elapsed_time = 0;
                            self.quest_name = None;
                        } else {
                            self.location = 5; // in quest
                            self.in_quest = true;
                            self.quest_name = Some(read_string((self.base_address + 0x08A33F4C).try_into().unwrap(), &self.handle));
                            self.elapsed_time = (read_int((self.base_address + 0x09BAE1D4).try_into().unwrap(), &self.handle) -
                                read_int((self.base_address + 0x09BAE1D8).try_into().unwrap(), &self.handle)) / 30;
                            self.get_monster();
                        }
                    },
                    ReadableTask::Lobby => {
                        self.in_quest = false;
                        self.hunting = None;
                        self.elapsed_time = 0;
                        self.quest_name = None;
                    }
                }
            },
            Task::Unknown => {
                self.location = 4; // In menu
                self.room = 0;
                self.in_quest = false;
                self.weapon = 5;
                self.online = false;
                self.players_in_room = 0;
                self.hunting = None;
                self.quest_name = None;
            }
        }
    }

    fn get_task(&self) -> Task {
        let task = read_string((self.base_address + 0x09C57CA0).try_into().unwrap(), &self.handle);
        if task == "lobby_task.ovl" { 
            Task::Readable(ReadableTask::Lobby) 
        } else if task == "game_task.ovl" {
            Task::Readable(ReadableTask::Game)
        } else {
            Task::Unknown
        }
    }

    fn get_monster(&mut self) -> () {
        let mut idx: usize = 0;
        let mut m_id: u8;
        let mut addr: usize;

        loop {
            if idx > 16 {
                self.hunting = None;
                return ();
            }
            addr = (read_int((self.base_address + 0x9DA9860 + idx).try_into().unwrap(), &self.handle)).try_into().unwrap();
            if addr == 0 {
                self.hunting = None;
                return ();
            }
            m_id = read_byte((self.base_address + addr + 0x62).try_into().unwrap(), &self.handle);
            if read_short((self.base_address + addr + 0x246).try_into().unwrap(), &self.handle) > 0 && MONSTERS.contains(&m_id) {
                break
            }
            idx += 4;
        }

        self.hunting = Some(m_id);
    }

    pub fn new(base_address: usize, handle: ProcessHandle) -> Mhp3rdStatus {
        Mhp3rdStatus {
            handle,
            base_address,
            location: 0_u8,
            room: 0_u8,
            weapon: 5_u8,
            online: false,
            players_in_room: 0_u8,
            in_quest: false,
            hunting: None,
            elapsed_time: 0_u32,
            quest_name: None,
        }
    }
}
