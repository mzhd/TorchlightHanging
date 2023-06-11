# TorchlightHanging
一个玩具挂

#需要替换main.rs中的OgreD3D9Wnd和Torchlight II v.1.13.5.12，通过spy++(+)得到
let memory_utils = MemoryUtils::new("OgreD3D9Wnd", "Torchlight II v.1.13.5.12");

#需要替换torchlight_utils.rs中的基址，如下
let hp_addr = memory_utils.r4(memory_utils.r4(0x00400000 + 0x3039094) + 0x14) + 0x558;

#运行
cargo run 
