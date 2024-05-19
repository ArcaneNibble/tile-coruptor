import("../pkg/index.js").then((rust) => {
    const { TileCorruptorAppInst } = rust;

    let rust_app_inst = undefined;

    async function choose_new_file(e) {
        let file = e.target.files[0];
        let file_data = await file.arrayBuffer();
        let file_data_u8 = new Uint8Array(file_data);

        if (rust_app_inst !== undefined) {
            console.log("free!");
            rust_app_inst.free();
        }
        console.log("about to call rust", file_data_u8);
        rust_app_inst = new TileCorruptorAppInst(file_data_u8);
        rust_app_inst.foobar();
    }

    document
        .getElementById("filechooser")
        .addEventListener("change", choose_new_file);
}).catch(console.error);
