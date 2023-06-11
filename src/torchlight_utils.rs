use crate::memory_utils::MemoryUtils;

#[derive(Debug)]
pub struct TorchlightUtils<'a> {
    pub memory_utils: MemoryUtils<'a>,
    pub hp_addr: i32,
    pub mp_addr: i32,
    pub gold_addr: i32,
}

impl<'a> TorchlightUtils<'a> {
    pub fn new(memory_utils: MemoryUtils) -> TorchlightUtils {
        let hp_addr = TorchlightUtils::get_hp_address(&memory_utils);
        let mp_addr = TorchlightUtils::get_mp_address(&memory_utils);
        let gold_addr = TorchlightUtils::get_gold_address(&memory_utils);

        TorchlightUtils {
            memory_utils,
            hp_addr,
            mp_addr,
            gold_addr,
        }
    }

    pub fn get_hp_address(memory_utils: &MemoryUtils) -> i32 {
        //读取hp的地址
        let hp_addr = memory_utils.r4(memory_utils.r4(0x00400000 + 0x3039094) + 0x14) + 0x558;
        println!("hp_addr为：{:?}", hp_addr);
        hp_addr
    }

    pub fn get_mp_address(memory_utils: &MemoryUtils) -> i32 {
        //读取mp的地址
        let mp_addr = memory_utils.r4(memory_utils.r4(0x00400000 + 0x3039094) + 0x14) + 0x57C;
        println!("mp_addr为：{:?}", mp_addr);
        mp_addr
    }

    pub fn get_gold_address(memory_utils: &MemoryUtils) -> i32 {
        //读取金币的地址
        let gold_addr = memory_utils.r4(memory_utils.r4(0x00400000 + 0x3666F50) + 0x8C) + 0x588;
        println!("money_addr{:?}", gold_addr);
        gold_addr
    }

    //无限血量
    pub fn hp_infinite(&self,mut new_hp: f32) -> bool {
        //写入hp
        if new_hp < 0.0f32 {
            new_hp = 0.0f32;
        }
        self.memory_utils.write_memory(self.hp_addr, new_hp)
    }

    //无限蓝量
    pub fn mp_infinite(&self,mut new_mp: f32) -> bool {
        //写入mp
        if new_mp < 0.0f32 {
            new_mp = 0.0f32;
        }
        self.memory_utils.write_memory(self.mp_addr, new_mp)
    }

    //无限金币
    pub fn gold_infinite(&self,mut new_money:i32) -> bool {
        //写入金币
        if new_money < 0i32 {
            new_money = 0i32;
        }
        self.memory_utils
            .write_memory_int(self.gold_addr, new_money)
    }
}
