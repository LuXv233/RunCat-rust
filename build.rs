fn main() {
    // 只使用 embed-resource 编译 resources.rc，避免与 winres 重复插入资源
    embed_resource::compile("resources.rc");

    // 触发重建
    println!("cargo:rerun-if-changed=resources.rc");
    for i in 0..5 {
        println!("cargo:rerun-if-changed=cat/cat_{}.ico", i);
    }
}