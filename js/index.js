import '../styles/style.scss'
import * as bootstrap from 'bootstrap';
import { TileCorruptorAppInst } from '../pkg/index.js';

let rust_app_inst = undefined;

async function choose_new_file(e) {
    let file = e.target.files[0];
    let file_data = await file.arrayBuffer();
    let file_data_u8 = new Uint8Array(file_data);

    if (rust_app_inst !== undefined) {
        rust_app_inst.free();
    }
    rust_app_inst = new TileCorruptorAppInst(file_data_u8);
    rust_app_inst.resize();
    rust_app_inst.render();
}

document.getElementById("file_open").addEventListener("click", (e) => {
    document.getElementById("filechooser").click();
});

document.getElementById("gfx_w_m").addEventListener("click", (e) => {
    if (rust_app_inst !== undefined)
        rust_app_inst.width_minus();
});
document.getElementById("gfx_w_p").addEventListener("click", (e) => {
    if (rust_app_inst !== undefined)
        rust_app_inst.width_plus();
});
document.getElementById("gfx_h_m").addEventListener("click", (e) => {
    if (rust_app_inst !== undefined)
        rust_app_inst.height_minus();
});
document.getElementById("gfx_h_p").addEventListener("click", (e) => {
    if (rust_app_inst !== undefined)
        rust_app_inst.height_plus();
});

document
    .getElementById("filechooser")
    .addEventListener("change", choose_new_file);
